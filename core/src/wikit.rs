/// Create or load wikit dictionary

use crate::error::{WikitError, WikitResult};
use crate::index;
use crate::mdict;
use crate::util;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, Seek, SeekFrom, Read};

use serde::{Deserialize, Serialize};

// `516` is the birthday of wikit project (the first commit date 2021-05-16)
const WIKIT_MAGIC: &'static str = "WIKIT516";
// the latest wikit dictionary format version
const LATEST_WIKIT_FMT_VERSION: u32 = 0x00_00_00_01;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum DataEntryType {
    Word = 0x1,
    Meaning = 0x2,
    CSS = 0x3,
    JS = 0x4,
    SVG = 0x5,
    PNG = 0x6,
    JPG = 0x7,
    MP3 = 0x8,
    WAV = 0x9,
    MP4 = 0xa,
}

#[derive(Debug)]
pub enum WikitSourceType {
    /// This type directory should contain `x.mdx` (must have), `x.mdd` (optional), `x.css`
    /// (optional), `x.js` (optional) where `x` is the dictionary name.
    Mdict,
    /// This type dictionary should contain `x.txt` (must have), `x.media` (it is an optional
    /// directory and if it exists, it should contain optional `img`, `video`, `audio` subdirectory)
    /// .  The `x` is the dictionary name.
    Wikit,
}

impl WikitSourceType {
    pub fn new<P>(text: P) -> Option<Self> where P: AsRef<str> {
        let text = text.as_ref();
        match text {
            "WIKIT" => Some(WikitSourceType::Wikit),
            "MDICT" => Some(WikitSourceType::Mdict),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WikitSourceConf {
    name: String,
    desc: String,
    output: String,
    source: String,
}

#[derive(Debug)]
struct DataEntry<'a> {
    typ: DataEntryType,
    sz: u32,
    buf: &'a [u8],
}

impl<'a> DataEntry<'a> {
    pub fn new(typ: DataEntryType, sz: u32, buf: &'a [u8]) -> Self {
        Self { typ, sz, buf }
    }

    pub fn write<W>(&self, writer: &mut W) -> WikitResult<(u64, u64)> where W: std::io::Write + std::io::Seek {
        let start = writer.seek(SeekFrom::Current(0))?;
        writer.write(&[self.typ as u8])?;
        writer.write(&self.sz.to_be_bytes()[..])?;
        writer.write(&self.buf)?;
        let end = writer.seek(SeekFrom::Current(0))?;
        Ok((start, end - start))
    }
}

#[derive(Debug)]
struct WikitHead {
    // format version of wikit dictionary
    pub version: u32,
    // dictionary standard name
    pub name: String,
    // detail description of dictionary
    pub desc: String,
    // index format
    pub ifmt: index::IndexFormat,
    // index offset from file start
    pub ibase: u32,
    // index size
    pub isz: u64,
    // data offset from file start
    pub dbase: u32,
    // data size, data is essential a vector of DataEntry
    pub dsz: u64,
}

impl WikitHead {
    fn size(&self) -> Option<u16> {
        let s =
            // version
            4
            + self.name.len()
            + self.desc.len()
            // ibase
            + 4
            // idxsz
            + 8
            // dbase
            + 4
            // dsz
            + 8;
        if s > (u16::MAX as usize) {
            return None;
        } else {
            return Some(s as u16);
        }
    }
}

struct WikitDictionary {
    pub head: WikitHead,
    // local path of dictionary
    path: String,
    idx: index::FSTIndex,
}

impl WikitDictionary {
    /// Create wikit dictionary from specific dictionary
    ///
    /// `path` should be a directory and contains a `wikit.toml` configuration file, see
    /// [WikitSourceConf] for more details.
    pub fn new_from<P>(path: P) -> WikitResult<bool> where P: AsRef<Path> {
        let path = path.as_ref();

        let mut dict_conf_file = File::open(path.join("wikit.toml"))?;
        let mut dict_conf = String::new();
        dict_conf_file.read_to_string(&mut dict_conf)?;
        let dict_conf = toml::from_str::<WikitSourceConf>(&dict_conf)?;

        // wikit dictionary starts_with with `magic` and `version` fields
        //
        //      magic:8
        //      version:4
        //
        // if version is 0x01, then the follwoing layout is
        //
        //      hdrsz:2
        //      namesz:2
        //      name:namesz
        //      descsz:2
        //      desc:descsz
        //      ifmt:1
        //      ibase:4
        //      isz:8
        //      dbase:4
        //      dsz:8
        //
        let outfile = path.join(dict_conf.name.clone() + ".wikit");
        let mut writer = BufWriter::new(File::create(&outfile)?);
        // magic
        writer.write(WIKIT_MAGIC.as_bytes())?;
        writer.write(&LATEST_WIKIT_FMT_VERSION.to_be_bytes()[..])?;
        // hdrsz
        let hdrsz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(2))?;
        // namesz and name
        let namesz = dict_conf.name.len() as u16;
        writer.write(&namesz.to_be_bytes()[..])?;
        writer.write(&dict_conf.name.as_bytes()[..])?;
        // descsz and desc
        let descsz = dict_conf.desc.len() as u16;
        writer.write(&descsz.to_be_bytes()[..])?;
        writer.write(&dict_conf.desc.as_bytes()[..])?;
        // ifmt
        writer.write(&[index::IndexFormat::FST as u8])?;
        // ibase
        let ibase_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;
        // isz
        let isz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;
        // dbase
        let dbase_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(4))?;
        // dsz
        let dsz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;

        match WikitSourceType::new(dict_conf.source) {
            Some(WikitSourceType::Mdict) => {
                let mdxpath = path.join(dict_conf.name.clone() + ".mdx");
                let _mddpath = path.join(dict_conf.name.clone() + ".mdd");
                let mdx_path_str = &format!("{}", mdxpath.display());
                // let word_meaning_list = mdict::parse_mdx(mdx_path_str, None)?;
                let mut word_meaning_list = vec![
                    ("b", "bb"),
                    ("a", "aaaaa"),
                    ("c", "ccccc"),
                    ("a", "ccccc"),
                ];
                // sort word by ascending
                word_meaning_list.sort_by(|a, b| a.0.cmp(b.0));
                // remove duplicate word
                word_meaning_list.dedup_by(|a, b| a.0.eq(b.0));
                let mut index_table = vec![];
                for (word, meaning) in word_meaning_list.iter() {
                    let entry = DataEntry::new(DataEntryType::Word, meaning.len() as u32, meaning.as_bytes());
                    let (offset, count) = entry.write(&mut writer)?;
                    index_table.push((word, offset));
                    println!("offset: {}; count: {}", offset, count);
                }
                println!("index_table: {:?}", index_table);
                let (ibase, isz) = index::FSTIndex::write(&mut index_table.iter(), &mut writer)?;
                println!("ibase: {}; isz: {}", ibase, isz);
            },
            Some(WikitSourceType::Wikit) => {
                todo!()
            }
            _ => {
                return Err(WikitError::new("unkown source type"));
            }
        }

        Ok(true)
    }

    pub fn unpack() {
    }

    pub fn load<P>(path: P) -> WikitResult<bool> where P: AsRef<Path> {
        let path = path.as_ref();
        Ok(true)
    }

    pub fn lookup(&self) {
    }
}

#[test]
fn debug() {
}
