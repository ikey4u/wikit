//! Embedded Tantivy full-text index (Chinese jieba + Latin) packed into `.wikit`.

use crate::config;
use crate::crypto;
use crate::error::{WikitError, WikitResult};
use crate::zstdutil;

use cang_jie::{CangJieTokenizer, TokenizerOption, CANG_JIE};
use jieba_rs::Jieba;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{
    IndexRecordOption, Schema, SchemaBuilder, TextFieldIndexing, TextOptions, Value, STORED, STRING,
};
use tantivy::{doc, Index, IndexReader, ReloadPolicy, TantivyDocument};

const TOKENIZER_NAME: &str = CANG_JIE;
const ARCHIVE_MAGIC: &[u8; 4] = b"WKFT";
/// v2: zstd(custom file bundle) — avoids fragile tar packing on large indexes.
const ARCHIVE_VERSION: u32 = 2;

static HTML_TAG_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?is)<[^>]+>").unwrap());
static WS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

#[derive(Debug, Clone)]
pub struct FulltextHit {
    pub headword: String,
    pub snippet: String,
}

pub struct FulltextIndex {
    _index: Index,
    reader: IndexReader,
    headword_field: tantivy::schema::Field,
    body_field: tantivy::schema::Field,
}

/// Strip HTML tags and collapse whitespace for indexing.
pub fn html_to_text(html: &str) -> String {
    let no_tags = HTML_TAG_RE.replace_all(html, " ");
    let decoded = no_tags
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    WS_RE.replace_all(decoded.trim(), " ").into_owned()
}

fn build_schema() -> (Schema, tantivy::schema::Field, tantivy::schema::Field) {
    let mut builder = SchemaBuilder::default();
    let headword = builder.add_text_field("headword", STRING | STORED);
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer(TOKENIZER_NAME)
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let body_opts = TextOptions::default()
        .set_indexing_options(text_indexing)
        .set_stored();
    let body = builder.add_text_field("body", body_opts);
    (builder.build(), headword, body)
}

fn register_tokenizer(index: &Index) {
    let tokenizer = CangJieTokenizer {
        worker: Arc::new(Jieba::new()),
        option: TokenizerOption::Default { hmm: true },
    };
    index.tokenizers().register(TOKENIZER_NAME, tokenizer);
}

/// Build a Tantivy index directory from (headword, plain body) pairs.
pub fn build_index_dir(dir: &Path, entries: &[(String, String)]) -> WikitResult<()> {
    build_index_dir_with_progress(dir, entries, |_| {})
}

pub fn build_index_dir_with_progress<F>(
    dir: &Path,
    entries: &[(String, String)],
    mut progress: F,
) -> WikitResult<()>
where
    F: FnMut(f64),
{
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)?;

    let (schema, headword_f, body_f) = build_schema();
    let index = Index::create_in_dir(dir, schema)
        .map_err(|e| WikitError::new(format!("tantivy create: {e}")))?;
    register_tokenizer(&index);

    let mut writer = index
        .writer(64 * 1024 * 1024)
        .map_err(|e| WikitError::new(format!("tantivy writer: {e}")))?;

    let total = entries.len().max(1);
    let step = (total / 200).max(1);
    for (i, (head, body)) in entries.iter().enumerate() {
        if head.trim().is_empty() || body.trim().is_empty() {
            continue;
        }
        writer
            .add_document(doc!(
                headword_f => head.as_str(),
                body_f => body.as_str(),
            ))
            .map_err(|e| WikitError::new(format!("tantivy add_document: {e}")))?;
        if i % step == 0 || i + 1 == total {
            progress((i + 1) as f64 / total as f64);
        }
    }
    writer
        .commit()
        .map_err(|e| WikitError::new(format!("tantivy commit: {e}")))?;
    progress(1.0);
    Ok(())
}

/// Collect files under `dir` as relative posix paths.
fn collect_files(dir: &Path) -> WikitResult<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();
    fn walk(base: &Path, cur: &Path, out: &mut Vec<(String, Vec<u8>)>) -> WikitResult<()> {
        for entry in fs::read_dir(cur)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, out)?;
            } else if path.is_file() {
                let rel = path
                    .strip_prefix(base)
                    .map_err(|e| WikitError::new(format!("strip prefix: {e}")))?
                    .to_string_lossy()
                    .replace('\\', "/");
                let bytes = fs::read(&path)?;
                out.push((rel, bytes));
            }
        }
        Ok(())
    }
    walk(dir, dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(files)
}

fn encode_bundle(files: &[(String, Vec<u8>)]) -> WikitResult<Vec<u8>> {
    let mut plain = Vec::new();
    plain.extend_from_slice(&(files.len() as u32).to_be_bytes());
    for (name, data) in files {
        let name_bytes = name.as_bytes();
        if name_bytes.len() > u16::MAX as usize {
            return Err(WikitError::new("file name too long in fulltext bundle"));
        }
        plain.extend_from_slice(&(name_bytes.len() as u16).to_be_bytes());
        plain.extend_from_slice(name_bytes);
        plain.extend_from_slice(&(data.len() as u64).to_be_bytes());
        plain.extend_from_slice(data);
    }
    Ok(plain)
}

fn decode_bundle(plain: &[u8], dest: &Path) -> WikitResult<()> {
    if plain.len() < 4 {
        return Err(WikitError::new("fulltext bundle truncated"));
    }
    let count = u32::from_be_bytes(plain[0..4].try_into().unwrap()) as usize;
    let mut off = 4usize;
    for _ in 0..count {
        if off + 2 > plain.len() {
            return Err(WikitError::new("fulltext bundle name length truncated"));
        }
        let nlen = u16::from_be_bytes(plain[off..off + 2].try_into().unwrap()) as usize;
        off += 2;
        if off + nlen + 8 > plain.len() {
            return Err(WikitError::new("fulltext bundle entry truncated"));
        }
        let name = std::str::from_utf8(&plain[off..off + nlen])
            .map_err(|e| WikitError::new(format!("bad file name utf8: {e}")))?
            .to_string();
        off += nlen;
        let dlen = u64::from_be_bytes(plain[off..off + 8].try_into().unwrap()) as usize;
        off += 8;
        if off + dlen > plain.len() {
            return Err(WikitError::new("fulltext bundle data truncated"));
        }
        // Reject path traversal.
        let dest_path = dest.join(&name);
        if !dest_path.starts_with(dest) || name.contains("..") {
            return Err(WikitError::new(format!("unsafe path in bundle: {name}")));
        }
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest_path, &plain[off..off + dlen])?;
        off += dlen;
    }
    Ok(())
}

/// Pack index directory → `WKFT` + version + plain_sz + zstd(bundle).
pub fn pack_index_dir(dir: &Path) -> WikitResult<Vec<u8>> {
    let files = collect_files(dir)?;
    let plain = encode_bundle(&files)?;
    let compressed = zstdutil::compress(&plain, zstdutil::DEFAULT_BLOCK_LEVEL)?;
    let mut out = Vec::with_capacity(4 + 4 + 8 + compressed.len());
    out.extend_from_slice(ARCHIVE_MAGIC);
    out.extend_from_slice(&ARCHIVE_VERSION.to_be_bytes());
    out.extend_from_slice(&(plain.len() as u64).to_be_bytes());
    out.extend_from_slice(&compressed);
    Ok(out)
}

pub fn build_archive(entries: &[(String, String)]) -> WikitResult<Vec<u8>> {
    build_archive_with_progress(entries, |_| {})
}

pub fn build_archive_with_progress<F>(entries: &[(String, String)], mut progress: F) -> WikitResult<Vec<u8>>
where
    F: FnMut(f64),
{
    let tmp = tempfile::tempdir().map_err(|e| WikitError::new(format!("tempdir: {e}")))?;
    build_index_dir_with_progress(tmp.path(), entries, |p| progress(p * 0.85))?;
    progress(0.88);
    let packed = pack_index_dir(tmp.path())?;
    progress(1.0);
    Ok(packed)
}

fn unpack_archive_to(archive: &[u8], dest: &Path) -> WikitResult<()> {
    if archive.len() < 16 || &archive[0..4] != ARCHIVE_MAGIC {
        return Err(WikitError::new("invalid fulltext archive magic"));
    }
    let version = u32::from_be_bytes(archive[4..8].try_into().unwrap());
    if version != ARCHIVE_VERSION {
        return Err(WikitError::new(format!(
            "unsupported fulltext archive version {version} (need {ARCHIVE_VERSION})"
        )));
    }
    let plain_sz = u64::from_be_bytes(archive[8..16].try_into().unwrap()) as usize;
    let compressed = &archive[16..];
    let plain = zstdutil::decompress(compressed, plain_sz)?;

    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;
    decode_bundle(&plain, dest)?;
    Ok(())
}

fn cache_dir_for(dict_path: &Path, fbase: u64, fsz: u64) -> WikitResult<PathBuf> {
    let conf = config::get_config_dir().map_err(|e| WikitError::new(format!("{e}")))?;
    let key = format!("{}|{}|{}", dict_path.display(), fbase, fsz);
    let hash = crypto::md5(key.as_bytes());
    Ok(conf.join("ft_index").join(hash))
}

/// Open embedded fulltext from wikit file region; extracts to cache on first use.
pub fn open_from_wikit(
    dict_path: &Path,
    fbase: u64,
    fsz: u64,
) -> WikitResult<Option<FulltextIndex>> {
    if fbase == 0 || fsz == 0 {
        return Ok(None);
    }
    let cache = cache_dir_for(dict_path, fbase, fsz)?;
    let ready = cache.join(".ready");
    if !ready.exists() {
        let mut file = File::open(dict_path)?;
        use std::io::{Seek, SeekFrom};
        file.seek(SeekFrom::Start(fbase))?;
        let mut buf = vec![0u8; fsz as usize];
        file.read_exact(&mut buf)?;
        unpack_archive_to(&buf, &cache)?;
        File::create(&ready)?;
    }

    let index =
        Index::open_in_dir(&cache).map_err(|e| WikitError::new(format!("tantivy open: {e}")))?;
    register_tokenizer(&index);
    let schema = index.schema();
    let headword_f = schema
        .get_field("headword")
        .map_err(|e| WikitError::new(format!("missing headword field: {e}")))?;
    let body_f = schema
        .get_field("body")
        .map_err(|e| WikitError::new(format!("missing body field: {e}")))?;
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()
        .map_err(|e| WikitError::new(format!("tantivy reader: {e}")))?;

    Ok(Some(FulltextIndex {
        _index: index,
        reader,
        headword_field: headword_f,
        body_field: body_f,
    }))
}

impl FulltextIndex {
    pub fn search(&self, query: &str, limit: usize) -> WikitResult<Vec<FulltextHit>> {
        let q = query.trim();
        if q.is_empty() || limit == 0 {
            return Ok(Vec::new());
        }
        let searcher = self.reader.searcher();
        let parser = QueryParser::for_index(&self._index, vec![self.body_field]);
        let parsed = match parser.parse_query(q) {
            Ok(p) => p,
            Err(_) => {
                // Escape special query syntax — treat as literal phrase-ish terms.
                let escaped = q
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace(':', " ")
                    .replace('[', " ")
                    .replace(']', " ")
                    .replace('{', " ")
                    .replace('}', " ")
                    .replace('(', " ")
                    .replace(')', " ")
                    .replace('!', " ")
                    .replace('^', " ")
                    .replace('~', " ")
                    .replace('*', " ")
                    .replace('?', " ");
                parser
                    .parse_query(&escaped)
                    .map_err(|e| WikitError::new(format!("query parse: {e}")))?
            }
        };

        let top = searcher
            .search(&parsed, &TopDocs::with_limit(limit.max(1)).order_by_score())
            .map_err(|e| WikitError::new(format!("search: {e}")))?;

        let mut hits = Vec::new();
        for (_score, addr) in top {
            let doc: TantivyDocument = searcher
                .doc(addr)
                .map_err(|e| WikitError::new(format!("doc: {e}")))?;
            let headword = doc
                .get_first(self.headword_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let body = doc
                .get_first(self.body_field)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let snippet = make_snippet(body, q, 120);
            if !headword.is_empty() {
                hits.push(FulltextHit { headword, snippet });
            }
        }
        Ok(hits)
    }
}

fn make_snippet(body: &str, query: &str, max_len: usize) -> String {
    let lower_body = body.to_lowercase();
    let needle = query.trim().to_lowercase();
    let pos = if needle.is_empty() {
        None
    } else {
        lower_body.find(&needle)
    };
    let start = match pos {
        Some(p) => p.saturating_sub(40),
        None => 0,
    };
    let mut end = (start + max_len).min(body.len());
    // Align to char boundary.
    while end < body.len() && !body.is_char_boundary(end) {
        end += 1;
    }
    let mut start = start;
    while start > 0 && !body.is_char_boundary(start) {
        start -= 1;
    }
    let mut snip = body[start..end].to_string();
    if start > 0 {
        snip.insert_str(0, "…");
    }
    if end < body.len() {
        snip.push('…');
    }
    snip
}

/// Collect indexable text entries (skip LINK redirects).
pub fn collect_index_entries(word_meaning_list: &[(String, String)]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (word, meaning) in word_meaning_list {
        let w = word.trim();
        if w.is_empty() || meaning.trim_start().starts_with("@@@LINK=") {
            continue;
        }
        if !seen.insert(w.to_string()) {
            continue;
        }
        let plain = html_to_text(meaning);
        if plain.is_empty() {
            continue;
        }
        out.push((w.to_string(), plain));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_and_english_search() {
        let entries = vec![
            (
                "hello".to_string(),
                "This is an example sentence about friendship.".to_string(),
            ),
            (
                "友谊".to_string(),
                "这是一个关于友谊的例句，用于全文检索测试。".to_string(),
            ),
        ];
        let archive = build_archive(&entries).expect("build");
        assert!(archive.starts_with(ARCHIVE_MAGIC));

        let tmp = tempfile::tempdir().unwrap();
        unpack_archive_to(&archive, tmp.path()).expect("unpack");
        let index = Index::open_in_dir(tmp.path()).expect("open");
        register_tokenizer(&index);
        let (schema, headword_f, body_f) = build_schema();
        let _ = (schema, headword_f);
        let reader = index.reader().unwrap();
        let ft = FulltextIndex {
            _index: index,
            reader,
            headword_field: headword_f,
            body_field: body_f,
        };

        let en = ft.search("example", 10).expect("en");
        assert!(
            en.iter().any(|h| h.headword == "hello"),
            "english hit missing: {en:?}"
        );
        let zh = ft.search("友谊", 10).expect("zh");
        assert!(
            zh.iter().any(|h| h.headword == "友谊"),
            "chinese hit missing: {zh:?}"
        );
    }

    #[test]
    fn html_stripped() {
        let t = html_to_text("<div>hello&nbsp;<b>world</b></div>");
        assert_eq!(t, "hello world");
    }
}
