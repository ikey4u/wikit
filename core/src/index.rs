/// This module is used to build index for dictionary

use crate::error::{WikitError, WikitResult};

use std::io::{BufReader, BufWriter, Write, Seek, SeekFrom, Read};

use fst::automaton::Levenshtein;
use fst::{IntoStreamer, Streamer, Map, MapBuilder};
use memmap::Mmap;
use memmap::MmapOptions;

#[derive(Debug)]
#[repr(u8)]
pub enum IndexFormat {
    FST = 1,
}

#[derive(Debug)]
pub struct FSTIndex;

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

    pub fn lookup(&self) -> u64 {
        0
    }
}
