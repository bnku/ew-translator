// #![allow(unused)]

mod hotkey;
mod settings;
mod translator;
use notify_rust::Notification;
use std::process::Command;

fn main() {
    settings::parse_args();
    hotkey_listener();
}

fn hotkey_listener() {
    let mut hk = hotkey::Listener::new();
    hk.register_hotkey(
        0, // hotkey::modifiers::CONTROL, // | hotkey::modifiers::SHIFT,
        hotkey::keys::F7,
        || hotkey_handle(),
    )
    .unwrap();
    hk.listen();
}

fn hotkey_handle() {
    let text = get_clipboard();
    // dbg!(&text);

    let translation = match translator::google(text) {
        Ok(v) => v,
        Err(e) => e.to_string(),
    };
    // println!("{}", &translation);
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
