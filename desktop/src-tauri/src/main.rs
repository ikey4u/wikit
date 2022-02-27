#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::{sync::Mutex, collections::HashMap};
use std::path::Path;
use std::sync::Arc;
use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicU16, Ordering};

use wikit_core::config;
use wikit_core::crypto;
use wikit_core::wikit;
use wikit_core::util;
use wikit_core::wikit::WikitDictionary;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, RunEvent, Manager};
use tauri::api::dialog;
use once_cell::sync::Lazy;

static DICTDB: Lazy<Arc<Mutex<HashMap<String, WikitDictionary>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

// internal static file server port
static INTERNAL_FS_PORT: AtomicU16 = AtomicU16::new(7561);

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
fn lookup(dictid: String, word: String) -> LookupResponse {
    let (mut mp, mut script, mut style) = (HashMap::new(), String::new(), String::new());

    let dictdb = DICTDB.lock().unwrap();
    if let Some(dict) = dictdb.get(&dictid) {
        match dict {
            wikit::WikitDictionary::Local(ld) => {
                if let Ok(v) = ld.lookup(word) {
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

    let port = INTERNAL_FS_PORT.load(Ordering::SeqCst);

    let csstag = format!(r#" <link rel="stylesheet" href="http://127.0.0.1:{port}/static/{staticid}.css"> "#);
    let jstag = format!(r#" <script type="text/javascript" src="http://127.0.0.1:{port}/static/{staticid}.js"></script> "#);
    LookupResponse::new(mp, jstag.to_string(), csstag.to_string())
}

#[tauri::command]
fn get_dict_list() -> Vec<wikit::DictList> {
    let mut dictlist = vec![];
    let mut dictdb = DICTDB.lock().unwrap();
    if let Ok(dicts) = wikit::load_client_dictionary() {
        for dict in dicts {
            match dict {
                wikit::WikitDictionary::Local(ref ld) => {
                    let id = format!("{}", ld.path.display());
                    dictlist.push(wikit::DictList{ name: ld.head.name.clone(), id: id.clone() });
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

    let port = util::get_free_tcp_port(Some(INTERNAL_FS_PORT.load(Ordering::SeqCst)))
        .context("failed to get static file sever port")?;
    println!("[+] Internal file server listens at 127.0.0.1:{port}");
    INTERNAL_FS_PORT.store(port, Ordering::SeqCst);

    let addr = SocketAddr::from(([127, 0, 0, 1], INTERNAL_FS_PORT.load(Ordering::SeqCst)));
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tokio::spawn(async {
        http_file_server().await.expect("failed to run internal static file server");
    });

    let app = tauri::Builder::default()
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
        .invoke_handler(tauri::generate_handler![lookup, get_dict_list])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, e| match e {
        // Application is ready (triggered only once)
        RunEvent::Ready => {
        },
        RunEvent::CloseRequested { label, api, .. } => {
            let app_handle = app_handle.clone();
            let window = app_handle.get_window(&label).unwrap();
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
            .add_item(CustomMenuItem::new("about".to_string(), "About Wikit Desktop"))
            .add_item(CustomMenuItem::new("open_config_dir".to_string(), "Open Configuration Directory"))
            .add_item(CustomMenuItem::new("close".to_string(), "Close and Exit"))
    );
    let editmenu = Submenu::new("Edit",
        Menu::new()
            .add_native_item(MenuItem::Copy)
            .add_native_item(MenuItem::Cut)
            .add_native_item(MenuItem::Paste)
            .add_native_item(MenuItem::Undo)
            .add_native_item(MenuItem::Redo)
            .add_native_item(MenuItem::SelectAll)
    );
    let about_menu = Submenu::new("Help",
        Menu::new()
            .add_item(CustomMenuItem::new("feedback".to_string(), "Feedback"))
            .add_item(CustomMenuItem::new("manual".to_string(), "Manual"))
    );

    Menu::new()
        .add_submenu(filemenu)
        .add_submenu(editmenu)
        .add_submenu(about_menu)
}
