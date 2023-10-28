mod hotkey;
mod settings;
mod translator;
use notify_rust::Notification;
use settings::HOTKEYS;
use std::process::Command;

fn main() {
    settings::define();
    add_hotkey_listener();
}

fn add_hotkey_listener() {
    let hk = hotkey::Listener::new(&HOTKEYS.read().unwrap());
    hk.listen();
}

fn hotkey_handle() {
    let text = get_clipboard();
    let translation = match translator::google(text) {
        Ok(v) => v,
        Err(e) => e.to_string(),
    };
    send_notification(translation);
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

fn send_notification(text: String) {
    match Notification::new()
        .summary("Translation")
        .body(text.as_str())
        .icon("locale")
        .show()
    {
        Err(e) => panic!("Notification send error: {}", e),
        _ => (),
    }
}
