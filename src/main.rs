mod gui;
mod settings;
mod translator;

use std::{process::Command, sync::Arc};
use tauri::GlobalShortcutManager;
use translator::Translator;

fn main() {
    // Parse configuration before Tauri initializes GTK so `--help` and
    // configuration errors also work in headless shells.
    let settings = settings::load().unwrap_or_else(|error| {
        eprintln!("Configuration error: {error}");
        std::process::exit(2);
    });

    tauri::Builder::default()
        .setup(move |app| {
            let translator = Arc::new(Translator::new(settings.translation.clone())?);
            let app_handle = app.handle();

            gui::create_window(&app_handle, settings::WINDOW_LABEL, "index.html");
            gui::hide_window(&app_handle);

            let shortcut_handle = app_handle.clone();
            let target_lang = settings.target_lang.clone();
            app_handle
                .global_shortcut_manager()
                .register(&settings.hotkeys, move || {
                    gui::show_window(
                        &shortcut_handle,
                        Arc::clone(&translator),
                        target_lang.clone(),
                    )
                })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub(crate) fn translate(translator: &Translator, target_lang: &str) -> String {
    let result = get_selection().and_then(|text| {
        translator
            .translate(&text, target_lang)
            .map_err(|error| error.to_string())
    });

    match result {
        Ok(translation) => translation,
        Err(error) => {
            eprintln!("Translation failed: {error}");
            error
        }
    }
}

fn get_selection() -> Result<String, String> {
    let output = Command::new("xsel")
        .arg("-o")
        .output()
        .map_err(|error| format!("Cannot run xsel: {error}"))?;
    if !output.status.success() {
        return Err(format!("xsel exited with status {}", output.status));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("xsel returned invalid UTF-8: {error}"))
}
