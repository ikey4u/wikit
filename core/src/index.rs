/// This module is used to build index for dictionary

use crate::error::{WikitResult};

use std::io::{SeekFrom};
use std::fs::File;

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Streamer, Map, MapBuilder};
use memmap::MmapOptions;

#[derive(Debug, Clone)]
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

#[derive(Debug)]
pub struct FSTIndex {
    path: std::path::PathBuf,
    offset: u64,
    length: u64,
}

impl FSTIndex {
    /// Create index from iterator of `(keyword, offset) of type (&str, u64)`,
    /// the keyword must be lexicographically ordered and has no duplications.
    pub fn write<S, W>(iter: &mut dyn Iterator<Item = &(S, u64)>, writer: &mut W) -> WikitResult<(u64, u64)>
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

    pub fn lookup<P>(&self, keyword: P) -> WikitResult<Vec<(String, u64)>> where P: AsRef<str> {
        let file = File::open(&self.path)?;
        let mmap = unsafe { MmapOptions::new().offset(self.offset).len(self.length as usize).map(&file)? };
        let map = Map::new(mmap)?;

        let query = Levenshtein::new(keyword.as_ref(), 2)?;
        let mut stream = map.search(&query).into_stream();

        let mut r = vec![];
        let (mut cnt, limit) = (0, 20);
        while let Some((k, v)) = stream.next() {
            r.push((String::from_utf8(k.to_vec())?, v));
            cnt += 1;
            if cnt >= limit {
                break;
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
