/// This module is used to build index for dictionary
use crate::error::WikitResult;

use std::collections::HashSet;
use std::fs::File;
use std::io::SeekFrom;

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Map, MapBuilder, Streamer};
use memmap::MmapOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum IndexFormat {
    FST = 1,
}

impl IndexFormat {
    pub fn new(v: u8) -> Option<IndexFormat> {
        match v {
            1u8 => Some(IndexFormat::FST),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FSTIndex {
    path: std::path::PathBuf,
    offset: u64,
    length: u64,
}

/// Allowed Levenshtein distance for fuzzy headword lookup.
///
/// `fst::automaton::Levenshtein` already counts edits in Unicode scalar values
/// (characters), not bytes. The threshold must use the same unit — `chars().count()` —
/// otherwise a short CJK query like "测试" (2 chars / 6 bytes) is given distance 2
/// and matches unrelated Latin/symbol keys (e.g. `%`, `'a`) within two char edits.
fn fuzzy_edit_distance(keyword: &str) -> u32 {
    match keyword.chars().count() {
        0 | 1 | 2 => 0,
        3 | 4 | 5 => 1,
        _ => 2,
    }
}

impl FSTIndex {
    /// Create index from iterator of `(keyword, offset) of type (&str, u64)`,
    /// the keyword must be lexicographically ordered and has no duplications.
    pub fn write<S, W>(
        iter: &mut dyn Iterator<Item = &(S, u64)>,
        writer: &mut W,
    ) -> WikitResult<(u64, u64)>
    where
        S: AsRef<str>,
        W: std::io::Write + std::io::Seek,
    {
        let start = writer.seek(SeekFrom::Current(0))?;
        let mut fst_builder = MapBuilder::new(writer)?;
        for (keyword, offset) in iter {
            fst_builder.insert(&keyword.as_ref()[..], *offset)?;
        }
        let writer = fst_builder.into_inner()?;
        let end = writer.seek(SeekFrom::Current(0))?;
        Ok((start as u64, (end - start) as u64))
    }

    pub fn format(&self) -> IndexFormat {
        IndexFormat::FST
    }

    pub fn lookup<P>(&self, keyword: P) -> WikitResult<Vec<(String, u64)>>
    where
        P: AsRef<str>,
    {
        let file = File::open(&self.path)?;
        let mmap = unsafe {
            MmapOptions::new()
                .offset(self.offset)
                .len(self.length as usize)
                .map(&file)?
        };
        let map = Map::new(mmap)?;

        let keyword = keyword.as_ref();
        let mut r = vec![];
        let mut seen = HashSet::new();
        if let Some(v) = map.get(keyword) {
            let key = keyword.to_string();
            seen.insert(key.clone());
            r.push((key, v));
        }

        let fuzzycnt = fuzzy_edit_distance(keyword);
        if fuzzycnt > 0 {
            let query = Levenshtein::new(keyword, fuzzycnt)?;
            let mut stream = map.search(&query).into_stream();
            let (mut cnt, limit) = (0, 20);
            while let Some((k, v)) = stream.next() {
                let key = String::from_utf8(k.to_vec())?;
                if !seen.insert(key.clone()) {
                    continue;
                }
                r.push((key, v));
                cnt += 1;
                if cnt >= limit {
                    break;
                }
            }
        }
        Ok(r)
    }

    pub fn new(path: std::path::PathBuf, offset: u64, length: u64) -> Self {
        Self {
            path,
            offset,
            length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fuzzy_edit_distance;

    #[test]
    fn fuzzy_distance_uses_unicode_char_count() {
        assert_eq!(fuzzy_edit_distance("a"), 0);
        assert_eq!(fuzzy_edit_distance("ab"), 0);
        assert_eq!(fuzzy_edit_distance("abc"), 1);
        assert_eq!(fuzzy_edit_distance("hello"), 1);
        assert_eq!(fuzzy_edit_distance("testing"), 2);

        // CJK: character count, not UTF-8 byte length.
        assert_eq!("测试".len(), 6);
        assert_eq!("测试".chars().count(), 2);
        assert_eq!(fuzzy_edit_distance("测"), 0);
        assert_eq!(fuzzy_edit_distance("测试"), 0);
        assert_eq!(fuzzy_edit_distance("计算机"), 1);
        assert_eq!(fuzzy_edit_distance("奥林匹克运动会"), 2);
    }
}
