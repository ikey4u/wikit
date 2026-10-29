use crate::config;
use crate::elog;
/// Create or load wikit dictionary
use crate::error::{AnyResult, Context, NomResult, WikitError, WikitResult};
use crate::index;
use crate::mdict;
use crate::reader;
use crate::util;

use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use nom::number::streaming::{be_u16, be_u32, be_u64};
use nom::{do_parse, map_res, take};
use serde::{Deserialize, Serialize};
use wikit_proto::DictMeta;

// `516` is the birthday of wikit project (the first commit date 2021-05-16)
const WIKIT_MAGIC: &'static str = "WIKIT516";
// the latest wikit dictionary format version
const LATEST_WIKIT_FMT_VERSION: u32 = 0x00_00_00_01;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum DataEntryType {
    TXT = 0x1,
    SVG = 0x2,
    PNG = 0x3,
    JPG = 0x4,
    MP3 = 0x5,
    WAV = 0x6,
    // Do we really need this?
    MP4 = 0x7,
}

#[derive(Debug)]
pub enum WikitSourceType {
    /// This type directory should contain `x.mdx` (must have), `x.mdd` (optional), `x.css`
    /// (optional), `x.js` (optional) where `x` is the dictionary name.
    Mdict,
    /// This type dictionary should contain `x.txt` (must have), `x.media` (it is an optional
    /// directory and if it exists, it should contain optional `img`, `video`, `audio` subdirectory)
    /// . The `x` is the dictionary name.
    Wikit,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WikitDictProfile {
    name: String,
    version: String,
    authors: Vec<String>,
    distributors: Vec<String>,
    description: String,
    homepage: String,
    css: crate::resources::Resources,
    js: crate::resources::Resources,
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

    pub fn write<W>(&self, writer: &mut W) -> WikitResult<(u64, u64)>
    where
        W: std::io::Write + std::io::Seek,
    {
        let start = writer.seek(SeekFrom::Current(0))?;
        writer.write(&[self.typ as u8])?;
        writer.write(&self.sz.to_be_bytes()[..])?;
        writer.write(&self.buf)?;
        let end = writer.seek(SeekFrom::Current(0))?;
        Ok((start, end - start))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    // javascript script
    pub script: String,
    // css style
    pub style: String,
}

impl WikitHead {
    pub fn new(headbuf: &[u8]) -> WikitResult<Self> {
        let r: NomResult<WikitHead> = do_parse!(headbuf,
            namesz: be_u16 >>
            name: map_res!(take!(namesz),
                |x: &[u8]| -> AnyResult<String> {
                    let name = String::from_utf8(x.to_vec())
                        .context(elog!("failed to get wikti dictionary name"))?;
                    Ok(name)
                }
            ) >>
            descsz: be_u16 >>
            desc: map_res!(take!(descsz),
                |x: &[u8]| -> AnyResult<String> {
                    let desc = String::from_utf8(x.to_vec())
                        .context(elog!("failed to get dictionary description"))?;
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
            scriptsz: be_u32 >>
            script: map_res!(take!(scriptsz),
                |x: &[u8]| -> AnyResult<String> {
                    let script = String::from_utf8(x.to_vec()).context(elog!("cannot get script"))?;
                    Ok(script)
                }
            ) >>
            stylesz: be_u32 >>
            style: map_res!(take!(stylesz),
                |x: &[u8]| -> AnyResult<String> {
                    let style = String::from_utf8(x.to_vec()).context(elog!("cannot get style"))?;
                    Ok(style)
                }
            ) >>
            (
                WikitHead {
                    name,
                    desc,
                    ifmt,
                    ibase,
                    isz,
                    dbase,
                    dsz,
                    script,
                    style,
                }
            )
        );

        match r {
            Ok(r) => Ok(r.1),
            Err(e) => Err(WikitError::new(format!(
                "failed to parse WikitHead: {:?}",
                e
            ))),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum WikitDictionary {
    Local(LocalDictionary),
    Remote(RemoteDictionary),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RemoteDictionary {
    pub url: String,
    user: String,
    token: String,
}

impl RemoteDictionary {
    pub fn new(url: String, user: String, token: String) -> Self {
        RemoteDictionary { url, user, token }
    }

    fn get(
        &self,
        operation: &str,
        query: &[(&str, &str)],
    ) -> WikitResult<reqwest::blocking::Response> {
        let mut url = url::Url::parse(&self.url).map_err(|e| WikitError::new(e.to_string()))?;
        url.set_path(&format!(
            "{}/wikit/{operation}",
            url.path().trim_end_matches('/')
        ));
        url.set_query(None);
        url.set_fragment(None);
        let user = if self.user.is_empty() {
            url.username().to_owned()
        } else {
            self.user.clone()
        };
        let password = if self.token.is_empty() {
            url.password().unwrap_or("").to_owned()
        } else {
            self.token.clone()
        };
        let _ = url.set_username("");
        let _ = url.set_password(None);
        url.query_pairs_mut().extend_pairs(query.iter().copied());
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()?;
        let mut request = client.get(url);
        if !user.is_empty() {
            request = request.basic_auth(user, Some(password));
        }
        Ok(request.send()?.error_for_status()?)
    }
    pub fn get_dict_list(&self) -> WikitResult<Vec<DictMeta>> {
        Ok(self.get("list", &[])?.json()?)
    }
    pub fn lookup<P>(&self, word: P, dict: P) -> WikitResult<Vec<(String, String)>>
    where
        P: AsRef<str>,
    {
        Ok(self
            .get(
                "query",
                &[("word", word.as_ref()), ("dictname", dict.as_ref())],
            )?
            .json()?)
    }
    pub fn get_script<S: AsRef<str>>(&self, dict: S) -> String {
        self.get("script", &[("dictname", dict.as_ref())])
            .and_then(|r| r.text().map_err(Into::into))
            .unwrap_or_default()
    }
    pub fn get_style<S: AsRef<str>>(&self, dict: S) -> String {
        self.get("style", &[("dictname", dict.as_ref())])
            .and_then(|r| r.text().map_err(Into::into))
            .unwrap_or_default()
    }
}

/// LocalDictionary represents a wikit dictionary.
///
/// wikit dictionary starts_with with `magic` and `version` fields
///
/// ```text
///      magic:8
///      version:4
///
/// ```
///
/// if version is 0x01, then the follwoing layout is
///
/// ```text
///      hdrsz:4
///      namesz:2
///      name:namesz
///      descsz:2
///      desc:descsz
///      ifmt:1
///      ibase:8
///      isz:8
///      dbase:8
///      dsz:8
///      scriptsz: 4
///      script: scriptsz
///      stylesz: 4
///      style: stylesz
///
///      data: dsz
///      index: isz
///
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LocalDictionary {
    pub head: WikitHead,
    // local path of dictionary
    pub path: PathBuf,
    idx: index::FSTIndex,
}

impl LocalDictionary {
    /// Create wikit dictionary from wikit source file
    ///
    /// `srcfile` is absolute path to wikit source file (txt or mdx) such as `/some/dir/dict.mdx`
    /// or `/some/dir/dict.txt`, `outfile` is optional, if it is none, then the output file will be
    /// `/some/dir/dict.wikit`.
    ///
    /// Moreover, you can provide a file named `dict.toml` alonside with your dictionary such as
    /// `/some/dir/dict.toml` to describe your dictionary, see [WikitDictProfile] for more details.
    pub fn create<P, Q>(srcfile: P, outfile: Option<Q>) -> WikitResult<PathBuf>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let srcfile = srcfile.as_ref();
        let (pdir, stem, suffix) = util::parse_path(srcfile).context(elog!(
            "failed to get parent directory of {}",
            srcfile.display()
        ))?;

        let mut conf = String::new();
        if let Ok(mut f) = File::open(pdir.join(stem.clone() + ".toml")) {
            f.read_to_string(&mut conf)?;
        };
        let mut profile = if conf.trim().is_empty() {
            WikitDictProfile::default()
        } else {
            toml::from_str::<WikitDictProfile>(&conf)?
        };
        if profile.name.is_empty() {
            profile.name = stem.clone();
        }
        let conf = profile;
        let style = conf.css.load(&pdir, "css")?;
        let script = conf.js.load(&pdir, "js")?;

        let outfile = if let Some(outfile) = outfile {
            let outfile = outfile.as_ref();
            if outfile.exists() && outfile.is_dir() {
                return Err(WikitError::new(
                    "output path must be a file but got directory",
                ));
            }
            outfile.to_path_buf()
        } else {
            pdir.join(conf.name.clone() + ".wikit")
        };

        let mut writer = BufWriter::new(File::create(&outfile)?);
        // magic
        writer.write(WIKIT_MAGIC.as_bytes())?;
        writer.write(&LATEST_WIKIT_FMT_VERSION.to_be_bytes()[..])?;
        // hdrsz
        let hdrsz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(4))?;
        // namesz and name
        let namesz = conf.name.len() as u16;
        writer.write(&namesz.to_be_bytes()[..])?;
        writer.write(&conf.name.as_bytes()[..])?;
        // descsz and desc
        let descsz = conf.description.len() as u16;
        writer.write(&descsz.to_be_bytes()[..])?;
        writer.write(&conf.description.as_bytes()[..])?;
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
        // scriptsz and script
        let scriptsz = script.len() as u32;
        writer.write(&scriptsz.to_be_bytes()[..])?;
        writer.write(&script.as_bytes()[..])?;
        // stylesz and style
        let stylesz = style.len() as u32;
        writer.write(&stylesz.to_be_bytes()[..])?;
        writer.write(&style.as_bytes()[..])?;

        // save header size
        let hdrsz = writer.seek(SeekFrom::Current(0))? as u32;
        writer.seek(SeekFrom::Start(hdrsz_pos))?;
        writer.write(&hdrsz.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(hdrsz as u64))?;

        let srcfile_path_str = &format!("{}", srcfile.display());
        let mut word_meaning_list = match suffix.to_lowercase().as_str() {
            "mdx" => mdict::parse_mdx(srcfile_path_str, None)?.entries,
            "txt" => {
                let f = File::open(srcfile_path_str)
                    .context(elog!("failed to open {}", srcfile_path_str))?;
                reader::MDXSource::new(f).collect::<Vec<(String, String)>>()
            }
            _ => {
                return Err(WikitError::new(format!(
                    "source type {} is not supported",
                    srcfile.display()
                )));
            }
        };
        // sort word by ascending
        word_meaning_list.sort_by(|a, b| a.0.cmp(&b.0));
        // remove duplicate word
        word_meaning_list.dedup_by(|a, b| a.0.eq(&b.0));

        let dstart = writer.seek(SeekFrom::Current(0))?;
        let mut index_table = vec![];
        for (word, meaning) in word_meaning_list.iter() {
            let entry =
                DataEntry::new(DataEntryType::TXT, meaning.len() as u32, meaning.as_bytes());
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

        writer.flush()?;
        Ok(outfile)
    }

    pub fn load<P>(path: P) -> WikitResult<Self>
    where
        P: AsRef<Path>,
    {
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

        let mut hdrsz = [0u8; 4];
        file.read_exact(&mut hdrsz)?;
        let hdrsz = u32::from_be_bytes(hdrsz) as usize;

        let prefix = WIKIT_MAGIC.len() + 8;
        if hdrsz < prefix || hdrsz as u64 > file.metadata()?.len() {
            return Err(WikitError::new("Invalid Wikit header length"));
        }
        let mut hdrbuf = vec![0; hdrsz - prefix];
        file.read_exact(&mut hdrbuf)?;
        if hdrbuf.len() != hdrsz - prefix {
            return Err(WikitError::new("Wikit header is broken"));
        }
        let wikit_head = WikitHead::new(&hdrbuf[..])?;

        Ok(LocalDictionary {
            head: wikit_head.clone(),
            path: path.to_path_buf(),
            idx: index::FSTIndex::new(path.to_path_buf(), wikit_head.ibase, wikit_head.isz),
        })
    }

    pub fn lookup<P>(&self, word: P) -> WikitResult<Vec<(String, String)>>
    where
        P: AsRef<str>,
    {
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
                let meaning_buf = file
                    .bytes()
                    .take(meaning_size)
                    .filter_map(Result::ok)
                    .collect::<Vec<u8>>();
                if meaning_buf.len() == meaning_size {
                    anslist.push((word.to_string(), String::from_utf8(meaning_buf)?));
                }
            }
            return Ok(anslist);
        }
        return Err(WikitError::new("No such word or similar words"));
    }

    pub fn get_script(&self) -> &str {
        &self.head.script
    }

    pub fn get_style(&self) -> &str {
        &self.head.style
    }
}

pub fn load_dictionary_from_uri<S>(uri: S) -> Option<WikitDictionary>
where
    S: AsRef<str>,
{
    let uri = uri.as_ref();
    if let Ok(url) = url::Url::parse(uri) {
        match url.scheme() {
            "file" => {
                if let Ok(dictpath) = url.to_file_path() {
                    if let Ok(dict) = LocalDictionary::load(dictpath) {
                        return Some(WikitDictionary::Local(dict));
                    }
                }
            }
            "http" | "https" => {
                let host = if let Some(host) = url.host_str() {
                    host
                } else {
                    return None;
                };
                let port = if let Some(port) = url.port() {
                    format!(":{}", port)
                } else {
                    "".to_string()
                };
                let user = url.username();
                let token = if let Some(token) = url.password() {
                    token
                } else {
                    ""
                };
                let dict = RemoteDictionary::new(
                    format!("{}://{}{}{}", url.scheme(), host, port, url.path()),
                    user.to_string(),
                    token.to_string(),
                );
                return Some(WikitDictionary::Remote(dict));
            }
            _ => {
                return None;
            }
        }
    }
    return None;
}

pub fn load_server_dictionary() -> WikitResult<Vec<WikitDictionary>> {
    let mut dicts = vec![];
    for uri in config::load_config()?.srvcfg.uris.iter() {
        if let Some(dict) = load_dictionary_from_uri(uri) {
            dicts.push(dict);
        }
    }
    Ok(dicts)
}

pub fn load_client_dictionary() -> WikitResult<Vec<WikitDictionary>> {
    let mut dicts = vec![];
    for uri in config::load_config()?.cltcfg.uris.iter() {
        if let Some(dict) = load_dictionary_from_uri(uri) {
            dicts.push(dict);
        }
    }
    Ok(dicts)
}
