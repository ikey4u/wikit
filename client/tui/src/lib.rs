//! Shared query and terminal-text logic for interactive and editor-friendly lookup.
use std::{path::Path, sync::Arc};
use wikit_base::{Dictionary, wikit::RemoteDictionary};

#[derive(Clone)]
pub struct Source {
    pub name: String,
    backend: Backend,
}
#[derive(Clone)]
enum Backend {
    Local(Arc<Dictionary>),
    Remote(Arc<RemoteDictionary>, String),
}
impl Source {
    pub fn local(path: &Path) -> anyhow::Result<Self> {
        let dictionary = Dictionary::open(path)?;
        Ok(Self {
            name: dictionary.name().into(),
            backend: Backend::Local(Arc::new(dictionary)),
        })
    }
    pub fn remote(base: &str) -> anyhow::Result<Vec<Self>> {
        let remote = Arc::new(RemoteDictionary::new(
            base.into(),
            String::new(),
            String::new(),
        ));
        Ok(remote
            .get_dict_list()?
            .into_iter()
            .map(|d| Self {
                name: d.name,
                backend: Backend::Remote(remote.clone(), d.id),
            })
            .collect())
    }
    pub fn lookup(&self, word: &str) -> anyhow::Result<Vec<(String, String)>> {
        Ok(match &self.backend {
            Backend::Local(d) => d.lookup(word)?,
            Backend::Remote(d, id) => d.lookup(word, id)?,
        })
    }
}
/// Convert HTML to readable UTF-8 without executing scripts or requiring a browser.
pub fn plain_text(html: &str, width: usize) -> anyhow::Result<String> {
    Ok(html2text::from_read(html.as_bytes(), width.max(10))?)
}
/// Remove terminal control sequences from authored words, names, and rendered definitions.
pub fn terminal_text(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .collect()
}
#[derive(Default)]
pub struct QueryInput {
    pub value: String,
}
impl QueryInput {
    pub fn push(&mut self, c: char) {
        if !c.is_control() && self.value.len() < 4096 {
            self.value.push(c);
        }
    }
    pub fn backspace(&mut self) {
        self.value.pop();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_input_and_terminal_html() {
        let mut input = QueryInput::default();
        input.push('你');
        input.push('好');
        input.backspace();
        assert_eq!(input.value, "你");
        let text=plain_text("<style>body{color:red}</style><script>alert(1)</script><p>你好 <b>world</b></p><ul><li>Example</li></ul>",40).unwrap();
        assert!(text.contains("你好"));
        assert!(text.contains("Example"));
        assert!(!text.contains("alert"));
        assert!(!text.contains("color:red"));
        assert_eq!(terminal_text("hello\u{1b}[2J"), "hello[2J");
    }
    #[test]
    fn local_query_can_be_used_without_a_terminal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, "hello\n<p>A greeting.</p>\n</>\n").unwrap();
        let dictionary = wikit_base::LocalDictionary::create(&path, None::<&Path>).unwrap();
        let source = Source::local(&dictionary).unwrap();
        let entries = source.lookup("HELLO").unwrap();
        assert_eq!(entries.len(), 1);
        assert!(
            plain_text(&entries[0].1, 80)
                .unwrap()
                .contains("A greeting.")
        );
    }
}
