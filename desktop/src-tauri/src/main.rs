#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::{sync::Mutex, collections::HashMap};
use std::path::Path;
use std::sync::Arc;

use wikit_core::config;
use wikit_core::wikit;
use wikit_core::wikit::WikitDictionary;
use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, Event, Manager};
use tauri::api::dialog;
use once_cell::sync::Lazy;

static DICTDB: Lazy<Arc<Mutex<HashMap<String, WikitDictionary>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

#[derive(serde::Serialize, serde::Deserialize)]
struct LookupResponse {
    // possible of (word, meaning) pair list
    words: HashMap<String, String>,
    script: String,
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
    LookupResponse::new(mp, script, style)
}

#[tauri::command]
fn get_dict_list() -> Vec<String> {
    let mut dictdb = DICTDB.lock().unwrap();

    if let Ok(dicts) = wikit::load_client_dictionary() {
        for dict in dicts {
            match dict {
                wikit::WikitDictionary::Local(ref ld) => {
                    let id = format!("{}", ld.path.display());
                    dictdb.insert(id.clone(), dict);
                },
                wikit::WikitDictionary::Remote(ref rd) => {
                    if let Ok(ds) = rd.get_dict_list() {
                        for d in ds {
                            dictdb.insert(d.id.clone(), dict.clone());
                        }
                    }
                },
            }
        }
    }

    let mut dictlist = vec![];
    for (k, _) in dictdb.iter() {
        dictlist.push(k.to_string());
    }

    dictlist
}

fn main() {
    let app = tauri::Builder::default()
        .menu(get_menu())
        .on_menu_event(|event| {
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
                "about" => {
                    println!("click about menu");
                },
                id @ _ => {
                    println!("unkown menu event: {}", id);
                }
           }
        })
        .invoke_handler(tauri::generate_handler![lookup, get_dict_list])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    app.run(|app_handle, e| match e {
        // Application is ready (triggered only once)
        Event::Ready => {
        },
        Event::CloseRequested { label, api, .. } => {
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
        Event::ExitRequested { api, .. } => {
            // Keep the event loop running even if all windows are closed
            // This allow us to catch system tray events when there is no window
            api.prevent_exit();
        },
        _ => {}
    })
}

fn get_menu() -> Menu {
    let filemenu = Submenu::new("File",
        Menu::new()
            .add_item(CustomMenuItem::new("open".to_string(), "Open"))
            .add_item(CustomMenuItem::new("close".to_string(), "Close"))
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
            .add_item(CustomMenuItem::new("about".to_string(), "About"))
    );

    Menu::new()
        .add_submenu(filemenu)
        .add_submenu(editmenu)
        .add_submenu(about_menu)
}
