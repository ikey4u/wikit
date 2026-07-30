use crate::config;
use crate::elog;
/// Create or load wikit dictionary
use crate::error::{AnyResult, Context, NomResult, WikitError, WikitResult};
use crate::fulltext;
use crate::index;
use crate::mdict;
use crate::reader;
use crate::util;
use crate::wikit_block;
use crate::zstdutil;

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use fst::{IntoStreamer, Streamer};
use nom::combinator::rest;
use nom::number::streaming::{be_u16, be_u32, be_u64};
use nom::{do_parse, map_res, take};
use regex::Regex;
use serde::{Deserialize, Serialize};
use wikit_proto::DictMeta;

// `516` is the birthday of wikit project (the first commit date 2021-05-16)
const WIKIT_MAGIC: &'static str = "WIKIT516";
/// Legacy uncompressed entry layout.
const WIKIT_FMT_V1: u32 = 0x00_00_00_01;
/// Entry payloads may be zstd-compressed (see ENTRY_CODEC_*).
const WIKIT_FMT_V2: u32 = 0x00_00_00_02;
/// Block-level zstd compression with logical-offset FST.
const WIKIT_FMT_V3: u32 = 0x00_00_00_03;
/// Version written by current builders.
const LATEST_WIKIT_FMT_VERSION: u32 = WIKIT_FMT_V3;

const ENTRY_CODEC_RAW: u8 = 0;
const ENTRY_CODEC_ZSTD: u8 = 1;

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum DataEntryType {
    TXT = 0x1,
    SVG = 0x2,
    PNG = 0x3,
    JPG = 0x4,
    MP3 = 0x5,
    WAV = 0x6,
    MP4 = 0x7,
    /// Generic binary resource (fallback).
    BIN = 0x8,
}

impl DataEntryType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x1 => Some(Self::TXT),
            0x2 => Some(Self::SVG),
            0x3 => Some(Self::PNG),
            0x4 => Some(Self::JPG),
            0x5 => Some(Self::MP3),
            0x6 => Some(Self::WAV),
            0x7 => Some(Self::MP4),
            0x8 => Some(Self::BIN),
            _ => None,
        }
    }

    pub fn from_resource_key(key: &str) -> Self {
        let lower = key.to_ascii_lowercase();
        if lower.ends_with(".svg") {
            Self::SVG
        } else if lower.ends_with(".png") {
            Self::PNG
        } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            Self::JPG
        } else if lower.ends_with(".mp3") {
            Self::MP3
        } else if lower.ends_with(".wav") {
            Self::WAV
        } else if lower.ends_with(".mp4") {
            Self::MP4
        } else {
            Self::BIN
        }
    }

    pub fn is_text(self) -> bool {
        matches!(self, Self::TXT)
    }
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

    /// Write a v1 (uncompressed) data entry: typ + sz + raw bytes.
    pub fn write_v1<W>(&self, writer: &mut W) -> WikitResult<(u64, u64)>
    where
        W: std::io::Write + std::io::Seek,
    {
        let start = writer.seek(SeekFrom::Current(0))?;
        writer.write_all(&[self.typ as u8])?;
        writer.write_all(&self.sz.to_be_bytes()[..])?;
        writer.write_all(self.buf)?;
        let end = writer.seek(SeekFrom::Current(0))?;
        Ok((start, end - start))
    }

    /// Write a v2 data entry: typ + codec + raw_sz + store_sz + payload.
    /// Payload is zstd-compressed when that shrinks the entry; otherwise raw.
    pub fn write_v2<W>(&self, writer: &mut W) -> WikitResult<(u64, u64)>
    where
        W: std::io::Write + std::io::Seek,
    {
        let start = writer.seek(SeekFrom::Current(0))?;
        writer.write_all(&[self.typ as u8])?;

        let compressed = zstdutil::compress(self.buf, zstdutil::DEFAULT_ENTRY_LEVEL)?;
        let use_zstd = !self.buf.is_empty() && compressed.len() < self.buf.len();
        let (codec, payload): (u8, &[u8]) = if use_zstd {
            (ENTRY_CODEC_ZSTD, compressed.as_slice())
        } else {
            (ENTRY_CODEC_RAW, self.buf)
        };

        writer.write_all(&[codec])?;
        writer.write_all(&self.sz.to_be_bytes()[..])?;
        let store_sz = payload.len() as u32;
        writer.write_all(&store_sz.to_be_bytes()[..])?;
        writer.write_all(payload)?;
        let end = writer.seek(SeekFrom::Current(0))?;
        Ok((start, end - start))
    }

    pub fn write<W>(&self, writer: &mut W, fmt_version: u32) -> WikitResult<(u64, u64)>
    where
        W: std::io::Write + std::io::Seek,
    {
        if fmt_version >= WIKIT_FMT_V2 {
            self.write_v2(writer)
        } else {
            self.write_v1(writer)
        }
    }
}

fn supported_wikit_version(version: u32) -> bool {
    version == WIKIT_FMT_V1 || version == WIKIT_FMT_V2 || version == WIKIT_FMT_V3
}

fn read_entry_payload<R: Read + Seek>(
    file: &mut R,
    offset: u64,
    fmt_version: u32,
) -> WikitResult<Vec<u8>> {
    file.seek(SeekFrom::Start(offset))?;
    let mut typ = [0u8; 1];
    file.read_exact(&mut typ)?;
    let _ = typ; // reserved for future media types

    if fmt_version <= WIKIT_FMT_V1 {
        let mut meaning_size = [0u8; 4];
        file.read_exact(&mut meaning_size)?;
        let meaning_size = u32::from_be_bytes(meaning_size) as usize;
        let mut meaning_buf = vec![0u8; meaning_size];
        file.read_exact(&mut meaning_buf)?;
        return Ok(meaning_buf);
    }

    let mut codec = [0u8; 1];
    file.read_exact(&mut codec)?;
    let mut raw_sz = [0u8; 4];
    file.read_exact(&mut raw_sz)?;
    let raw_sz = u32::from_be_bytes(raw_sz) as usize;
    let mut store_sz = [0u8; 4];
    file.read_exact(&mut store_sz)?;
    let store_sz = u32::from_be_bytes(store_sz) as usize;
    let mut payload = vec![0u8; store_sz];
    file.read_exact(&mut payload)?;

    match codec[0] {
        ENTRY_CODEC_RAW => {
            if payload.len() != raw_sz {
                return Err(WikitError::new(format!(
                    "raw entry size mismatch: header {raw_sz}, payload {}",
                    payload.len()
                )));
            }
            Ok(payload)
        }
        ENTRY_CODEC_ZSTD => zstdutil::decompress(&payload, raw_sz),
        other => Err(WikitError::new(format!("unknown entry codec: {other}"))),
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
            && (meaning.contains(".css")
                || meaning.contains(".js")
                || meaning.contains("stylesheet"))
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
    /// v3 block table (empty for v1/v2).
    pub blocks: Vec<wikit_block::DataBlockInfo>,
    /// Embedded fulltext archive offset (0 = none).
    pub fbase: u64,
    /// Embedded fulltext archive size (0 = none).
    pub fsz: u64,
}

impl WikitHead {
    /// Parse header body (bytes after the `hdrsz` field). For v3, trailing bytes are the block table.
    pub fn new(headbuf: &[u8], fmt_version: u32) -> WikitResult<Self> {
        let r: NomResult<(WikitHead, &[u8])> = do_parse!(headbuf,
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
            remain: rest >>
            (
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
                        blocks: Vec::new(),
                        fbase: 0,
                        fsz: 0,
                    },
                    remain
                )
            )
        );

        match r {
            Ok((_, (mut head, remain))) => {
                if fmt_version >= WIKIT_FMT_V3 {
                    let (blocks, rest) = wikit_block::parse_block_table(remain)?;
                    head.blocks = blocks;
                    if rest.len() >= 16 {
                        head.fbase = u64::from_be_bytes(rest[0..8].try_into().unwrap());
                        head.fsz = u64::from_be_bytes(rest[8..16].try_into().unwrap());
                    }
                }
                Ok(head)
            }
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

    pub fn get_dict_list(&self) -> WikitResult<Vec<DictMeta>> {
        let url = format!("{}/wikit/list", self.url);
        let r = reqwest::blocking::get(url)?.json::<Vec<DictMeta>>()?;
        Ok(r)
    }

    pub fn lookup<P>(&self, word: P, dict: P) -> WikitResult<Vec<(String, String)>>
    where
        P: AsRef<str>,
    {
        let r = reqwest::blocking::get(format!(
            "{}/wikit/query?word={}&dictname={}",
            self.url,
            word.as_ref(),
            dict.as_ref()
        ))?
        .json::<Vec<(String, String)>>()?;
        Ok(r)
    }

    pub fn get_script<S>(&self, dict: S) -> String
    where
        S: AsRef<str>,
    {
        if let Ok(r) = reqwest::blocking::get(format!(
            "{}/wikit/script?dictname={}",
            self.url,
            dict.as_ref()
        )) {
            if let Ok(r) = r.text() {
                return r;
            }
        }
        "".to_string()
    }

    pub fn get_style<S>(&self, dict: S) -> String
    where
        S: AsRef<str>,
    {
        if let Ok(r) = reqwest::blocking::get(format!(
            "{}/wikit/style?dictname={}",
            self.url,
            dict.as_ref()
        )) {
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
/// if version is 0x01 (v1), then the following layout is
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
///      data: dsz   (entries: typ:1 + sz:4 + raw:sz)
///      index: isz
///
/// if version is 0x02 (v2), header layout is unchanged; each data entry is
///      typ:1 + codec:1 + raw_sz:4 + store_sz:4 + payload:store_sz
/// where codec 0 = raw payload, codec 1 = zstd(payload) -> raw_sz bytes.
///
/// if version is 0x03 (v3), header appends a block table after style:
///      bcount:u32
///      repeat bcount: ulog_off:u64, ulog_sz:u32, c_off:u64, c_sz:u32
/// data region holds concatenated zstd frames; FST values are logical offsets
/// into the uncompressed entry stream (typ:1 + sz:4 + payload).
#[derive(Clone, Deserialize, Serialize)]
pub struct LocalDictionary {
    pub head: WikitHead,
    // local path of dictionary
    pub path: PathBuf,
    /// File format version (`WIKIT_FMT_V1` / `V2` / `V3`).
    pub fmt_version: u32,
    idx: index::FSTIndex,
    #[serde(skip)]
    block_cache: Arc<Mutex<wikit_block::BlockCache>>,
    #[serde(skip)]
    fulltext: Option<Arc<fulltext::FulltextIndex>>,
}

impl std::fmt::Debug for LocalDictionary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalDictionary")
            .field("head", &self.head)
            .field("path", &self.path)
            .field("fmt_version", &self.fmt_version)
            .field("has_fulltext", &self.fulltext.is_some())
            .finish()
    }
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
        Self::create_with_progress(srcfile, outfile, |_| {})
    }

    pub fn create_with_progress<P, Q, F>(
        srcfile: P,
        outfile: Option<Q>,
        mut progress: F,
    ) -> WikitResult<PathBuf>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
        F: FnMut(f64),
    {
        let srcfile = srcfile.as_ref();
        progress(0.02);
        let (pdir, stem, suffix) = util::parse_path(srcfile).context(elog!(
            "failed to get parent directory of {}",
            srcfile.display()
        ))?;

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
                return Err(WikitError::new(
                    "output path must be a file but got directory",
                ));
            }
            outfile.to_path_buf()
        } else {
            pdir.join(conf.name.clone() + ".wikit")
        };

        let srcfile_path_str = &format!("{}", srcfile.display());
        let mut word_meaning_list = match suffix.to_lowercase().as_str() {
            "mdx" => {
                mdict::parse_mdx_with_progress(srcfile_path_str, None, |pct| {
                    progress(0.05 + pct * 0.55);
                })?
                .entries
            }
            "txt" => {
                let f = File::open(srcfile_path_str)
                    .context(elog!("failed to open {}", srcfile_path_str))?;
                let entries = reader::MDXSource::new(f).collect::<Vec<(String, String)>>();
                progress(0.60);
                entries
            }
            _ => {
                return Err(WikitError::new(format!(
                    "source type {} is not supported",
                    srcfile.display()
                )));
            }
        };

        // Prefer CSS/JS bundled as dictionary entries or sibling files referenced by HTML.
        let (embedded_style, embedded_script, css_js_keys) =
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
        // Only strip CSS/JS resource *keys* — never @@@LINK text entries.
        if !css_js_keys.is_empty() {
            word_meaning_list.retain(|(word, _)| !css_js_keys.contains(word));
        }

        let link_count = word_meaning_list
            .iter()
            .filter(|(_, body)| body.trim_start().starts_with("@@@LINK="))
            .count();
        log::info!("[+] Preserved {link_count} @@@LINK entries during conversion");

        // Optional companion MDD resources.
        let mut binary_entries: Vec<(String, Vec<u8>, DataEntryType)> = Vec::new();
        let mdd_path = pdir.join(format!("{stem}.mdd"));
        if mdd_path.exists() {
            progress(0.62);
            match mdict::parse_mdd_with_progress(&format!("{}", mdd_path.display()), |pct| {
                progress(0.62 + pct * 0.12);
            }) {
                Ok(mdd) => {
                    log::info!(
                        "[+] Loaded {} MDD resources from {}",
                        mdd.entries.len(),
                        mdd_path.display()
                    );
                    for (key, bytes) in mdd.entries {
                        let typ = DataEntryType::from_resource_key(&key);
                        // CSS/JS from MDD go to header if still empty / always skip duplicate keys in data when already extracted.
                        let fname = normalize_asset_filename(&key);
                        let lower = fname.to_ascii_lowercase();
                        if lower.ends_with(".css") || lower.ends_with(".js") {
                            continue;
                        }
                        binary_entries.push((key, bytes, typ));
                    }
                }
                Err(e) => {
                    log::warn!("[!] Failed to parse MDD {}: {:?}", mdd_path.display(), e);
                }
            }
        }

        // Build logical entry stream + FST keys (logical offsets).
        // Build logical entry stream + FST keys (logical offsets).
        let mut logical = Vec::new();
        let mut pending_keys: Vec<(String, u64)> = Vec::new();

        for (word, meaning) in word_meaning_list.iter() {
            let off = wikit_block::append_logical_entry(
                &mut logical,
                DataEntryType::TXT as u8,
                meaning.as_bytes(),
            );
            pending_keys.push((word.clone(), off));
        }
        for (key, bytes, typ) in &binary_entries {
            let off = wikit_block::append_logical_entry(&mut logical, *typ as u8, bytes);
            for alias in wikit_block::resource_key_aliases(key) {
                pending_keys.push((alias, off));
            }
        }
        pending_keys.sort_by(|a, b| a.0.cmp(&b.0));
        pending_keys.dedup_by(|a, b| a.0 == b.0);
        let index_table = pending_keys;
        progress(0.78);

        let (mut blocks, compressed_blob) = wikit_block::compress_logical_stream(&logical)?;
        log::info!(
            "[+] v3 blocks={} logical={} compressed={}",
            blocks.len(),
            logical.len(),
            compressed_blob.len()
        );

        let mut writer = BufWriter::new(File::create(&outfile)?);
        writer.write_all(WIKIT_MAGIC.as_bytes())?;
        writer.write_all(&LATEST_WIKIT_FMT_VERSION.to_be_bytes()[..])?;
        let hdrsz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(4))?;
        let namesz = conf.name.len() as u16;
        writer.write_all(&namesz.to_be_bytes()[..])?;
        writer.write_all(conf.name.as_bytes())?;
        let descsz = conf.description.len() as u16;
        writer.write_all(&descsz.to_be_bytes()[..])?;
        writer.write_all(conf.description.as_bytes())?;
        writer.write_all(&[index::IndexFormat::FST as u8])?;
        let ibase_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;
        let isz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;
        let dbase_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;
        let dsz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?;
        let scriptsz = script.len() as u32;
        writer.write_all(&scriptsz.to_be_bytes()[..])?;
        writer.write_all(script.as_bytes())?;
        let stylesz = style.len() as u32;
        writer.write_all(&stylesz.to_be_bytes()[..])?;
        writer.write_all(style.as_bytes())?;

        // Block table with relative c_off; rewrite after dbase is known.
        // Then fbase/fsz placeholders for embedded fulltext archive.
        let block_table_pos = writer.seek(SeekFrom::Current(0))?;
        let block_table_bytes = wikit_block::write_block_table(&blocks);
        writer.write_all(&block_table_bytes)?;
        let fbase_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?; // fbase
        let fsz_pos = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Current(8))?; // fsz

        let hdrsz = writer.seek(SeekFrom::Current(0))? as u32;
        writer.seek(SeekFrom::Start(hdrsz_pos))?;
        writer.write_all(&hdrsz.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(hdrsz as u64))?;
        progress(0.82);

        let dstart = writer.seek(SeekFrom::Current(0))?;
        for b in &mut blocks {
            b.c_off += dstart;
        }
        writer.seek(SeekFrom::Start(block_table_pos))?;
        writer.write_all(&wikit_block::write_block_table(&blocks))?;
        writer.seek(SeekFrom::Start(dstart))?;
        writer.write_all(&compressed_blob)?;
        let dend = writer.seek(SeekFrom::Current(0))?;

        writer.seek(SeekFrom::Start(dbase_pos))?;
        writer.write_all(&(dstart as u64).to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(dsz_pos))?;
        writer.write_all(&((dend - dstart) as u64).to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(dend))?;
        progress(0.90);

        let (ibase, isz) = index::FSTIndex::write(&mut index_table.iter(), &mut writer)?;
        writer.seek(SeekFrom::Start(ibase_pos))?;
        writer.write_all(&(ibase as u64).to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(isz_pos))?;
        writer.write_all(&(isz as u64).to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(ibase + isz))?;
        progress(0.93);

        // Build and append fulltext archive (Chinese + English body search).
        let ft_entries = fulltext::collect_index_entries(&word_meaning_list);
        log::info!(
            "[+] Building fulltext index for {} entries ...",
            ft_entries.len()
        );
        let ft_archive = fulltext::build_archive_with_progress(&ft_entries, |p| {
            progress(0.93 + p * 0.05);
        })?;
        let fstart = writer.seek(SeekFrom::Current(0))?;
        writer.write_all(&ft_archive)?;
        let fend = writer.seek(SeekFrom::Current(0))?;
        writer.seek(SeekFrom::Start(fbase_pos))?;
        writer.write_all(&(fstart as u64).to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(fsz_pos))?;
        writer.write_all(&((fend - fstart) as u64).to_be_bytes()[..])?;
        writer.flush()?;
        progress(0.99);
        log::info!(
            "[+] Fulltext archive {} bytes at offset {}",
            fend - fstart,
            fstart
        );

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
        if !supported_wikit_version(version) {
            return Err(WikitError::new(format!(
                "Unsupported wikit version: {version:#x} (supported: v1/v2/v3)"
            )));
        }

        // `hdrsz` is stored as the absolute file offset of the end of the header
        // (historical layout). Header body starts immediately after this field.
        let mut hdrsz_bytes = [0u8; 4];
        file.read_exact(&mut hdrsz_bytes)?;
        let header_end = u32::from_be_bytes(hdrsz_bytes) as u64;
        let header_body_start = file.seek(SeekFrom::Current(0))?;
        if header_end < header_body_start {
            return Err(WikitError::new("Wikit header size is invalid"));
        }
        let hdrsz = (header_end - header_body_start) as usize;
        let mut hdrbuf = vec![0u8; hdrsz];
        file.read_exact(&mut hdrbuf)?;
        let wikit_head = WikitHead::new(&hdrbuf[..], version)?;
        if version >= WIKIT_FMT_V3 && wikit_head.blocks.is_empty() {
            return Err(WikitError::new("v3 dictionary missing block table"));
        }

        let fulltext = match fulltext::open_from_wikit(path, wikit_head.fbase, wikit_head.fsz) {
            Ok(ft) => ft.map(Arc::new),
            Err(e) => {
                log::warn!("[!] fulltext index unavailable: {e}");
                None
            }
        };

        Ok(LocalDictionary {
            head: wikit_head.clone(),
            path: path.to_path_buf(),
            fmt_version: version,
            idx: index::FSTIndex::new(path.to_path_buf(), wikit_head.ibase, wikit_head.isz),
            block_cache: Arc::new(Mutex::new(wikit_block::BlockCache::new(8))),
            fulltext,
        })
    }

    fn read_logical_entry(&self, logical_off: u64) -> WikitResult<(u8, Vec<u8>)> {
        wikit_block::read_entry_at_logical_offset(
            &self.path,
            &self.head.blocks,
            logical_off,
            &self.block_cache,
        )
    }

    pub fn lookup<P>(&self, word: P) -> WikitResult<Vec<(String, String)>>
    where
        P: AsRef<str>,
    {
        if let Ok(poslist) = self.idx.lookup(word) {
            let mut anslist = vec![];
            if self.fmt_version >= WIKIT_FMT_V3 {
                for (word, offset) in poslist {
                    let (typ, payload) = self.read_logical_entry(offset)?;
                    if !DataEntryType::from_u8(typ)
                        .map(|t| t.is_text())
                        .unwrap_or(false)
                    {
                        continue;
                    }
                    anslist.push((word.to_string(), String::from_utf8(payload)?));
                }
            } else {
                let mut file = File::open(&self.path)?;
                for (word, offset) in poslist {
                    let meaning_buf = read_entry_payload(&mut file, offset, self.fmt_version)?;
                    anslist.push((word.to_string(), String::from_utf8(meaning_buf)?));
                }
            }
            if anslist.is_empty() {
                return Err(WikitError::new("No such word or similar words"));
            }
            return Ok(anslist);
        }
        Err(WikitError::new("No such word or similar words"))
    }

    /// Look up a non-text resource (audio/image/…) by key or basename alias.
    pub fn lookup_resource<P>(&self, key: P) -> WikitResult<(DataEntryType, Vec<u8>)>
    where
        P: AsRef<str>,
    {
        if self.fmt_version < WIKIT_FMT_V3 {
            return Err(WikitError::new("resource lookup requires wikit v3"));
        }
        let raw = key.as_ref().trim();
        let candidates = {
            let mut c = wikit_block::resource_key_aliases(raw);
            if !raw.is_empty() {
                c.insert(0, raw.to_string());
            }
            c.sort();
            c.dedup();
            c
        };
        for cand in candidates {
            if let Ok(poslist) = self.idx.lookup(&cand) {
                for (word, offset) in poslist {
                    if word != cand {
                        continue;
                    }
                    let (typ, payload) = self.read_logical_entry(offset)?;
                    if let Some(entry_typ) = DataEntryType::from_u8(typ) {
                        if !entry_typ.is_text() {
                            return Ok((entry_typ, payload));
                        }
                    }
                }
            }
        }
        Err(WikitError::new(format!("resource not found: {raw}")))
    }

    /// Full-text search over entry bodies (requires embedded index).
    pub fn search_fulltext<P>(
        &self,
        query: P,
        limit: usize,
    ) -> WikitResult<Vec<fulltext::FulltextHit>>
    where
        P: AsRef<str>,
    {
        match &self.fulltext {
            Some(ft) => ft.search(query.as_ref(), limit),
            None => Ok(Vec::new()),
        }
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
        // Preserve source format version so entry layouts stay valid when copying the data blob.
        writer.write(WIKIT_MAGIC.as_bytes())?;
        writer.write(&dict.fmt_version.to_be_bytes()[..])?;

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

        let is_v3 = dict.fmt_version >= WIKIT_FMT_V3;
        let (block_table_pos, fbase_pos, fsz_pos) = if is_v3 {
            let bt_pos = writer.seek(SeekFrom::Current(0))?;
            writer.write_all(&wikit_block::write_block_table(&head.blocks))?;
            let fb_pos = writer.seek(SeekFrom::Current(0))?;
            writer.seek(SeekFrom::Current(8))?;
            let fs_pos = writer.seek(SeekFrom::Current(0))?;
            writer.seek(SeekFrom::Current(8))?;
            (Some(bt_pos), Some(fb_pos), Some(fs_pos))
        } else {
            (None, None, None)
        };

        let hdrsz = writer.seek(SeekFrom::Current(0))? as u32;
        writer.seek(SeekFrom::Start(hdrsz_pos))?;
        writer.write(&hdrsz.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(hdrsz as u64))?;

        let dstart = writer.seek(SeekFrom::Current(0))?;
        let delta = dstart as i64 - head.dbase as i64;
        writer.write_all(&data)?;
        let dend = writer.seek(SeekFrom::Current(0))?;

        if let Some(bt_pos) = block_table_pos {
            let mut blocks = head.blocks.clone();
            for b in &mut blocks {
                b.c_off = (b.c_off as i64 + delta) as u64;
            }
            writer.seek(SeekFrom::Start(bt_pos))?;
            writer.write_all(&wikit_block::write_block_table(&blocks))?;
            writer.seek(SeekFrom::Start(dend))?;
        }

        let dbase = dstart as u64;
        writer.seek(SeekFrom::Start(dbase_pos))?;
        writer.write(&dbase.to_be_bytes()[..])?;
        let dsz = (dend - dstart) as u64;
        writer.seek(SeekFrom::Start(dsz_pos))?;
        writer.write(&dsz.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(dend))?;

        // v1/v2 FST stores file offsets → adjust by delta.
        // v3 FST stores logical offsets → keep unchanged.
        let adjusted: Vec<(String, u64)> = if is_v3 {
            index_table
        } else {
            index_table
                .into_iter()
                .map(|(word, offset)| (word, (offset as i64 + delta) as u64))
                .collect()
        };
        let (ibase, isz) = index::FSTIndex::write(&mut adjusted.iter(), &mut writer)?;
        let (ibase, isz) = (ibase as u64, isz as u64);
        writer.seek(SeekFrom::Start(ibase_pos))?;
        writer.write(&ibase.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(isz_pos))?;
        writer.write(&isz.to_be_bytes()[..])?;
        writer.seek(SeekFrom::Start(ibase + isz))?;

        // Copy embedded fulltext archive if present.
        let (new_fbase, new_fsz) = if head.fsz > 0 && head.fbase > 0 {
            let mut src_file = File::open(src)?;
            src_file.seek(SeekFrom::Start(head.fbase))?;
            let mut ft = vec![0u8; head.fsz as usize];
            src_file.read_exact(&mut ft)?;
            let fstart = writer.seek(SeekFrom::Current(0))?;
            writer.write_all(&ft)?;
            (fstart, head.fsz)
        } else {
            (0u64, 0u64)
        };
        if let (Some(fb), Some(fs)) = (fbase_pos, fsz_pos) {
            writer.seek(SeekFrom::Start(fb))?;
            writer.write_all(&new_fbase.to_be_bytes()[..])?;
            writer.seek(SeekFrom::Start(fs))?;
            writer.write_all(&new_fsz.to_be_bytes()[..])?;
        }

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
    cfg.cltcfg
        .uris
        .retain(|uri| match load_dictionary_from_uri(uri) {
            Some(WikitDictionary::Local(ld)) => ld.path.display().to_string() != dictid,
            _ => true,
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
            r#"<link rel="stylesheet" href="oald10.css"><h1 class="headword">interject</h1>"#
                .to_string(),
        )];
        let (style, _script, _keys) = extract_dictionary_assets(dir.path(), "dict", &entries);
        assert!(style.contains(".headword{color:red}"), "style={style}");
    }

    #[test]
    fn v3_zstd_dictionary_roundtrip() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("demo.txt");
        let body = "<div class=\"entry\">hello hello hello compression</div>\n".repeat(32);
        let content = format!("hello\n{body}</>\nworld\n<small>world world world</small>\n</>\n");
        fs::write(&src, content).unwrap();

        let out = LocalDictionary::create(&src, None::<&Path>).expect("create");
        let dict = LocalDictionary::load(&out).expect("load");
        assert_eq!(dict.fmt_version, WIKIT_FMT_V3);
        assert!(!dict.head.blocks.is_empty());

        let hits = dict.lookup("hello").expect("lookup");
        let exact = hits
            .iter()
            .find(|(k, _)| k == "hello")
            .expect("exact hello");
        assert!(exact.1.contains("compression"));
    }

    #[test]
    fn link_entries_preserved_in_v3() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("demo.txt");
        let content = "\
see also\n@@@LINK=target word\n</>\n\
target word\n<div>real definition of target</div>\n</>\n\
";
        fs::write(&src, content).unwrap();
        let out = LocalDictionary::create(&src, None::<&Path>).expect("create");
        let dict = LocalDictionary::load(&out).expect("load");
        let hits = dict.lookup("see also").expect("lookup link");
        let exact = hits.iter().find(|(k, _)| k == "see also").expect("exact");
        assert!(
            exact.1.trim_start().starts_with("@@@LINK="),
            "LINK body should be preserved, got: {}",
            exact.1
        );
    }

    #[test]
    fn packs_binary_resources_and_aliases() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("res.wikit");
        let mp3 = b"ID3fake-mp3-bytes-hello";
        {
            let mut logical = Vec::new();
            let text_off = wikit_block::append_logical_entry(
                &mut logical,
                DataEntryType::TXT as u8,
                b"<b>hi</b>",
            );
            let res_off =
                wikit_block::append_logical_entry(&mut logical, DataEntryType::MP3 as u8, mp3);
            let mut pending = vec![("hi".to_string(), text_off)];
            for alias in wikit_block::resource_key_aliases(r"\sound\uk\hello.mp3") {
                pending.push((alias, res_off));
            }
            pending.sort_by(|a, b| a.0.cmp(&b.0));
            pending.dedup_by(|a, b| a.0 == b.0);
            let (mut blocks, blob) = wikit_block::compress_logical_stream(&logical).unwrap();

            let mut writer = BufWriter::new(File::create(&path).unwrap());
            writer.write_all(WIKIT_MAGIC.as_bytes()).unwrap();
            writer.write_all(&WIKIT_FMT_V3.to_be_bytes()).unwrap();
            let hdrsz_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(4)).unwrap();
            let name = b"res";
            writer
                .write_all(&(name.len() as u16).to_be_bytes())
                .unwrap();
            writer.write_all(name).unwrap();
            let desc = b"v3";
            writer
                .write_all(&(desc.len() as u16).to_be_bytes())
                .unwrap();
            writer.write_all(desc).unwrap();
            writer.write_all(&[index::IndexFormat::FST as u8]).unwrap();
            let ibase_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(8)).unwrap();
            let isz_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(8)).unwrap();
            let dbase_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(8)).unwrap();
            let dsz_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(8)).unwrap();
            writer.write_all(&0u32.to_be_bytes()).unwrap();
            writer.write_all(&0u32.to_be_bytes()).unwrap();
            let block_table_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer
                .write_all(&wikit_block::write_block_table(&blocks))
                .unwrap();
            writer.write_all(&0u64.to_be_bytes()).unwrap(); // fbase
            writer.write_all(&0u64.to_be_bytes()).unwrap(); // fsz
            let hdrsz = writer.seek(SeekFrom::Current(0)).unwrap() as u32;
            writer.seek(SeekFrom::Start(hdrsz_pos)).unwrap();
            writer.write_all(&hdrsz.to_be_bytes()).unwrap();
            writer.seek(SeekFrom::Start(hdrsz as u64)).unwrap();
            let dstart = writer.seek(SeekFrom::Current(0)).unwrap();
            for b in &mut blocks {
                b.c_off += dstart;
            }
            writer.seek(SeekFrom::Start(block_table_pos)).unwrap();
            writer
                .write_all(&wikit_block::write_block_table(&blocks))
                .unwrap();
            writer.seek(SeekFrom::Start(dstart)).unwrap();
            writer.write_all(&blob).unwrap();
            let dend = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Start(dbase_pos)).unwrap();
            writer.write_all(&(dstart as u64).to_be_bytes()).unwrap();
            writer.seek(SeekFrom::Start(dsz_pos)).unwrap();
            writer
                .write_all(&((dend - dstart) as u64).to_be_bytes())
                .unwrap();
            writer.seek(SeekFrom::Start(dend)).unwrap();
            let (ibase, isz) = index::FSTIndex::write(&mut pending.iter(), &mut writer).unwrap();
            writer.seek(SeekFrom::Start(ibase_pos)).unwrap();
            writer.write_all(&(ibase as u64).to_be_bytes()).unwrap();
            writer.seek(SeekFrom::Start(isz_pos)).unwrap();
            writer.write_all(&(isz as u64).to_be_bytes()).unwrap();
            writer.flush().unwrap();
        }

        let dict = LocalDictionary::load(&path).expect("load");
        assert_eq!(dict.fmt_version, WIKIT_FMT_V3);
        let hits = dict.lookup("hi").expect("text");
        assert!(hits.iter().any(|(k, v)| k == "hi" && v.contains("hi")));
        let (typ, bytes) = dict.lookup_resource("hello.mp3").expect("resource");
        assert!(matches!(typ, DataEntryType::MP3));
        assert_eq!(bytes, mp3);
        let (_, bytes2) = dict
            .lookup_resource("sound/uk/hello.mp3")
            .expect("path alias");
        assert_eq!(bytes2, mp3);
    }

    #[test]
    fn v1_entries_still_readable() {
        // Synthesize a minimal v1 dictionary by writing entries uncompressed.
        let dir = tempdir().unwrap();
        let path = dir.path().join("legacy.wikit");
        let meaning = b"<b>hi</b>";
        {
            let mut writer = BufWriter::new(File::create(&path).unwrap());
            writer.write_all(WIKIT_MAGIC.as_bytes()).unwrap();
            writer.write_all(&WIKIT_FMT_V1.to_be_bytes()).unwrap();
            let hdrsz_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(4)).unwrap();
            let name = b"legacy";
            writer
                .write_all(&(name.len() as u16).to_be_bytes())
                .unwrap();
            writer.write_all(name).unwrap();
            let desc = b"v1";
            writer
                .write_all(&(desc.len() as u16).to_be_bytes())
                .unwrap();
            writer.write_all(desc).unwrap();
            writer.write_all(&[index::IndexFormat::FST as u8]).unwrap();
            let ibase_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(8)).unwrap();
            let isz_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(8)).unwrap();
            let dbase_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(8)).unwrap();
            let dsz_pos = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Current(8)).unwrap();
            writer.write_all(&0u32.to_be_bytes()).unwrap(); // script
            writer.write_all(&0u32.to_be_bytes()).unwrap(); // style
            let hdrsz = writer.seek(SeekFrom::Current(0)).unwrap() as u32;
            writer.seek(SeekFrom::Start(hdrsz_pos)).unwrap();
            writer.write_all(&hdrsz.to_be_bytes()).unwrap();
            writer.seek(SeekFrom::Start(hdrsz as u64)).unwrap();

            let dstart = writer.seek(SeekFrom::Current(0)).unwrap();
            let entry = DataEntry::new(DataEntryType::TXT, meaning.len() as u32, meaning);
            let (offset, _) = entry.write_v1(&mut writer).unwrap();
            let dend = writer.seek(SeekFrom::Current(0)).unwrap();
            writer.seek(SeekFrom::Start(dbase_pos)).unwrap();
            writer.write_all(&(dstart as u64).to_be_bytes()).unwrap();
            writer.seek(SeekFrom::Start(dsz_pos)).unwrap();
            writer
                .write_all(&((dend - dstart) as u64).to_be_bytes())
                .unwrap();
            writer.seek(SeekFrom::Start(dend)).unwrap();
            let table = vec![("hi".to_string(), offset)];
            let (ibase, isz) = index::FSTIndex::write(&mut table.iter(), &mut writer).unwrap();
            writer.seek(SeekFrom::Start(ibase_pos)).unwrap();
            writer.write_all(&(ibase as u64).to_be_bytes()).unwrap();
            writer.seek(SeekFrom::Start(isz_pos)).unwrap();
            writer.write_all(&(isz as u64).to_be_bytes()).unwrap();
            writer.flush().unwrap();
        }

        let dict = LocalDictionary::load(&path).expect("load v1");
        assert_eq!(dict.fmt_version, WIKIT_FMT_V1);
        let hits = dict.lookup("hi").expect("lookup v1");
        let exact = hits.iter().find(|(k, _)| k == "hi").expect("exact");
        assert_eq!(exact.1.as_bytes(), meaning);
    }

    #[test]
    fn embedded_fulltext_chinese_english_roundtrip() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("demo.txt");
        let content = "\
hello\n<div>This contains a rarephrase for search.</div>\n</>\n\
友谊\n<div>这是一个关于检索的中文例句。</div>\n</>\n\
";
        fs::write(&src, content).unwrap();
        let out = LocalDictionary::create(&src, None::<&Path>).expect("create");
        let dict = LocalDictionary::load(&out).expect("load");
        assert!(dict.head.fsz > 0, "fulltext archive should be embedded");
        assert!(dict.fulltext.is_some(), "fulltext index should open");

        let en = dict.search_fulltext("rarephrase", 10).expect("en search");
        assert!(
            en.iter().any(|h| h.headword == "hello"),
            "missing english body hit: {en:?}"
        );
        let zh = dict.search_fulltext("检索", 10).expect("zh search");
        assert!(
            zh.iter().any(|h| h.headword == "友谊"),
            "missing chinese body hit: {zh:?}"
        );
    }

    #[test]
    #[ignore = "manual: rebuild Oxford with embedded fulltext"]
    fn convert_oxford_with_fulltext() {
        let mdx = PathBuf::from(
            "/Users/zhqli/Downloads/牛津高阶英汉双解词典（第10版）V3.mdx",
        );
        assert!(mdx.exists());
        let out = PathBuf::from(
            "/Users/zhqli/Library/Application Support/bootsmind-wikit/local-dictionaries/牛津高阶英汉双解词典（第10版）V3-09565ab3743f7da899c15a31f0c18362.wikit",
        );
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let _ = fs::remove_file(&out);
        let started = std::time::Instant::now();
        let path = LocalDictionary::create(&mdx, Some(&out)).expect("create with fulltext");
        let elapsed = started.elapsed();
        let dict = LocalDictionary::load(&path).expect("load");
        assert!(dict.head.fsz > 0, "expected embedded fulltext");
        assert!(dict.fulltext.is_some(), "fulltext should open");
        let hits = dict.search_fulltext("example", 5).expect("search");
        eprintln!(
            "Oxford FT convert: size={} fsz={} hits={} elapsed={elapsed:?}",
            fs::metadata(&path).unwrap().len(),
            dict.head.fsz,
            hits.len()
        );
        assert!(!hits.is_empty(), "expected body hits for 'example'");
    }

    #[test]
    #[ignore = "manual lookup microbench"]
    fn lookup_speed_bench() {
        let path =
            PathBuf::from("/Users/zhqli/Downloads/牛津高阶英汉双解词典（第10版）V3.v3bench.wikit");
        assert!(path.exists());
        let dict = LocalDictionary::load(&path).expect("load");
        let words = [
            "a",
            "the",
            "interject",
            "dictionary",
            "hello",
            "world",
            "apple",
            "zebra",
            "make",
            "take",
        ];
        // warmup / cold
        let t0 = std::time::Instant::now();
        for w in &words {
            let _ = dict.lookup(w);
        }
        let cold = t0.elapsed();
        let t1 = std::time::Instant::now();
        let rounds = 200;
        for _ in 0..rounds {
            for w in &words {
                let _ = dict.lookup(w);
            }
        }
        let warm = t1.elapsed();
        let n = (words.len() * rounds) as u32;
        eprintln!(
            "wikit v3 lookup: first-pass({})={:?} ({:.2}ms/lookup), warm {} lookups={:?} ({:.3}ms/lookup)",
            words.len(),
            cold,
            cold.as_secs_f64() * 1000.0 / words.len() as f64,
            n,
            warm,
            warm.as_secs_f64() * 1000.0 / n as f64,
        );
    }

    #[test]
    #[ignore = "manual: convert Oxford MDX and compare sizes"]
    fn oxford_v3_size_vs_mdx() {
        let mdx = PathBuf::from("/Users/zhqli/Downloads/牛津高阶英汉双解词典（第10版）V3.mdx");
        assert!(mdx.exists(), "missing sample MDX at {}", mdx.display());
        let out = mdx.with_extension("v3bench.wikit");
        let _ = fs::remove_file(&out);
        let started = std::time::Instant::now();
        let path = LocalDictionary::create(&mdx, Some(&out)).expect("create v3");
        let elapsed = started.elapsed();
        let dict = LocalDictionary::load(&path).expect("load");
        assert_eq!(dict.fmt_version, WIKIT_FMT_V3);

        let mdx_sz = fs::metadata(&mdx).unwrap().len();
        let mdd = mdx.with_extension("mdd");
        let mdd_sz = fs::metadata(&mdd).map(|m| m.len()).unwrap_or(0);
        let wikit_sz = fs::metadata(&path).unwrap().len();
        let source_total = mdx_sz + mdd_sz;
        let ratio = wikit_sz as f64 / mdx_sz as f64;
        eprintln!(
            "Oxford v3 bench: mdx={mdx_sz} mdd={mdd_sz} source_total={source_total} \
             wikit={wikit_sz} ratio_vs_mdx={ratio:.3} blocks={} elapsed={elapsed:?}",
            dict.head.blocks.len()
        );
        // Text-only comparison should beat or approach MDX (zlib/LZO blocks).
        // Allow modest overhead for FST + header + block table.
        assert!(
            wikit_sz as f64 <= mdx_sz as f64 * 1.15,
            "wikit v3 ({wikit_sz}) should be within 15% of mdx ({mdx_sz}); ratio={ratio:.3}"
        );

        let sample = dict.lookup("a").unwrap_or_default();
        eprintln!("sample lookup 'a' hits={}", sample.len());
    }
}
