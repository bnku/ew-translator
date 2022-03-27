use lazy_static::lazy_static;
use std::sync::RwLock;

use super::hotkey;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Target language
    #[clap(short, long, default_value_t = String::from("ru"))]
    lang: String,

    /// Hotkeys (modifier+key).
    /// Modifiers: ALT, CTRL, SHIFT, SUPER;
    /// Keys: 0-9, A-Z, F1-F12, BACKSPACE, TAB, ENTER, CAPS_LOCK, ESCAPE, SPACEBAR, PAGE_UP, PAGE_DOWN, END, HOME, ARROW_LEFT, ARROW_RIGHT, ARROW_UP, ARROW_DOWN, PRINT_SCREEN, INSERT, DELETE
    #[clap(short, long, default_value_t = String::from("CTRL+SHIFT+Z"))]
    hotkeys: String,
}

lazy_static! {
    pub static ref TARGET_LANG: RwLock<String> = RwLock::new("ru".to_string());
    pub static ref HK_MOD: RwLock<Vec<u32>> = RwLock::new(vec![]);
    pub static ref HK_KEY: RwLock<Vec<u32>> = RwLock::new(vec![]);
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

fn set_hotkeys(keys: String) {
    for s in keys.split('+') {
        let key = hotkey::get_key(&s.to_uppercase());
        match key.0 {
            1 => HK_MOD.write().unwrap().push(key.1),
            2 => HK_KEY.write().unwrap().push(key.1),
            _ => (),
        }
    }
}
