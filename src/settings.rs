use clap::Parser;
use lazy_static::lazy_static;
use std::sync::RwLock;

pub mod default_settings {
    pub const LANG: &str = "ru";
    pub const HOTKEY: &str = "CTRL+SHIFT+F7";
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Target language
    #[clap(short, long, default_value_t = default_settings::LANG.to_string())]
    lang: String,

    /// Hotkeys (modifier+key).
    /// Modifiers: ALT, CTRL, SHIFT, SUPER;
    /// Keys: 0-9, A-Z, F1-F12, BACKSPACE, TAB, ENTER, CAPS_LOCK, ESCAPE, SPACEBAR, PAGE_UP, PAGE_DOWN, END, HOME, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, ARROW_DOWN, PRINT_SCREEN, INSERT, DELETE
    #[clap(short, long, default_value_t = default_settings::HOTKEY.to_string())]
    hotkeys: String,
}

lazy_static! {
    pub static ref TARGET_LANG: RwLock<String> = RwLock::new(default_settings::LANG.to_string());
    pub static ref HOTKEYS: RwLock<String> = RwLock::new(default_settings::HOTKEY.to_string());
}

pub fn define() {
    let args = Args::parse();
    set_hotkeys(args.hotkeys);
    set_lang(args.lang);
}

fn set_lang(lang: String) {
    *TARGET_LANG.write().unwrap() = lang;
    println!("Target language is `{}`", *TARGET_LANG.read().unwrap());
}

fn set_hotkeys(hotkey: String) {
    *HOTKEYS.write().unwrap() = hotkey;
    println!("Hotkeys is `{}`", *HOTKEYS.read().unwrap());
}
