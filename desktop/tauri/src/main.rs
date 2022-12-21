#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::{sync::Mutex, collections::HashMap};
use std::path::Path;
use std::sync::Arc;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicU16, Ordering};

use wikit_core::config;
use wikit_core::crypto;
use wikit_core::wikit;
use wikit_core::util;
use wikit_core::preview;
use wikit_core::wikit::WikitDictionary;
use wikit_proto::DictMeta;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, RunEvent, WindowEvent, Manager};
use tauri::api::dialog;
use once_cell::sync::Lazy;
use anyhow::{Context, Result};
use tokio::sync::broadcast::{self, Sender};

static DICTDB: Lazy<Arc<Mutex<HashMap<String, WikitDictionary>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
// internal static file server port
static INTERNAL_FS_PORT: AtomicU16 = AtomicU16::new(7561);
pub type FFIResult<T> = Result<T, String>;

struct WikitState {
    shutdown_previewer: Sender<()>,
    is_previewer_started: Arc<Mutex<bool>>,
}

impl WikitState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1);
        Self {
            shutdown_previewer: tx,
            is_previewer_started: Arc::new(Mutex::new(false)),
        }
    }

    fn is_previewer_started(&self) -> bool {
        if let Ok(started) = self.is_previewer_started.lock() {
            *started
        } else {
            false
        }
    }

    fn mark_previewer_started(&self) {
        if let Ok(mut v) = self.is_previewer_started.lock() {
            *v = true
        }
    }

    fn mark_previewer_stopped(&self) {
        if let Ok(mut v) = self.is_previewer_started.lock() {
            *v = false
        }
    }

    fn stop_previewer_jobs(&self) {
        if let Ok(_) = self.shutdown_previewer.send(()) {
            self.mark_previewer_stopped()
        }
    }

    async fn start_previewer(&self, dir: String) -> Result<(), String> {
        if self.is_previewer_started() {
            return Ok(())
        }

        let rx = self.shutdown_previewer.subscribe();
        let previewer = preview::Previewer::new(dir)?;

        self.mark_previewer_started();
        if let Err(e) = Arc::new(previewer).run(rx).await {
            self.stop_previewer_jobs();
            println!("previewer server exit with error: {e:?}");
            return Err(format!("{e:?}"));
        };

        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LookupResponse {
    // possible of (word, meaning) pair list
    words: HashMap<String, String>,
    // html js tag
    script: String,
    // html css tag
    style: String,
}

impl LookupResponse {
    fn new(words: HashMap<String, String>, script: String, style: String) -> Self {
        LookupResponse {
            words,
            script,
            style,
        }
    }
}

#[tauri::command]
async fn start_preview_server(state: tauri::State<'_, WikitState>, dir: String) -> FFIResult<()> {
    state.start_previewer(dir).await
}

#[tauri::command]
async fn stop_preview_server(state: tauri::State<'_, WikitState>) -> FFIResult<()> {
    state.stop_previewer_jobs();
    Ok(())
}

#[tauri::command]
async fn is_preview_server_up(state: tauri::State<'_, WikitState>) -> FFIResult<bool> {
    Ok(state.is_previewer_started())
}

#[tauri::command]
fn ffi_hello(name: String) -> FFIResult<String> {
    if name.len() == 0 {
        Err("name is empty".into())
    } else {
        Ok(format!("ffi_hello got name: {}", name))
    }
}

#[tauri::command]
fn lookup(dictid: String, word: String) -> LookupResponse {
    let (mut mp, mut script, mut style) = (HashMap::new(), String::new(), String::new());

    let dictdb = DICTDB.lock().unwrap();
    if let Some(dict) = dictdb.get(&dictid) {
        match dict {
            wikit::WikitDictionary::Local(ld) => {
                if let Ok(v) = ld.lookup(&word) {
                    for (k, v) in v {
                        mp.insert(k, v);
                    }
                }
                script.push_str(ld.get_script());
                style.push_str(ld.get_style());
            },
            wikit::WikitDictionary::Remote(rd) => {
                if let Ok(v) = rd.lookup(&word, &dictid) {
                    for (k, v) in v {
                        mp.insert(k, v);
                    }
                }
                script.push_str(&rd.get_script(&dictid));
                style.push_str(&rd.get_style(&dictid));
            },
        }
    }

    let staticdir = config::get_static_dir().unwrap();
    let staticid = crypto::md5(dictid.as_bytes());
    let cssfile = staticdir.join(format!("{staticid}.css"));
    let jsfile = staticdir.join(format!("{staticid}.js"));
    let write_file = |content: &[u8], file: &Path| {
        if !file.exists() {
            let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(file).unwrap();
            file.write(content).unwrap();
        }
    };
    write_file(style.as_bytes(), cssfile.as_path());
    write_file(script.as_bytes(), jsfile.as_path());
    if let Some(v) = mp.get(&word) {
        let wordfile = staticdir.join(format!("{staticid}_{word}.html"));
        write_file(v.as_bytes(), wordfile.as_path());
    }

    let port = INTERNAL_FS_PORT.load(Ordering::SeqCst);

    let csstag = format!(r#" <link rel="stylesheet" href="http://127.0.0.1:{port}/static/{staticid}.css"> "#);
    let jstag = format!(r#" <script type="text/javascript" src="http://127.0.0.1:{port}/static/{staticid}.js"></script> "#);
    LookupResponse::new(mp, jstag.to_string(), csstag.to_string())
}

#[tauri::command]
fn get_dict_list() -> Vec<DictMeta> {
    let mut dictlist = vec![];
    let mut dictdb = DICTDB.lock().unwrap();
    if let Ok(dicts) = wikit::load_client_dictionary() {
        for dict in dicts {
            match dict {
                wikit::WikitDictionary::Local(ref ld) => {
                    let id = format!("{}", ld.path.display());
                    dictlist.push(DictMeta { name: ld.head.name.clone(), id: id.clone() });
                    dictdb.insert(id.clone(), dict);
                },
                wikit::WikitDictionary::Remote(ref rd) => {
                    if let Ok(ds) = rd.get_dict_list() {
                        for d in ds {
                            dictdb.insert(d.id.clone(), dict.clone());
                            dictlist.push(d);
                        }
                    } else {
                        println!("failed to get remote dictionary list");
                    }
                },
            }
        }
    }
    dictlist
}

// Copied and modified from `https://github.com/tokio-rs/axum/tree/main/examples/static-file-server`
async fn http_file_server() -> Result<()> {
    use axum::{http::StatusCode, routing::get_service, Router};
    use std::net::SocketAddr;
    use tower_http::services::ServeDir;

    let app = Router::new().nest(
        "/static",
        get_service(ServeDir::new(config::get_static_dir()?)).handle_error(|error: std::io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            )
        }),
    );

    // Restricted ports are not allowed in webkit, see https://chromium.googlesource.com/chromium/src.git/+/refs/heads/master/net/base/port_util.cc#27
    let restricted_ports = [6000, 6566, 6665, 6666, 6667, 6668, 6669, 6697u16];
    let port = loop {
        let port = util::get_free_tcp_port(Some(INTERNAL_FS_PORT.load(Ordering::SeqCst)))
            .context("failed to get static file sever port")?;
        if !restricted_ports.contains(&port) {
            break port;
        }
    };
    println!("[+] Internal file server listens at 127.0.0.1:{port}");
    INTERNAL_FS_PORT.store(port, Ordering::SeqCst);

    let addr = SocketAddr::from(([127, 0, 0, 1], INTERNAL_FS_PORT.load(Ordering::SeqCst)));
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

fn async_main() -> Result<()> {
    use tokio::runtime::Builder;

    let rt = Builder::new_multi_thread()
        .worker_threads(5)
        .enable_all()
        .build()?;
    rt.block_on(async {
        http_file_server().await.expect("failed to run internal static file server");
    });

    Ok(())
}

fn main() -> Result<()> {
    std::thread::spawn(move || {
        async_main().expect("start async runtime failed");
    });

    let app = tauri::Builder::default()
        .manage(WikitState::new())
        .menu(get_menu())
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            let window_ = window.clone();
            window.on_menu_event(move |event| {
              match event.menu_item_id() {
                "close" => {
                    println!("click close menu");
                    std::process::exit(0);
                },
                "open" => {
                    dialog::FileDialogBuilder::default().pick_file(|path| {
                        if let Some(path) = path {
                            println!("open file {}", path.display());
                        } else {
                            println!("open nothing");
                        }
                    });
                },
                "open_config_dir" => {
                    if let Ok(d) = config::get_config_dir() {
                        let _ = opener::open(d);
                    }
                },
                "about" => {
                    dialog::message(
                        Some(&window_),
                        "Wikit Desktop",
                        &format!("A universal dictionary\nv{}\nhttps://github.com/ikey4u/wikit", VERSION),
                    );
                },
                "homepage" => {
                    let _ = opener::open("https://github.com/ikey4u/wikit");
                },
                "feedback" => {
                    let _ = opener::open("https://github.com/ikey4u/wikit/issues/new");
                },
                "manual" => {
                    let _ = opener::open("https://github.com/ikey4u/wikit/wiki");
                },
                id @ _ => {
                    println!("unkown menu event: {}", id);
                }
              }
            });
            Ok(())
        })
        .on_page_load(|window, _| {
          let window_ = window.clone();
          window.listen("js-event", move |event| {
              println!("got js-event with message '{:?}'", event.payload());
              let reply = "something else".to_string();
              window_.emit("rust-event", Some(reply)).expect("failed to emit");
          });
        })
        .invoke_handler(tauri::generate_handler![
            lookup,
            get_dict_list,
            ffi_hello,
            start_preview_server,
            stop_preview_server,
            is_preview_server_up,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, e| match e {
        // Application is ready (triggered only once)
        RunEvent::Ready => {
        },
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } => {
            if label == "main" {
                let app_handle = app_handle.clone();
                let window = app_handle.get_window("main").unwrap();
                // prevent the event loop to close
                api.prevent_close();
                dialog::ask(
                    Some(&window),
                    "Wikit Desktop",
                    "Are you sure that you want to exit wikit desktop?",
                    move |answer| {
                        if answer {
                            // .close() cannot be called on the main thread
                            std::thread::spawn(move || {
                                std::process::exit(0);
                            });
                        }
                    },
                );
            }
        },
        RunEvent::ExitRequested { api, .. } => {
            // Keep the event loop running even if all windows are closed
            // This allow us to catch system tray events when there is no window
            api.prevent_exit();
        },
        _ => {}
    });

    Ok(())
}

fn get_menu() -> Menu {
    let filemenu = Submenu::new("File",
        Menu::new()
            .add_item(CustomMenuItem::new("open_config_dir".to_string(), "Configuration"))
            .add_item(CustomMenuItem::new("close".to_string(), "Quit"))
    );
    let menu = Menu::new().add_submenu(filemenu);

    let editmenu = Submenu::new("Edit",
        Menu::new()
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Cut)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::Undo)
            .add_native_item(MenuItem::Redo)
            .add_native_item(MenuItem::SelectAll)
    );
    // edit menu is not supported on linux
    let menu = if cfg!(not(target_os = "linux")) {
        menu.add_submenu(editmenu)
    } else {
        menu
    };

    let about_menu = Submenu::new("Help",
        Menu::new()
            .add_item(CustomMenuItem::new("homepage".to_string(), "Home Page"))
            .add_item(CustomMenuItem::new("feedback".to_string(), "Report Bug"))
            .add_item(CustomMenuItem::new("manual".to_string(), "Manual"))
            .add_item(CustomMenuItem::new("about".to_string(), "About"))
    );
    let menu = menu.add_submenu(about_menu);

    menu
}
