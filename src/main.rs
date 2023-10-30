mod gui;
mod settings;
mod translator;
use settings::{HOTKEYS, WINDOW_LABEL};
use std::process::Command;
use tauri::GlobalShortcutManager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            settings::define();

            let app_handle = app.handle(); //.clone();
            gui::create_window(&app_handle, &WINDOW_LABEL.read().unwrap(), "index.html");
            gui::hide_window(&app_handle);

            app_handle
                .global_shortcut_manager()
                .register(&HOTKEYS.read().unwrap(), move || {
                    gui::show_window(&app_handle)
                })
                .unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn translate() -> String {
    let text = get_clipboard();
    let translation = match translator::google(text) {
        Ok(v) => v,
        Err(e) => e.to_string(),
    };
    translation
}

fn get_clipboard() -> String {
    let output = Command::new("xsel")
        .arg("-o")
        .output()
        .expect("Not found binary: xsel");
    let text = match std::str::from_utf8(&output.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    String::from(text)
}
