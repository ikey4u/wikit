//! Explicit-path dictionary API. Opening a dictionary never reads global app configuration.
use crate::{
    error::{Context, WikitResult},
    mdict, util,
    wikit::LocalDictionary,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// A reusable local `.wikit` or `.mdx` dictionary.
/// MDX is parsed into memory once; Wikit uses its cached FST index.
pub struct Dictionary {
    path: PathBuf,
    name: String,
    content: Content,
}
enum Content {
    Wikit(LocalDictionary),
    Mdx(BTreeMap<String, String>),
}
impl Dictionary {
    pub fn open(path: impl AsRef<Path>) -> WikitResult<Self> {
        let path = path.as_ref().canonicalize()?;
        let (name, content) = if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("mdx"))
        {
            let parsed = mdict::parse_mdx(
                path.to_str().context("Dictionary path must be UTF-8")?,
                None,
            )?;
            let name = parsed.header.get("Title").cloned().unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            });
            (name, Content::Mdx(parsed.entries.into_iter().collect()))
        } else {
            let dictionary = LocalDictionary::load(&path)?;
            (dictionary.head.name.clone(), Content::Wikit(dictionary))
        };
        Ok(Self {
            path,
            name,
            content,
        })
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn style(&self) -> &str {
        match &self.content {
            Content::Wikit(d) => d.get_style(),
            _ => "",
        }
    }
    pub fn script(&self) -> &str {
        match &self.content {
            Content::Wikit(d) => d.get_script(),
            _ => "",
        }
    }
    /// Find an exact word first, followed by up to twenty nearby words.
    pub fn lookup(&self, word: &str) -> WikitResult<Vec<(String, String)>> {
        let word = util::normalize_word(word.trim());
        if word.is_empty() {
            return Ok(Vec::new());
        }
        match &self.content {
            Content::Wikit(d) => d.lookup(word),
            Content::Mdx(entries) => {
                let mut results = Vec::new();
                if let Some(value) = entries.get(&word) {
                    results.push((word.clone(), value.clone()));
                }
                for (key, value) in entries.range(word.clone()..) {
                    if !key.starts_with(&word) {
                        break;
                    }
                    if key != &word {
                        results.push((key.clone(), value.clone()));
                    }
                    if results.len() >= 20 {
                        break;
                    }
                }
                Ok(results)
            }
        }
    }
}
