#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use tauri::{CustomMenuItem, Menu, MenuItem, Submenu};
use tauri::api::dialog;

fn main() {
    let open = CustomMenuItem::new("open".to_string(), "Open");
    let close = CustomMenuItem::new("close".to_string(), "Close");
    let file_submenu = Submenu::new("File", Menu::new().add_item(open).add_item(close));

    let about = CustomMenuItem::new("about".to_string(), "About");
    let about_submenu = Submenu::new("Help", Menu::new().add_item(about));

    let menu = Menu::new()
        .add_native_item(MenuItem::Copy)
        .add_item(CustomMenuItem::new("hide", "Hide"))
        .add_submenu(file_submenu)
        .add_submenu(about_submenu);
    tauri::Builder::default()
        .menu(menu)
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
                _ => {}
           }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
