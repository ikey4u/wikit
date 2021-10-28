#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::io::Read;

use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, Event, Manager};
use tauri::api::dialog;

#[derive(serde::Serialize, serde::Deserialize)]
struct Dictionary {
    // standard name
    stdname: String,
    // human readable name
    name: String,
    // abbreviation name
    abbr: String,
    // dictionary path
    path: String,
    csspath: String,
    jspath: String,
}

#[tauri::command]
fn lookup(dict: Dictionary, word: String) -> String {
    let mut meaning = String::new();
    if dict.path.starts_with("http://") || dict.path.starts_with("https://") {
        let url = format!("{}/dict/q?word={}&dictname={}", dict.path, word, dict.stdname);
        let mut resp = reqwest::blocking::get(url).unwrap();
        resp.read_to_string(&mut meaning);
    } else {
        meaning.push_str("local dictionary is not unsupported for now");
    }
    meaning
}

#[tauri::command]
fn get_dict_list() -> Vec<Dictionary> {
    // fixed for test, will be changed once development done
    let srv = "http://106.53.152.194";
    vec![
        Dictionary {
            stdname: "en_en_oxford_advanced".into(),
            name: "Oxford Advanced Learner Dictionary".into(),
            abbr: "OALD".into(),
            path: srv.to_string(),
            jspath: format!("{}/{}", srv, "/wikit/static/en_en_oxford_advanced.js"),
            csspath: format!("{}/{}", srv, "/wikit/static/en_en_oxford_advanced.css"),
        },
    ]
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
                "Wikit App",
                "Are you sure that you want to close this window?",
                move |answer| {
                    if answer {
                        // .close() cannot be called on the main thread
                        std::thread::spawn(move || {
                            app_handle.get_window(&label).unwrap().close().unwrap();
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
