use crate::hotkey_handle;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::{hotkey::HotKey, GlobalHotKeyManager};
use std::str::FromStr;

pub struct Listener {
    hotkey: HotKey,
}

impl Listener {
    pub fn new(str_: &str) -> Listener {
        let hotkey = HotKey::from_str(str_).unwrap();
        Listener { hotkey }
    }

    pub fn listen(self) {
        let manager = GlobalHotKeyManager::new().unwrap();
        let _ = manager.register(self.hotkey);
        loop {
            if let Ok(_) = GlobalHotKeyEvent::receiver().try_recv() {
                hotkey_handle()
            }
        }
    }
}
