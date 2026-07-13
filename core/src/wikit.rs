/// Create or load wikit dictionary

use crate::error::{WikitError, Context, WikitResult, AnyResult, NomResult};
use crate::elog;
use crate::index;
use crate::mdict;
use crate::util;
use crate::reader;
use crate::config;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{BufWriter, Write, Seek, SeekFrom, Read};

use regex::Regex;
use serde::{Deserialize, Serialize};
use nom::{do_parse, map_res, take};
use nom::number::streaming::{be_u16, be_u32, be_u64};
use fst::{IntoStreamer, Streamer};
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
pub struct WikitDictProfile {
    name: String,
    version: String,
    authors: Vec<String>,
    distributors: Vec<String>,
    description: String,
    homepage: String,
    css: String,
    js: String,
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

fn normalize_asset_filename(path: &str) -> String {
    path.trim()
        .trim_start_matches(|c| c == '/' || c == '\\')
        .replace('\\', "/")
        .split('/')
        .last()
        .unwrap_or(path)
        .to_string()
}

fn read_sibling_text_file(dir: &Path, filename: &str) -> Option<String> {
    let path = dir.join(filename);
    let mut content = String::new();
    File::open(path).ok()?.read_to_string(&mut content).ok()?;
    if content.trim().is_empty() {
        None
    } else {
        Some(content)
    }
}

/// Collect CSS/JS from MDX entries and sibling files referenced by entry HTML.
fn extract_dictionary_assets(
    dir: &Path,
    stem: &str,
    entries: &[(String, String)],
) -> (String, String, HashSet<String>) {
    let mut style = String::new();
    let mut script = String::new();
    let mut resource_keys = HashSet::new();
    let mut css_files = HashSet::new();
    let mut js_files = HashSet::new();
    let mut html_samples = 0usize;

    let css_link_re = Regex::new(r#"(?i)<link[^>]+href\s*=\s*["']([^"']+\.css)["']"#).ok();
    let script_src_re = Regex::new(r#"(?i)<script[^>]+src\s*=\s*["']([^"']+\.js)["']"#).ok();

    for (word, meaning) in entries {
        let key = word.trim();
        let file_name = normalize_asset_filename(key);
        let file_lower = file_name.to_lowercase();

        if file_lower.ends_with(".css") {
            if !meaning.trim().is_empty() {
                if !style.is_empty() {
                    style.push('\n');
                }
                style.push_str(meaning);
            }
            resource_keys.insert(word.clone());
            css_files.insert(file_name);
            continue;
        }
        if file_lower.ends_with(".js") {
            if !meaning.trim().is_empty() {
                if !script.is_empty() {
                    script.push('\n');
                }
                script.push_str(meaning);
            }
            resource_keys.insert(word.clone());
            js_files.insert(file_name);
            continue;
        }

        // Sample a few HTML entries for linked asset filenames.
        if html_samples < 32
            && (meaning.contains(".css") || meaning.contains(".js") || meaning.contains("stylesheet"))
        {
            html_samples += 1;
            if let Some(re) = css_link_re.as_ref() {
                for cap in re.captures_iter(meaning) {
                    css_files.insert(normalize_asset_filename(&cap[1]));
                }
            }
            if let Some(re) = script_src_re.as_ref() {
                for cap in re.captures_iter(meaning) {
                    js_files.insert(normalize_asset_filename(&cap[1]));
                }
            }
        }
    }

    // Preferred local names: explicit HTML references, then dictionary stem.
    if css_files.is_empty() {
        css_files.insert(format!("{stem}.css"));
    }
    if js_files.is_empty() {
        js_files.insert(format!("{stem}.js"));
    }

    if style.trim().is_empty() {
        for name in &css_files {
            if let Some(content) = read_sibling_text_file(dir, name) {
                if !style.is_empty() {
                    style.push('\n');
                }
                style.push_str(&content);
            }
        }
    }
    if script.trim().is_empty() {
        for name in &js_files {
            if let Some(content) = read_sibling_text_file(dir, name) {
                if !script.is_empty() {
                    script.push('\n');
                }
                script.push_str(&content);
            }
        }
    }

    (style, script, resource_keys)
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
            Err(e) => {
                Err(WikitError::new(format!("failed to parse WikitHead: {:?}", e)))
            }
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

    pub fn get_dict_list(&self) -> WikitResult<Vec<DictMeta>> {
        let url = format!("{}/wikit/list", self.url);
        let r = reqwest::blocking::get(url)?.json::<Vec<DictMeta>>()?;
        Ok(r)
    }

    pub fn lookup<P>(&self, word: P, dict: P) -> WikitResult<Vec<(String, String)>> where P: AsRef<str> {
        let r = reqwest::blocking::get(
            format!("{}/wikit/query?word={}&dictname={}", self.url, word.as_ref(), dict.as_ref())
        )?.json::<Vec<(String, String)>>()?;
        Ok(r)
    }

    pub fn get_script<S>(&self, dict: S) -> String where S: AsRef<str> {
        if let Ok(r) = reqwest::blocking::get(format!("{}/wikit/script?dictname={}", self.url, dict.as_ref())) {
            if let Ok(r) = r.text() {
                return r;
            }
        }
        "".to_string()
    }

    pub fn get_style<S>(&self, dict: S) -> String where S: AsRef<str> {
        if let Ok(r) = reqwest::blocking::get(format!("{}/wikit/style?dictname={}", self.url, dict.as_ref())) {
            if let Ok(r) = r.text() {
                return r;
            }
        }
        "".to_string()
    }
}

/// LocalDictionary represents a wikit dictionary.
///
/// wikit dictionary starts_with with `magic` and `version` fields
///
///      magic:8
///      version:4
///
/// if version is 0x01, then the follwoing layout is
///
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
        Q: AsRef<Path>
    {
        Self::create_with_progress(srcfile, outfile, |_| {})
    }

    pub fn create_with_progress<P, Q, F>(srcfile: P, outfile: Option<Q>, mut progress: F) -> WikitResult<PathBuf>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
        F: FnMut(f64),
    {
        let srcfile = srcfile.as_ref();
        progress(0.02);
        let (pdir, stem, suffix) = util::parse_path(srcfile)
            .context(elog!("failed to get parent directory of {}", srcfile.display()))?;

        let mut conf = String::new();
        if let Ok(mut f) = File::open(pdir.join(stem.clone() + ".toml")) {
            f.read_to_string(&mut conf)?;
        };
        let conf = toml::from_str::<WikitDictProfile>(&conf).unwrap_or({
            let mut def = WikitDictProfile::default();
            def.name = stem.as_str().to_owned();
            def.description = "this dictionary has no description".to_owned();
            def.authors = vec!["anonymous".to_owned()];
            def
        });

        let read_include_file = |maybe_path: &str| -> String {
            let mut content = String::new();
            if maybe_path.trim().starts_with('@') {
                // TODO: avoid path traversal?
                if let Ok(mut f) = File::open(pdir.join(&maybe_path.trim()[1..])) {
                    _ = f.read_to_string(&mut content);
                }
            } else {
                if maybe_path.trim().len() > 0 {
                    content = maybe_path.to_owned();
                }
            }
            content
        };
        let mut style = read_include_file(conf.css.trim());
        let mut script = read_include_file(conf.js.trim());

        let outfile = if let Some(outfile) = outfile {
            let outfile = outfile.as_ref();
            if outfile.exists() && outfile.is_dir() {
                return Err(WikitError::new("output path must be a file but got directory"));
            }
            outfile.to_path_buf()
        } else {
            pdir.join(conf.name.clone() + ".wikit")
        };

        let srcfile_path_str = &format!("{}", srcfile.display());
        let mut word_meaning_list = match suffix.to_lowercase().as_str() {
            "mdx" => {
                mdict::parse_mdx_with_progress(srcfile_path_str, None, |pct| {
                    progress(0.05 + pct * 0.70);
                })?.entries
            },
            "txt" => {
                let f = File::open(srcfile_path_str).context(elog!("failed to open {}", srcfile_path_str))?;
                let entries = reader::MDXSource::new(f).collect::<Vec<(String, String)>>();
                progress(0.75);
                entries
            }
            _ => {
                return Err(WikitError::new(format!("source type {} is not supported", srcfile.display())));
            }
        };

        // Prefer CSS/JS bundled as dictionary entries or sibling files referenced by HTML.
        let (embedded_style, embedded_script, resource_keys) =
            extract_dictionary_assets(&pdir, &stem, &word_meaning_list);
        if style.trim().is_empty() {
            style = embedded_style;
        } else if !embedded_style.is_empty() {
            style.push('\n');
            style.push_str(&embedded_style);
        }
        if script.trim().is_empty() {
            script = embedded_script;
        } else if !embedded_script.is_empty() {
            script.push('\n');
            script.push_str(&embedded_script);
        }
        if !resource_keys.is_empty() {
            word_meaning_list.retain(|(word, _)| !resource_keys.contains(word));
        }

        // sort word by ascending
        word_meaning_list.sort_by(|a, b| a.0.cmp(&b.0));
        // remove duplicate word
        word_meaning_list.dedup_by(|a, b| a.0.eq(&b.0));
        progress(0.78);

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
        progress(0.80);

        let dstart = writer.seek(SeekFrom::Current(0))?;
        let mut index_table = vec![];
        let total_entries = word_meaning_list.len();
        let write_progress_step = std::cmp::max(1, total_entries / 1000);
        for (idx, (word, meaning)) in word_meaning_list.iter().enumerate() {
            let entry = DataEntry::new(DataEntryType::TXT, meaning.len() as u32, meaning.as_bytes());
            let (offset, _count) = entry.write(&mut writer)?;
            index_table.push((word, offset));
            if idx % write_progress_step == 0 || idx + 1 == total_entries {
                let ratio = if total_entries == 0 { 1.0 } else { (idx + 1) as f64 / total_entries as f64 };
                progress(0.80 + ratio * 0.14);
            }
        }
        let dend = writer.seek(SeekFrom::Current(0))?;

        let dbase = dstart as u64;
        writer.seek(SeekFrom::Start(dbase_pos))?;
        writer.write(&dbase.to_be_bytes()[..])?;
        let dsz = (dend - dstart) as u64;
        writer.seek(SeekFrom::Start(dsz_pos))?;
        writer.write(&dsz.to_be_bytes()[..])?;

        writer.seek(SeekFrom::Start(dend))?;
        progress(0.95);
        let (ibase, isz) = index::FSTIndex::write(&mut index_table.iter(), &mut writer)?;
        let (ibase, isz) = (ibase as u64, isz as u64);
        writer.seek(SeekFrom::Start(ibase_pos))?;
        writer.write(&ibase.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(isz_pos))?;
        writer.write(&isz.to_be_bytes()[..])?;
        progress(0.99);

        Ok(outfile)
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

        let mut hdrsz = [0u8; 4];
        file.read_exact(&mut hdrsz)?;
        let hdrsz = u32::from_be_bytes(hdrsz) as usize;

        let hdrbuf = file.bytes().take(hdrsz).filter_map(Result::ok).collect::<Vec<u8>>();
        if hdrbuf.len() != hdrsz {
            return Err(WikitError::new("Wikit header is broken"));
        }
        let wikit_head = WikitHead::new(&hdrbuf[..])?;

        Ok(LocalDictionary {
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

    pub fn get_script(&self) -> &str {
        &self.head.script
    }

    pub fn get_style(&self) -> &str {
        &self.head.style
    }

    /// Rewrite a wikit dictionary with new CSS/JS (and optional name/description),
    /// keeping entries and adjusting index offsets.
    pub fn republish_with_assets<P, Q>(
        src: P,
        dest: Q,
        style: &str,
        script: &str,
        name: Option<&str>,
        desc: Option<&str>,
    ) -> WikitResult<PathBuf>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let src = src.as_ref();
        let dest = dest.as_ref();
        let dict = Self::load(src)?;
        let head = &dict.head;
        let name = name.unwrap_or(head.name.as_str());
        let desc = desc.unwrap_or(head.desc.as_str());

        if head.ibase < head.dbase {
            return Err(WikitError::new("invalid wikit layout: ibase < dbase"));
        }

        let data_len = (head.ibase - head.dbase) as usize;
        let mut src_file = File::open(src)?;
        src_file.seek(SeekFrom::Start(head.dbase))?;
        let mut data = vec![0u8; data_len];
        src_file.read_exact(&mut data)?;

        let mut index_table: Vec<(String, u64)> = Vec::new();
        {
            let mmap = unsafe {
                memmap::MmapOptions::new()
                    .offset(head.ibase)
                    .len(head.isz as usize)
                    .map(&src_file)?
            };
            let map = fst::Map::new(mmap)?;
            let mut stream = map.stream();
            while let Some((key, value)) = stream.next() {
                let word = String::from_utf8(key.to_vec())
                    .context(elog!("invalid utf8 keyword in dictionary index"))?;
                index_table.push((word, value));
            }
        }
        drop(src_file);

        let write_path = if src == dest {
            dest.with_extension("wikit.republish-tmp")
        } else {
            dest.to_path_buf()
        };

        let mut writer = BufWriter::new(File::create(&write_path)?);
        writer.write(WIKIT_MAGIC.as_bytes())?;
        writer.write(&LATEST_WIKIT_FMT_VERSION.to_be_bytes()[..])?;

        let hdrsz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(4))?;

        let namesz = name.len() as u16;
        writer.write(&namesz.to_be_bytes()[..])?;
        writer.write(name.as_bytes())?;
        let descsz = desc.len() as u16;
        writer.write(&descsz.to_be_bytes()[..])?;
        writer.write(desc.as_bytes())?;
        writer.write(&[index::IndexFormat::FST as u8])?;

        let ibase_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;
        let isz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;
        let dbase_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;
        let dsz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;

        let scriptsz = script.len() as u32;
        writer.write(&scriptsz.to_be_bytes()[..])?;
        writer.write(script.as_bytes())?;
        let stylesz = style.len() as u32;
        writer.write(&stylesz.to_be_bytes()[..])?;
        writer.write(style.as_bytes())?;

        let hdrsz = writer.seek(SeekFrom::Current(0))? as u32;
        writer.seek(SeekFrom::Start(hdrsz_pos))?;
        writer.write(&hdrsz.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(hdrsz as u64))?;

        let dstart = writer.seek(SeekFrom::Current(0))?;
        let delta = dstart as i64 - head.dbase as i64;
        writer.write_all(&data)?;
        let dend = writer.seek(SeekFrom::Current(0))?;

        let dbase = dstart as u64;
        writer.seek(SeekFrom::Start(dbase_pos))?;
        writer.write(&dbase.to_be_bytes()[..])?;
        let dsz = (dend - dstart) as u64;
        writer.seek(SeekFrom::Start(dsz_pos))?;
        writer.write(&dsz.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(dend))?;

        let adjusted: Vec<(String, u64)> = index_table
            .into_iter()
            .map(|(word, offset)| (word, (offset as i64 + delta) as u64))
            .collect();
        let (ibase, isz) = index::FSTIndex::write(&mut adjusted.iter(), &mut writer)?;
        let (ibase, isz) = (ibase as u64, isz as u64);
        writer.seek(SeekFrom::Start(ibase_pos))?;
        writer.write(&ibase.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(isz_pos))?;
        writer.write(&isz.to_be_bytes()[..])?;
        writer.flush()?;
        drop(writer);

        if write_path != dest {
            fs::rename(&write_path, dest).context(elog!(
                "failed to replace dictionary file {}",
                dest.display()
            ))?;
        }

        Ok(dest.to_path_buf())
    }
}

pub fn load_dictionary_from_uri<S>(uri: S) -> Option<WikitDictionary> where S: AsRef<str> {
    let uri = uri.as_ref();
    if let Ok(url) = url::Url::parse(uri) {
        match url.scheme() {
            "file" => {
                if let Ok(dictpath) = url.to_file_path() {
                    if let Ok(dict) = LocalDictionary::load(dictpath) {
                        return Some(WikitDictionary::Local(dict));
                    }
                }
            },
            "http" | "https" => {
                let host = if let Some(host) = url.host_str() {
                    host
                } else {
                    return None;
                };
                let port = if let Some(port) = url.port() {
                    format!(":{}", port)
                }else {
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
            },
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

/// Register a local dictionary path for the desktop client, replacing any
/// previously registered dictionaries that share the same display name.
pub fn register_local_dictionary(path: &Path) -> WikitResult<()> {
    let local = LocalDictionary::load(path)?;
    let uri = config::path_to_file_uri(path);
    let cfg = config::load_config()?;
    let mut replace_uris = Vec::new();

    for existing in cfg.cltcfg.uris.iter() {
        if existing == &uri {
            continue;
        }
        if let Some(WikitDictionary::Local(ld)) = load_dictionary_from_uri(existing) {
            if ld.head.name == local.head.name {
                replace_uris.push(existing.clone());
            }
        }
    }

    config::register_client_dictionary_uri_replacing(path, &replace_uris)
        .context(elog!("failed to register dictionary {}", path.display()))?;
    Ok(())
}

/// Unregister a dictionary from the desktop client config by id (usually local file path).
pub fn unregister_local_dictionary(dictid: &str) -> WikitResult<bool> {
    let removed = config::unregister_client_dictionary_uri(dictid)
        .context(elog!("failed to unregister dictionary {}", dictid))?;
    if removed {
        return Ok(true);
    }

    // Fall back: match by loaded local dictionary path / name identity.
    let mut cfg = config::load_config()?;
    let before = cfg.cltcfg.uris.len();
    cfg.cltcfg.uris.retain(|uri| {
        match load_dictionary_from_uri(uri) {
            Some(WikitDictionary::Local(ld)) => {
                ld.path.display().to_string() != dictid
            }
            _ => true,
        }
    });
    let removed = cfg.cltcfg.uris.len() != before;
    if removed {
        config::save_config(&cfg).context(elog!("failed to save config after unregister"))?;
    }
    Ok(removed)
}

#[cfg(test)]
mod asset_tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn extracts_sibling_css_referenced_by_html() {
        let dir = tempdir().unwrap();
        let css_path = dir.path().join("oald10.css");
        fs::write(&css_path, ".headword{color:red}").unwrap();
        let entries = vec![(
            "interject".to_string(),
            r#"<link rel="stylesheet" href="oald10.css"><h1 class="headword">interject</h1>"#.to_string(),
        )];
        let (style, _script, _keys) = extract_dictionary_assets(dir.path(), "dict", &entries);
        assert!(style.contains(".headword{color:red}"), "style={style}");
    }
}
