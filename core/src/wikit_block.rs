//! Wikit v3 block-table helpers and shared entry I/O.

use crate::error::{WikitError, WikitResult};
use crate::zstdutil;

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Mutex;

pub const WIKIT_FMT_V1: u32 = 0x00_00_00_01;
pub const WIKIT_FMT_V2: u32 = 0x00_00_00_02;
pub const WIKIT_FMT_V3: u32 = 0x00_00_00_03;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBlockInfo {
    pub ulog_off: u64,
    pub ulog_sz: u32,
    pub c_off: u64,
    pub c_sz: u32,
}

#[derive(Debug)]
pub struct BlockCache {
    map: HashMap<usize, Vec<u8>>,
    order: VecDeque<usize>,
    capacity: usize,
}

impl Default for BlockCache {
    fn default() -> Self {
        Self::new(8)
    }
}

impl BlockCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn get(&mut self, idx: usize) -> Option<&Vec<u8>> {
        if self.map.contains_key(&idx) {
            if let Some(pos) = self.order.iter().position(|&x| x == idx) {
                self.order.remove(pos);
            }
            self.order.push_back(idx);
            self.map.get(&idx)
        } else {
            None
        }
    }

    pub fn insert(&mut self, idx: usize, data: Vec<u8>) {
        if self.map.contains_key(&idx) {
            self.map.insert(idx, data);
            if let Some(pos) = self.order.iter().position(|&x| x == idx) {
                self.order.remove(pos);
            }
            self.order.push_back(idx);
            return;
        }
        while self.map.len() >= self.capacity {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            } else {
                break;
            }
        }
        self.map.insert(idx, data);
        self.order.push_back(idx);
    }
}

pub fn parse_block_table(buf: &[u8]) -> WikitResult<(Vec<DataBlockInfo>, &[u8])> {
    if buf.is_empty() {
        return Ok((Vec::new(), buf));
    }
    if buf.len() < 4 {
        return Err(WikitError::new("truncated block table"));
    }
    let bcount = u32::from_be_bytes(buf[0..4].try_into().unwrap()) as usize;
    let need = 4 + bcount * 24;
    if buf.len() < need {
        return Err(WikitError::new(format!(
            "block table too short: need {need}, got {}",
            buf.len()
        )));
    }
    let mut blocks = Vec::with_capacity(bcount);
    let mut off = 4;
    for _ in 0..bcount {
        let ulog_off = u64::from_be_bytes(buf[off..off + 8].try_into().unwrap());
        off += 8;
        let ulog_sz = u32::from_be_bytes(buf[off..off + 4].try_into().unwrap());
        off += 4;
        let c_off = u64::from_be_bytes(buf[off..off + 8].try_into().unwrap());
        off += 8;
        let c_sz = u32::from_be_bytes(buf[off..off + 4].try_into().unwrap());
        off += 4;
        blocks.push(DataBlockInfo {
            ulog_off,
            ulog_sz,
            c_off,
            c_sz,
        });
    }
    Ok((blocks, &buf[need..]))
}

pub fn write_block_table(blocks: &[DataBlockInfo]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + blocks.len() * 24);
    out.extend_from_slice(&(blocks.len() as u32).to_be_bytes());
    for b in blocks {
        out.extend_from_slice(&b.ulog_off.to_be_bytes());
        out.extend_from_slice(&b.ulog_sz.to_be_bytes());
        out.extend_from_slice(&b.c_off.to_be_bytes());
        out.extend_from_slice(&b.c_sz.to_be_bytes());
    }
    out
}

pub fn find_block_index(blocks: &[DataBlockInfo], logical_off: u64) -> WikitResult<usize> {
    let mut lo = 0usize;
    let mut hi = blocks.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        let b = &blocks[mid];
        let end = b.ulog_off + b.ulog_sz as u64;
        if logical_off < b.ulog_off {
            hi = mid;
        } else if logical_off >= end {
            lo = mid + 1;
        } else {
            return Ok(mid);
        }
    }
    Err(WikitError::new(format!(
        "logical offset {logical_off} not found in block table"
    )))
}

pub fn load_block(
    path: &Path,
    blocks: &[DataBlockInfo],
    idx: usize,
    cache: &Mutex<BlockCache>,
) -> WikitResult<Vec<u8>> {
    {
        let mut guard = cache
            .lock()
            .map_err(|_| WikitError::new("block cache lock poisoned"))?;
        if let Some(hit) = guard.get(idx) {
            return Ok(hit.clone());
        }
    }
    let b = blocks
        .get(idx)
        .ok_or_else(|| WikitError::new(format!("bad block index {idx}")))?;
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(b.c_off))?;
    let mut compressed = vec![0u8; b.c_sz as usize];
    file.read_exact(&mut compressed)?;
    let plain = zstdutil::decompress(&compressed, b.ulog_sz as usize)?;
    let mut guard = cache
        .lock()
        .map_err(|_| WikitError::new("block cache lock poisoned"))?;
    guard.insert(idx, plain.clone());
    Ok(plain)
}

pub fn read_entry_from_slice(data: &[u8], offset: usize) -> WikitResult<(u8, Vec<u8>)> {
    if offset + 5 > data.len() {
        return Err(WikitError::new("entry header out of range"));
    }
    let typ = data[offset];
    let sz = u32::from_be_bytes(data[offset + 1..offset + 5].try_into().unwrap()) as usize;
    let start = offset + 5;
    let end = start + sz;
    if end > data.len() {
        return Err(WikitError::new("entry payload out of range"));
    }
    Ok((typ, data[start..end].to_vec()))
}

/// Read a logical entry that may span multiple v3 blocks (legacy dictionaries
/// split the stream on a fixed byte size and could cut mid-entry).
pub fn read_entry_at_logical_offset(
    path: &Path,
    blocks: &[DataBlockInfo],
    logical_off: u64,
    cache: &Mutex<BlockCache>,
) -> WikitResult<(u8, Vec<u8>)> {
    let start_idx = find_block_index(blocks, logical_off)?;
    let mut assembled = load_block(path, blocks, start_idx, cache)?;
    let start_rel = (logical_off - blocks[start_idx].ulog_off) as usize;
    let mut next_idx = start_idx + 1;

    // Pull in following blocks until header + payload are fully available.
    loop {
        if start_rel + 5 <= assembled.len() {
            let sz =
                u32::from_be_bytes(assembled[start_rel + 1..start_rel + 5].try_into().unwrap())
                    as usize;
            if start_rel + 5 + sz <= assembled.len() {
                return read_entry_from_slice(&assembled, start_rel);
            }
        }
        if next_idx >= blocks.len() {
            return Err(WikitError::new("entry payload out of range"));
        }
        let more = load_block(path, blocks, next_idx, cache)?;
        assembled.extend_from_slice(&more);
        next_idx += 1;
    }
}

pub fn append_logical_entry(buf: &mut Vec<u8>, typ: u8, payload: &[u8]) -> u64 {
    let offset = buf.len() as u64;
    buf.push(typ);
    buf.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    buf.extend_from_slice(payload);
    offset
}

pub fn compress_logical_stream(logical: &[u8]) -> WikitResult<(Vec<DataBlockInfo>, Vec<u8>)> {
    let mut blocks = Vec::new();
    let mut compressed_blob = Vec::new();
    let mut cursor = 0usize;
    while cursor < logical.len() {
        let block_start = cursor;
        let target_end = block_start.saturating_add(zstdutil::DEFAULT_BLOCK_UNCOMPRESSED);

        // Pack whole entries only — never split typ/size/payload across blocks.
        while cursor < logical.len() {
            if cursor + 5 > logical.len() {
                return Err(WikitError::new("truncated logical entry header while packing blocks"));
            }
            let sz = u32::from_be_bytes(logical[cursor + 1..cursor + 5].try_into().unwrap()) as usize;
            let entry_end = cursor + 5 + sz;
            if entry_end > logical.len() {
                return Err(WikitError::new("truncated logical entry payload while packing blocks"));
            }
            if cursor > block_start && entry_end > target_end {
                break;
            }
            cursor = entry_end;
            if cursor >= target_end {
                break;
            }
        }

        let chunk = &logical[block_start..cursor];
        let compressed = zstdutil::compress(chunk, zstdutil::DEFAULT_BLOCK_LEVEL)?;
        let c_off = compressed_blob.len() as u64; // relative; caller adds dbase
        blocks.push(DataBlockInfo {
            ulog_off: block_start as u64,
            ulog_sz: chunk.len() as u32,
            c_off,
            c_sz: compressed.len() as u32,
        });
        compressed_blob.extend_from_slice(&compressed);
    }
    if blocks.is_empty() {
        // Empty dictionary still needs one empty block for a consistent table.
        let compressed = zstdutil::compress(&[], zstdutil::DEFAULT_BLOCK_LEVEL)?;
        blocks.push(DataBlockInfo {
            ulog_off: 0,
            ulog_sz: 0,
            c_off: 0,
            c_sz: compressed.len() as u32,
        });
        compressed_blob.extend_from_slice(&compressed);
    }
    Ok((blocks, compressed_blob))
}

pub fn normalize_resource_key(key: &str) -> String {
    key.trim()
        .trim_start_matches(|c| c == '/' || c == '\\')
        .replace('\\', "/")
        .to_lowercase()
}

pub fn resource_key_aliases(key: &str) -> Vec<String> {
    let norm = normalize_resource_key(key);
    let mut aliases = vec![norm.clone()];
    if let Some(base) = norm.rsplit('/').next() {
        if !base.is_empty() && base != norm {
            aliases.push(base.to_string());
        }
    }
    aliases.sort();
    aliases.dedup();
    aliases
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    const TXT: u8 = 0x1; // DataEntryType::TXT

    #[test]
    fn compress_does_not_split_entries() {
        let mut logical = Vec::new();
        for payload in [vec![1u8; 100], vec![2u8; 100], vec![3u8; 100]] {
            append_logical_entry(&mut logical, TXT, &payload);
        }
        let (blocks, blob) = compress_logical_stream(&logical).expect("compress");
        assert!(!blocks.is_empty());
        assert!(!blob.is_empty());
        let mut offsets = vec![0usize];
        let mut cur = 0usize;
        while cur < logical.len() {
            let sz = u32::from_be_bytes(logical[cur + 1..cur + 5].try_into().unwrap()) as usize;
            cur += 5 + sz;
            offsets.push(cur);
        }
        for b in &blocks {
            assert!(
                offsets.contains(&(b.ulog_off as usize)),
                "block ulog_off {} is mid-entry",
                b.ulog_off
            );
        }
    }

    #[test]
    fn read_entry_spans_legacy_split_blocks() {
        let mut logical = Vec::new();
        let payload = vec![7u8; 40];
        let off = append_logical_entry(&mut logical, TXT, &payload);
        assert_eq!(off, 0);

        let split = 10usize;
        let left = zstdutil::compress(&logical[..split], 1).unwrap();
        let right = zstdutil::compress(&logical[split..], 1).unwrap();
        let mut file_bytes = Vec::new();
        file_bytes.extend_from_slice(&left);
        let right_off = file_bytes.len() as u64;
        file_bytes.extend_from_slice(&right);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), &file_bytes).unwrap();

        let blocks = vec![
            DataBlockInfo {
                ulog_off: 0,
                ulog_sz: split as u32,
                c_off: 0,
                c_sz: left.len() as u32,
            },
            DataBlockInfo {
                ulog_off: split as u64,
                ulog_sz: (logical.len() - split) as u32,
                c_off: right_off,
                c_sz: right.len() as u32,
            },
        ];
        let cache = Mutex::new(BlockCache::new(4));
        let (typ, got) =
            read_entry_at_logical_offset(tmp.path(), &blocks, 0, &cache).expect("span read");
        assert_eq!(typ, TXT);
        assert_eq!(got, payload);
    }
}
