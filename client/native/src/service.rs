use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::Duration,
};
use url::Url;
use wikit_core::wikit::LocalDictionary;

// Serializes state transactions, including commands arriving from different screens.
static TRANSACTIONS: Mutex<()> = Mutex::new(());
const MAX_HISTORY: usize = 200;
const MAX_DOWNLOAD: u64 = 512 * 1024 * 1024;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct State {
    sources: Vec<String>,
    selected: String,
    history: Vec<Bookmark>,
    favorites: Vec<Bookmark>,
    notes: String,
    appearance: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct Bookmark {
    dictionary: String,
    word: String,
}

#[derive(Serialize)]
struct Dictionary {
    id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
enum Request {
    Snapshot,
    AddSource { source: String },
    RemoveSource { index: usize },
    Select { dictionary: String },
    Lookup { dictionary: String, word: String },
    Favorite { dictionary: String, word: String },
    ClearHistory,
    SaveNotes { notes: String },
    Appearance { appearance: String },
    Import { source: String },
    Preview { path: String },
}

fn http() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())
}

fn read_state(root: &Path) -> Result<State> {
    let file = root.join("client.json");
    if file.exists() {
        return serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Cannot read client settings: {e}"));
    }
    let mut state = State::default();
    // One-time migration; never rewrite the existing CLI/server configuration.
    if let Ok(config) = wikit_core::config::load_config() {
        state.sources = config.cltcfg.uris;
    }
    state.appearance = "system".into();
    Ok(state)
}

fn save(root: &Path, state: &State) -> Result<()> {
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(root).map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(root.join("client.json"))
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn source_url(source: &str) -> Result<Url> {
    let source = source.trim();
    if source.is_empty() {
        return Err("Enter a dictionary path or server URL".into());
    }
    let url = if source.contains("://") {
        Url::parse(source).map_err(|e| e.to_string())?
    } else {
        Url::from_file_path(Path::new(source)).map_err(|_| "Use an absolute dictionary path")?
    };
    if !matches!(url.scheme(), "http" | "https" | "file") {
        return Err("Supported sources: file, https, http".into());
    }
    if url.scheme() != "file" && url.host_str().is_none() {
        return Err("Server URL needs a host".into());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("A server source must not have a query or fragment".into());
    }
    Ok(url)
}

fn endpoint(base: &Url, operation: &str, query: &[(&str, &str)]) -> Url {
    let mut url = base.clone();
    url.set_path(&format!(
        "{}/wikit/{operation}",
        base.path().trim_end_matches('/')
    ));
    url.set_query(None);
    url.set_fragment(None);
    url.query_pairs_mut().extend_pairs(query.iter().copied());
    url
}

fn get(base: &Url, operation: &str, query: &[(&str, &str)]) -> Result<reqwest::blocking::Response> {
    let mut url = endpoint(base, operation, query);
    let user = url.username().to_owned();
    let pass = url.password().map(str::to_owned);
    let _ = url.set_username("");
    let _ = url.set_password(None);
    let mut request = http()?.get(url);
    if !user.is_empty() {
        request = request.basic_auth(user, pass);
    }
    request
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| {
            // reqwest's URL can contain query text; credentials were removed above.
            format!("Dictionary server request failed: {e}")
        })
}

fn source_id(source: &str) -> String {
    wikit_core::crypto::md5(source.as_bytes())
}
fn dictionary_id(source: &str, remote: &str) -> String {
    format!("{}:{remote}", source_id(source))
}

fn catalog(state: &State) -> (Vec<Dictionary>, Vec<String>) {
    let mut dictionaries = Vec::new();
    let mut errors = Vec::new();
    for (i, source) in state.sources.iter().enumerate() {
        let result = (|| -> Result<Vec<Dictionary>> {
            let url = source_url(source)?;
            if url.scheme() == "file" {
                let path = url.to_file_path().map_err(|_| "Invalid file URI")?;
                let dict = LocalDictionary::load(path).map_err(|e| e.to_string())?;
                Ok(vec![Dictionary {
                    id: dictionary_id(source, "local"),
                    name: dict.head.name,
                }])
            } else {
                let list: Vec<wikit_proto_meta::Meta> =
                    get(&url, "list", &[])?.json().map_err(|e| e.to_string())?;
                Ok(list
                    .into_iter()
                    .map(|d| Dictionary {
                        id: dictionary_id(source, &d.id),
                        name: d.name,
                    })
                    .collect())
            }
        })();
        match result {
            Ok(mut list) => dictionaries.append(&mut list),
            Err(e) => errors.push(format!("Source {}: {e}", i + 1)),
        }
    }
    (dictionaries, errors)
}

mod wikit_proto_meta {
    #[derive(serde::Deserialize)]
    pub struct Meta {
        pub id: String,
        pub name: String,
    }
}

fn snapshot(state: &State, root: &Path) -> Value {
    let (dictionaries, errors) = catalog(state);
    let sources: Vec<String> = state
        .sources
        .iter()
        .map(|source| {
            if let Ok(mut url) = Url::parse(source) {
                let _ = url.set_username("");
                let _ = url.set_password(None);
                url.to_string()
            } else {
                source.clone()
            }
        })
        .collect();
    json!({"dictionaries": dictionaries, "errors": errors, "sources": sources, "selected": state.selected,
        "history": state.history, "favorites": state.favorites, "notes": state.notes,
        "appearance": state.appearance, "dataDirectory": root.to_string_lossy()})
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn lookup(state: &State, id: &str, word: &str) -> Result<Value> {
    let (key, remote) = id.split_once(':').ok_or("Select a dictionary first")?;
    let source = state
        .sources
        .iter()
        .find(|s| source_id(s) == key)
        .ok_or("Dictionary was removed; select another")?;
    let url = source_url(source)?;
    let (entries, style, script, base) = if url.scheme() == "file" {
        let path = url.to_file_path().map_err(|_| "Invalid dictionary path")?;
        let dict = LocalDictionary::load(&path).map_err(|e| e.to_string())?;
        let base = Url::from_directory_path(path.parent().ok_or("Invalid path")?)
            .map_err(|_| "Invalid path")?
            .to_string();
        (
            dict.lookup(wikit_core::util::normalize_word(word))
                .map_err(|e| e.to_string())?,
            dict.head.style,
            dict.head.script,
            base,
        )
    } else {
        let query = [("word", word), ("dictname", remote)];
        let entries: Vec<(String, String)> = get(&url, "query", &query)?
            .json()
            .map_err(|e| e.to_string())?;
        let style = get(&url, "style", &[("dictname", remote)])?
            .text()
            .map_err(|e| e.to_string())?;
        let script = get(&url, "script", &[("dictname", remote)])?
            .text()
            .map_err(|e| e.to_string())?;
        let mut base = url;
        let _ = base.set_username("");
        let _ = base.set_password(None);
        (
            entries,
            style,
            script,
            format!("{}/", base.as_str().trim_end_matches('/')),
        )
    };
    let mut seen = std::collections::HashSet::new();
    let entries: Vec<_> = entries
        .into_iter()
        .filter(|(word, _)| seen.insert(word.clone()))
        .collect();
    let mut body = String::new();
    for (headword, meaning) in &entries {
        body.push_str(&format!(
            "<article><h2>{}</h2>{meaning}</article>",
            escape(headword)
        ));
    }
    if entries.is_empty() {
        body.push_str("<p>No matching entry. Try another spelling.</p>");
    }
    let html = format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><base href="{}"><style>body{{font-family:system-ui,sans-serif;margin:20px;line-height:1.6;overflow-wrap:anywhere}}img,video{{max-width:100%}}article+article{{border-top:1px solid #ddd;margin-top:24px}}{style}</style></head><body>{body}<script>{script}</script></body></html>"#,
        escape(&base)
    );
    Ok(
        json!({"word": word, "dictionary": id, "count": entries.len(), "html": html, "baseUrl": base}),
    )
}

fn import(root: &Path, source: &str) -> Result<String> {
    let dir = root.join("dictionaries");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut target = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
    let mut input: Box<dyn Read> =
        if source.starts_with("https://") || source.starts_with("http://") {
            Box::new(
                http()?
                    .get(source)
                    .send()
                    .and_then(|r| r.error_for_status())
                    .map_err(|e| e.to_string())?,
            )
        } else {
            let path = if source.starts_with("file://") {
                Url::parse(source)
                    .map_err(|e| e.to_string())?
                    .to_file_path()
                    .map_err(|_| "Invalid file path")?
            } else {
                PathBuf::from(source)
            };
            Box::new(fs::File::open(path).map_err(|e| e.to_string())?)
        };
    let size = std::io::copy(&mut input.by_ref().take(MAX_DOWNLOAD + 1), &mut target)
        .map_err(|e| e.to_string())?;
    if size > MAX_DOWNLOAD {
        return Err("Dictionary exceeds 512 MiB".into());
    }
    // Validate before publishing the imported source.
    let dictionary = LocalDictionary::load(target.path())
        .map_err(|e| format!("Invalid Wikit dictionary: {e}"))?;
    dictionary
        .lookup("")
        .map_err(|e| format!("Invalid Wikit index: {e}"))?;
    let path = target.path().with_extension("wikit");
    target.persist(&path).map_err(|e| e.to_string())?;
    Url::from_file_path(path)
        .map(|u| u.to_string())
        .map_err(|_| "Invalid imported path".into())
}

pub fn dispatch(root: &Path, payload: &str) -> Result<String> {
    let request: Request = serde_json::from_str(payload).map_err(|e| e.to_string())?;
    let _transaction = TRANSACTIONS
        .lock()
        .map_err(|_| "Settings transaction failed")?;
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let mut state = read_state(root)?;
    let mut changed = true;
    let value = match request {
        Request::Snapshot => {
            changed = false;
            snapshot(&state, root)
        }
        Request::AddSource { source } => {
            let source = source_url(&source)?.to_string();
            if !state.sources.contains(&source) {
                state.sources.push(source);
            }
            snapshot(&state, root)
        }
        Request::RemoveSource { index } => {
            if index >= state.sources.len() {
                return Err("Source no longer exists".into());
            }
            let removed = state.sources.remove(index);
            let prefix = format!("{}:", source_id(&removed));
            if state.selected.starts_with(&prefix) {
                state.selected.clear();
            }
            state.history.retain(|b| !b.dictionary.starts_with(&prefix));
            state
                .favorites
                .retain(|b| !b.dictionary.starts_with(&prefix));
            snapshot(&state, root)
        }
        Request::Select { dictionary } => {
            state.selected = dictionary;
            json!({"ok":true})
        }
        Request::Lookup { dictionary, word } => {
            let word = word.trim().to_owned();
            if word.is_empty() {
                return Err("Enter a word to look up".into());
            }
            if word.len() > 512 {
                return Err("Search text is too long".into());
            }
            let result = lookup(&state, &dictionary, &word)?;
            state.selected = dictionary.clone();
            let bookmark = Bookmark { dictionary, word };
            state.history.retain(|b| b != &bookmark);
            state.history.insert(0, bookmark);
            state.history.truncate(MAX_HISTORY);
            result
        }
        Request::Favorite { dictionary, word } => {
            let bookmark = Bookmark { dictionary, word };
            if state.favorites.contains(&bookmark) {
                state.favorites.retain(|b| b != &bookmark);
            } else {
                state.favorites.insert(0, bookmark);
            }
            json!({"favorites": state.favorites})
        }
        Request::ClearHistory => {
            state.history.clear();
            json!({"ok": true})
        }
        Request::SaveNotes { notes } => {
            state.notes = notes;
            json!({"ok":true})
        }
        Request::Appearance { appearance } => {
            if !matches!(appearance.as_str(), "system" | "light" | "dark") {
                return Err("Invalid appearance".into());
            }
            state.appearance = appearance;
            json!({"ok":true})
        }
        Request::Import { source } => {
            let source = import(root, &source)?;
            state.sources.push(source);
            snapshot(&state, root)
        }
        Request::Preview { path } => {
            changed = false;
            let path = PathBuf::from(path);
            let path = if path.is_dir() {
                path.join("index.html")
            } else {
                path
            };
            let html = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let base = Url::from_directory_path(path.parent().ok_or("Invalid preview path")?)
                .map_err(|_| "Invalid preview path")?;
            json!({"html": html, "baseUrl": base.as_str()})
        }
    };
    if changed || !root.join("client.json").exists() {
        save(root, &state)?;
    }
    serde_json::to_string(&value).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn call(root: &Path, payload: Value) -> Value {
        serde_json::from_str(&dispatch(root, &payload.to_string()).unwrap()).unwrap()
    }
    fn fixture(root: &Path, name: &str) -> PathBuf {
        let source = root.join(format!("{name}.txt"));
        fs::write(&source, "hello\n<p>Hello world</p>\n</>\n").unwrap();
        LocalDictionary::create(&source, Some(root.join(format!("{name}.wikit")))).unwrap()
    }
    fn clean() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        save(
            root.path(),
            &State {
                appearance: "system".into(),
                ..Default::default()
            },
        )
        .unwrap();
        root
    }
    #[test]
    fn import_lookup_favorite_and_restart() {
        let root = clean();
        let dictionary = fixture(root.path(), "demo");
        let snapshot = call(root.path(), json!({"action":"import", "source":dictionary}));
        let id = snapshot["dictionaries"][0]["id"].as_str().unwrap();
        assert!(
            snapshot["sources"][0]
                .as_str()
                .unwrap()
                .contains("dictionaries/")
        );
        // The imported copy survives removal of the original.
        fs::remove_file(dictionary).unwrap();
        let result = call(
            root.path(),
            json!({"action":"lookup", "dictionary":id, "word":"hello"}),
        );
        assert!(result["html"].as_str().unwrap().contains("Hello world"));
        call(
            root.path(),
            json!({"action":"favorite", "dictionary":id, "word":"hello"}),
        );
        call(
            root.path(),
            json!({"action":"saveNotes", "notes":"例句与笔记"}),
        );
        let restored = read_state(root.path()).unwrap();
        assert_eq!(restored.history.len(), 1);
        assert_eq!(restored.favorites.len(), 1);
        assert_eq!(restored.notes, "例句与笔记");
        call(
            root.path(),
            json!({"action":"favorite", "dictionary":id, "word":"hello"}),
        );
        assert!(read_state(root.path()).unwrap().favorites.is_empty());
    }
    #[test]
    fn removing_source_preserves_other_dictionary_references() {
        let root = clean();
        for name in ["first", "second"] {
            let file = fixture(root.path(), name);
            call(root.path(), json!({"action":"addSource", "source":file}));
        }
        let snapshot = call(root.path(), json!({"action":"snapshot"}));
        let id = snapshot["dictionaries"][1]["id"].as_str().unwrap();
        call(
            root.path(),
            json!({"action":"favorite", "dictionary":id, "word":"hello"}),
        );
        let next = call(root.path(), json!({"action":"removeSource", "index":0}));
        assert_eq!(next["dictionaries"][0]["id"], id);
        assert_eq!(next["favorites"][0]["dictionary"], id);
    }
    #[test]
    fn invalid_import_and_invalid_commands_preserve_settings() {
        let root = clean();
        let invalid = root.path().join("broken.wikit");
        fs::write(&invalid, "not a dictionary").unwrap();
        assert!(
            dispatch(
                root.path(),
                &json!({"action":"import", "source":invalid}).to_string()
            )
            .is_err()
        );
        assert!(read_state(root.path()).unwrap().sources.is_empty());
        assert!(
            dispatch(
                root.path(),
                r#"{"action":"appearance","appearance":"invalid"}"#
            )
            .is_err()
        );
        assert_eq!(read_state(root.path()).unwrap().appearance, "system");
        assert!(source_url("javascript:alert(1)").is_err());
        assert!(source_url("relative/path").is_err());
    }
    #[test]
    fn remote_query_encodes_search_and_dictionary_names() {
        let base = Url::parse("https://user:secret@example.com/prefix/").unwrap();
        let url = endpoint(
            &base,
            "query",
            &[("word", "a&b 中文?"), ("dictname", "one/two & three")],
        );
        assert_eq!(url.path(), "/prefix/wikit/query");
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["word"], "a&b 中文?");
        assert_eq!(pairs["dictname"], "one/two & three");
    }
    #[test]
    fn html_heading_is_escaped() {
        assert_eq!(
            escape("<img onerror='x'>&\""),
            "&lt;img onerror=&#39;x&#39;&gt;&amp;&quot;"
        );
    }
    #[test]
    fn history_is_bounded_and_preview_resolves_base_directory() {
        let root = clean();
        let file = fixture(root.path(), "demo");
        let result = call(root.path(), json!({"action":"addSource", "source":file}));
        let id = result["dictionaries"][0]["id"].as_str().unwrap();
        let mut state = read_state(root.path()).unwrap();
        state.history = (0..MAX_HISTORY)
            .map(|i| Bookmark {
                dictionary: id.into(),
                word: i.to_string(),
            })
            .collect();
        save(root.path(), &state).unwrap();
        call(
            root.path(),
            json!({"action":"lookup", "dictionary":id, "word":"hello"}),
        );
        assert_eq!(read_state(root.path()).unwrap().history.len(), MAX_HISTORY);
        let html = root.path().join("index.html");
        fs::write(&html, "<h1>Preview</h1>").unwrap();
        let preview = call(root.path(), json!({"action":"preview", "path":root.path()}));
        assert_eq!(preview["html"], "<h1>Preview</h1>");
        assert!(preview["baseUrl"].as_str().unwrap().starts_with("file://"));
    }
}
