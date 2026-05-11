use anyhow::{Context, Result as AnyResult};
use napi::{Error, Result as NapiResult};
use napi_derive::napi;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use wikit_core::{config, crypto, preview, util, wikit};
use wikit_core::wikit::WikitDictionary;

static DICTDB: Lazy<Arc<Mutex<HashMap<String, WikitDictionary>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});
static INTERNAL_FS_PORT: AtomicU16 = AtomicU16::new(7561);
static STATIC_SERVER_STARTED: AtomicBool = AtomicBool::new(false);
static STATIC_SERVER_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static PREVIEW_SERVER_STARTED: AtomicBool = AtomicBool::new(false);
static PREVIEW_SHUTDOWN: Lazy<Mutex<Option<tokio::sync::broadcast::Sender<()>>>> = Lazy::new(|| Mutex::new(None));

#[napi(object)]
pub struct DictMeta {
    pub name: String,
    pub id: String,
}

#[napi(object)]
pub struct LookupResponse {
    pub words: HashMap<String, String>,
    pub script: String,
    pub style: String,
}

fn napi_error<E: std::fmt::Debug>(context: &str, error: E) -> Error {
    Error::from_reason(format!("{context}: {error:?}"))
}

fn lock_dictdb() -> NapiResult<std::sync::MutexGuard<'static, HashMap<String, WikitDictionary>>> {
    DICTDB.lock().map_err(|e| Error::from_reason(format!("failed to lock dictionary database: {e}")))
}

fn lock_preview_shutdown() -> NapiResult<std::sync::MutexGuard<'static, Option<tokio::sync::broadcast::Sender<()>>>> {
    PREVIEW_SHUTDOWN.lock().map_err(|e| Error::from_reason(format!("failed to lock preview server state: {e}")))
}

fn choose_static_port() -> NapiResult<u16> {
    let restricted_ports = [6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697u16];
    loop {
        let port = util::get_free_tcp_port(Some(INTERNAL_FS_PORT.load(Ordering::SeqCst)))
            .ok_or_else(|| Error::from_reason("failed to get static file server port"))?;
        if !restricted_ports.contains(&port) {
            return Ok(port);
        }
    }
}

fn run_static_file_server(port: u16) -> AnyResult<()> {
    use axum::{http::StatusCode, routing::get_service, Router};
    use std::net::SocketAddr;
    use tower_http::services::ServeDir;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(5)
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let app = Router::new().nest(
            "/static",
            get_service(ServeDir::new(config::get_static_dir()?)).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        );
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        axum::Server::bind(&addr).serve(app.into_make_service()).await?;
        Ok(())
    })
}

fn ensure_static_file_server() -> NapiResult<u16> {
    let _guard = STATIC_SERVER_LOCK
        .lock()
        .map_err(|e| Error::from_reason(format!("failed to lock static file server state: {e}")))?;

    if STATIC_SERVER_STARTED.load(Ordering::SeqCst) {
        return Ok(INTERNAL_FS_PORT.load(Ordering::SeqCst));
    }

    let port = choose_static_port()?;
    INTERNAL_FS_PORT.store(port, Ordering::SeqCst);
    STATIC_SERVER_STARTED.store(true, Ordering::SeqCst);
    std::thread::spawn(move || {
        if let Err(error) = run_static_file_server(port) {
            eprintln!("failed to run internal static file server: {error:?}");
            STATIC_SERVER_STARTED.store(false, Ordering::SeqCst);
        }
    });

    Ok(port)
}

fn write_file_once(content: &[u8], file: &Path) -> NapiResult<()> {
    if !file.exists() {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file)
            .map_err(|e| napi_error("failed to open static resource", e))?;
        file.write_all(content)
            .map_err(|e| napi_error("failed to write static resource", e))?;
    }
    Ok(())
}

#[napi]
pub fn start_static_file_server() -> NapiResult<u16> {
    ensure_static_file_server()
}

#[napi]
pub fn get_dict_list() -> NapiResult<Vec<DictMeta>> {
    let mut dictlist = Vec::new();
    let mut dictdb = lock_dictdb()?;
    dictdb.clear();

    if let Ok(dicts) = wikit::load_client_dictionary() {
        for dict in dicts {
            match &dict {
                WikitDictionary::Local(ld) => {
                    let id = ld.path.display().to_string();
                    dictlist.push(DictMeta {
                        name: ld.head.name.clone(),
                        id: id.clone(),
                    });
                    dictdb.insert(id, dict.clone());
                }
                WikitDictionary::Remote(rd) => {
                    if let Ok(metas) = rd.get_dict_list() {
                        for meta in metas {
                            dictdb.insert(meta.id.clone(), dict.clone());
                            dictlist.push(DictMeta {
                                name: meta.name,
                                id: meta.id,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(dictlist)
}

#[napi]
pub fn lookup(dictid: String, word: String) -> NapiResult<LookupResponse> {
    let mut words = HashMap::new();
    let mut script = String::new();
    let mut style = String::new();
    let dictdb = lock_dictdb()?;

    if let Some(dict) = dictdb.get(&dictid) {
        match dict {
            WikitDictionary::Local(ld) => {
                if let Ok(entries) = ld.lookup(&word) {
                    for (key, value) in entries {
                        words.insert(key, value);
                    }
                }
                script.push_str(ld.get_script());
                style.push_str(ld.get_style());
            }
            WikitDictionary::Remote(rd) => {
                if let Ok(entries) = rd.lookup(&word, &dictid) {
                    for (key, value) in entries {
                        words.insert(key, value);
                    }
                }
                script.push_str(&rd.get_script(&dictid));
                style.push_str(&rd.get_style(&dictid));
            }
        }
    }

    let staticdir = config::get_static_dir().map_err(|e| napi_error("failed to get static directory", e))?;
    let staticid = crypto::md5(dictid.as_bytes());
    let cssfile = staticdir.join(format!("{staticid}.css"));
    let jsfile = staticdir.join(format!("{staticid}.js"));
    write_file_once(style.as_bytes(), cssfile.as_path())?;
    write_file_once(script.as_bytes(), jsfile.as_path())?;
    if let Some(meaning) = words.get(&word) {
        let wordfile = staticdir.join(format!("{staticid}_{word}.html"));
        write_file_once(meaning.as_bytes(), wordfile.as_path())?;
    }

    let port = ensure_static_file_server()?;
    let style = format!(r#" <link rel="stylesheet" href="http://127.0.0.1:{port}/static/{staticid}.css"> "#);
    let script = format!(r#" <script type="text/javascript" src="http://127.0.0.1:{port}/static/{staticid}.js"></script> "#);

    Ok(LookupResponse { words, script, style })
}

#[napi]
pub fn ffi_hello(name: String) -> NapiResult<String> {
    if name.is_empty() {
        Err(Error::from_reason("name is empty"))
    } else {
        Ok(format!("ffi_hello got name: {name}"))
    }
}

#[napi]
pub fn start_preview_server(dir: String) -> NapiResult<()> {
    if PREVIEW_SERVER_STARTED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(());
    }

    let (tx, rx) = tokio::sync::broadcast::channel(1);
    {
        let mut shutdown = lock_preview_shutdown()?;
        *shutdown = Some(tx);
    }

    std::thread::spawn(move || {
        let result = || -> AnyResult<()> {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(5)
                .enable_all()
                .build()?;
            rt.block_on(async move {
                let previewer = preview::Previewer::new(dir).context("failed to create preview server")?;
                Arc::new(previewer).run(rx).await.context("preview server exited with error")
            })
        }();

        if let Err(error) = result {
            eprintln!("preview server exit with error: {error:?}");
        }
        PREVIEW_SERVER_STARTED.store(false, Ordering::SeqCst);
        if let Ok(mut shutdown) = PREVIEW_SHUTDOWN.lock() {
            *shutdown = None;
        }
    });

    Ok(())
}

#[napi]
pub fn stop_preview_server() -> NapiResult<()> {
    if let Some(sender) = lock_preview_shutdown()?.as_ref() {
        let _ = sender.send(());
    }
    PREVIEW_SERVER_STARTED.store(false, Ordering::SeqCst);
    Ok(())
}

#[napi]
pub fn is_preview_server_up() -> NapiResult<bool> {
    Ok(PREVIEW_SERVER_STARTED.load(Ordering::SeqCst))
}

#[napi]
pub fn get_config_dir() -> NapiResult<String> {
    config::get_config_dir()
        .map(|path| path.display().to_string())
        .map_err(|e| napi_error("failed to get config directory", e))
}
