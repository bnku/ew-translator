use lazy_static::lazy_static;
use std::env;
use std::sync::RwLock;

lazy_static! {
    pub static ref TARGET_LANG: RwLock<String> = RwLock::new("ru".to_string());
}

pub fn parse_args() {
    let args = env::args();

    if args.len() > 1 {
        for argument in env::args() {
            *TARGET_LANG.write().unwrap() = argument.to_string();
        }
    }

    println!("Target language is `{}`", *TARGET_LANG.read().unwrap());
}
