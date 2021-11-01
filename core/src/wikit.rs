/// Create or load wikit dictionary

use crate::error::{WikitError, Context, WikitResult, AnyResult, NomResult};
use crate::elog;
use crate::index;
use crate::mdict;

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufWriter, Write, Seek, SeekFrom, Read};

use serde::{Deserialize, Serialize};
use nom::{do_parse, map_res, take};
use nom::number::streaming::{be_u16, be_u64};

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

#[derive(Debug, Clone)]
pub struct WikitHead {
    // dictionary standard name
    pub name: String,
    // detail description of dictionary
    pub desc: String,
    // index format
    pub ifmt: index::IndexFormat,
    // index offset from file start
    pub ibase: u64,
    // index size
    pub isz: u64,
    // data offset from file start
    pub dbase: u64,
    // data size, data is essential a vector of DataEntry
    pub dsz: u64,
}

impl WikitHead {
    pub fn new(headbuf: &[u8]) -> WikitResult<Self> {
        let r: NomResult<WikitHead> = do_parse!(headbuf,
            namesz: be_u16 >>
            name: map_res!(take!(namesz),
                |x: &[u8]| -> AnyResult<String> {
                    let name = String::from_utf8(x.to_vec()).context(elog!("cannot get name"))?;
                    Ok(name)
                }
            ) >>
            descsz: be_u16 >>
            desc: map_res!(take!(descsz),
                |x: &[u8]| -> AnyResult<String> {
                    let desc = String::from_utf8(x.to_vec()).context(elog!("cannot get desc"))?;
                    Ok(desc)
                }
            ) >>
            ifmt: map_res!(take!(1),
                |x: &[u8]| -> AnyResult<index::IndexFormat> {
                    index::IndexFormat::new(x[0]).ok_or(anyhow::anyhow!("unknown index format"))
                }
            ) >>
            ibase: be_u64 >>
            isz: be_u64 >>
            dbase: be_u64 >>
            dsz: be_u64 >>
            (
                WikitHead {
                    name,
                    desc,
                    ifmt,
                    ibase,
                    isz,
                    dbase,
                    dsz,
                }
            )
        );
        Ok(r.unwrap().1)
    }
}

pub struct WikitDictionary {
    pub head: WikitHead,
    // local path of dictionary
    path: PathBuf,
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
        //      ibase:8
        //      isz:8
        //      dbase:8
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
        writer.seek(SeekFrom::Current(8))?;
        // dsz
        let dsz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;

        // save header size
        let hdrsz = writer.seek(SeekFrom::Current(0))? as u16;
        writer.seek(SeekFrom::Start(hdrsz_pos))?;
        writer.write(&hdrsz.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(hdrsz as u64))?;

        match WikitSourceType::new(dict_conf.source) {
            Some(WikitSourceType::Mdict) => {
                let mdxpath = path.join(dict_conf.name.clone() + ".mdx");
                let _mddpath = path.join(dict_conf.name.clone() + ".mdd");
                let mdx_path_str = &format!("{}", mdxpath.display());
                let mut word_meaning_list = mdict::parse_mdx(mdx_path_str, None)?;
                // sort word by ascending
                word_meaning_list.sort_by(|a, b| a.0.cmp(&b.0));
                // remove duplicate word
                word_meaning_list.dedup_by(|a, b| a.0.eq(&b.0));

                let dstart = writer.seek(SeekFrom::Current(0))?;
                let mut index_table = vec![];
                for (word, meaning) in word_meaning_list.iter() {
                    let entry = DataEntry::new(DataEntryType::Word, meaning.len() as u32, meaning.as_bytes());
                    let (offset, _count) = entry.write(&mut writer)?;
                    index_table.push((word, offset));
                }
                let dend = writer.seek(SeekFrom::Current(0))?;

                let dbase = dstart as u64;
                writer.seek(SeekFrom::Start(dbase_pos))?;
                writer.write(&dbase.to_be_bytes()[..])?;
                let dsz = (dend - dstart) as u64;
                writer.seek(SeekFrom::Start(dsz_pos))?;
                writer.write(&dsz.to_be_bytes()[..])?;

                writer.seek(SeekFrom::Start(dend))?;
                let (ibase, isz) = index::FSTIndex::write(&mut index_table.iter(), &mut writer)?;
                let (ibase, isz) = (ibase as u64, isz as u64);
                writer.seek(SeekFrom::Start(ibase_pos))?;
                writer.write(&ibase.to_be_bytes()[..])?;
                writer.seek(SeekFrom::Start(isz_pos))?;
                writer.write(&isz.to_be_bytes()[..])?;
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

    pub fn load<P>(path: P) -> WikitResult<Self> where P: AsRef<Path> {
        let path = path.as_ref();
        let mut file = File::open(path)?;

        let mut magic = [0u8; WIKIT_MAGIC.len()];
        file.read_exact(&mut magic)?;
        let magic = String::from_utf8(magic.to_vec())?;
        if magic != WIKIT_MAGIC {
            return Err(WikitError::new("Wrong wikit magic"));
        }

        let mut version = [0u8; 4];
        file.read_exact(&mut version)?;
        let version = u32::from_be_bytes(version);
        if version != LATEST_WIKIT_FMT_VERSION {
            return Err(WikitError::new("Wrong wikit version"));
        }

        let mut hdrsz = [0u8; 2];
        file.read_exact(&mut hdrsz)?;
        let hdrsz = u16::from_be_bytes(hdrsz) as usize;

        let hdrbuf = file.bytes().take(hdrsz).filter_map(Result::ok).collect::<Vec<u8>>();
        if hdrbuf.len() != hdrsz {
            return Err(WikitError::new("Wikit header is broken"));
        }
        let wikit_head = WikitHead::new(&hdrbuf[..])?;

        Ok(WikitDictionary {
            head: wikit_head.clone(),
            path: path.to_path_buf(),
            idx: index::FSTIndex::new(path.to_path_buf(), wikit_head.ibase, wikit_head.isz),
        })
    }

    pub fn lookup<P>(&self, word: P) -> WikitResult<Vec<(String, String)>> where P: AsRef<str> {
        if let Ok(poslist) = self.idx.lookup(word) {
            let mut file = File::open(&self.path)?;
            let mut anslist = vec![];
            for (word, offset) in poslist {
                let file = std::io::Read::by_ref(&mut file);
                // just ignore DataEntryType
                file.seek(SeekFrom::Start(offset + 1))?;
                let mut meaning_size = [0u8; 4];
                file.read_exact(&mut meaning_size)?;
                let meaning_size = u32::from_be_bytes(meaning_size) as usize;
                let meaning_buf = file.bytes().take(meaning_size).filter_map(Result::ok).collect::<Vec<u8>>();
                if meaning_buf.len() == meaning_size {
                    anslist.push((word.to_string(), String::from_utf8(meaning_buf)?));
                }
            }
            return Ok(anslist);
        }
        return Err(WikitError::new("No such word or similar words"));
    }

    pub fn unpack() {
    }
}
