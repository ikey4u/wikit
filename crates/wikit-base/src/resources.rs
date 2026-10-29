//! Ordered dictionary CSS/JavaScript resources, relative to the source file.
use crate::error::{Context, WikitResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A legacy inline string, comma-separated filenames, or an ordered filename array.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Resources {
    Text(String),
    Files(Vec<String>),
}
impl Default for Resources {
    fn default() -> Self {
        Self::Text(String::new())
    }
}
impl Resources {
    pub fn load(&self, directory: &Path, extension: &str) -> WikitResult<String> {
        let files = match self {
            Self::Files(files) => files.clone(),
            Self::Text(text) => {
                let text = text.trim();
                if text.is_empty() {
                    return Ok(String::new());
                }
                // Preserve legacy inline CSS/JS. Filename lists require matching extensions.
                if !text.starts_with('@')
                    && !text
                        .split(',')
                        .filter(|s| !s.trim().is_empty())
                        .all(|s| s.trim().ends_with(&format!(".{extension}")))
                {
                    return Ok(text.to_owned());
                }
                text.split(',').map(str::to_owned).collect()
            }
        };
        let mut contents = Vec::new();
        for filename in files {
            let filename = filename.trim().trim_start_matches('@');
            if filename.is_empty() {
                continue;
            }
            let path = directory.join(filename);
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Cannot read {extension} resource {}", path.display()))?;
            contents.push(if extension == "css" {
                rebase_css(&content, filename)
            } else {
                content
            });
        }
        // A semicolon prevents adjacent scripts from being joined into one expression.
        Ok(contents.join(if extension == "js" { "\n;\n" } else { "\n\n" }))
    }
}

fn rebase_css(css: &str, filename: &str) -> String {
    let parent = Path::new(filename).parent().unwrap_or(Path::new(""));
    if parent.as_os_str().is_empty() {
        return css.to_owned();
    }
    let urls = regex::Regex::new(r#"url\(\s*([^)]*?)\s*\)"#).unwrap();
    urls.replace_all(css, |caps: &regex::Captures<'_>| {
        let raw = caps[1].trim();
        let value = raw.trim_matches(['\'', '"']);
        if value.is_empty() || value.starts_with(['/', '#']) || value.contains(':') {
            caps[0].to_owned()
        } else {
            let path = parent.join(value).to_string_lossy().replace('\\', "/");
            format!("url(\"{}\")", path.replace('"', "%22"))
        }
    })
    .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_files_and_legacy_inline() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.css"), "p { color: red }").unwrap();
        std::fs::write(dir.path().join("b.css"), "p { color: blue }").unwrap();
        let list = Resources::Text("a.css, b.css, ".into());
        let css = list.load(dir.path(), "css").unwrap();
        assert!(css.find("red").unwrap() < css.find("blue").unwrap());
        assert_eq!(
            Resources::Files(vec!["a.css".into(), "b.css".into()])
                .load(dir.path(), "css")
                .unwrap(),
            css
        );
        assert_eq!(
            Resources::Text("@a.css".into())
                .load(dir.path(), "css")
                .unwrap(),
            "p { color: red }"
        );
        assert_eq!(
            Resources::Text("p {color:red}".into())
                .load(dir.path(), "css")
                .unwrap(),
            "p {color:red}"
        );
        assert!(
            Resources::Text("missing.css".into())
                .load(dir.path(), "css")
                .is_err()
        );
        std::fs::write(dir.path().join("a.js"), "window.order = [1]").unwrap();
        std::fs::write(dir.path().join("b.js"), "window.order.push(2)").unwrap();
        assert_eq!(
            Resources::Text("a.js, b.js".into())
                .load(dir.path(), "js")
                .unwrap(),
            "window.order = [1]\n;\nwindow.order.push(2)"
        );
    }
    #[test]
    fn nested_css_keeps_relative_media_paths() {
        let css = rebase_css(
            "a{background:url('../img/a.png')} b{src:url(data:abc)}",
            "styles/main.css",
        );
        assert!(css.contains("styles/../img/a.png"));
        assert!(css.contains("url(data:abc)"));
    }
}
