/*
Fork of hotkey = "0.3.1"
https://crates.io/crates/hotkey
MIT License
*/

// #![allow(unused)]
use std::collections::HashMap;
use std::mem;
use std::os::raw::c_ulong;
use std::ptr;
use x11_dl::xlib;

pub mod modifiers {
    use x11_dl::xlib;
    pub const ALT: u32 = xlib::Mod1Mask;
    pub const CTRL: u32 = xlib::ControlMask;
    pub const SHIFT: u32 = xlib::ShiftMask;
    pub const SUPER: u32 = xlib::Mod4Mask;
}

pub mod keys {
    use x11_dl::keysym;
    pub const BACKSPACE: u32 = keysym::XK_BackSpace;
    pub const TAB: u32 = keysym::XK_Tab;
    pub const ENTER: u32 = keysym::XK_Return;
    pub const CAPS_LOCK: u32 = keysym::XK_Caps_Lock;
    pub const ESCAPE: u32 = keysym::XK_Escape;
    pub const SPACEBAR: u32 = keysym::XK_space;
    pub const PAGE_UP: u32 = keysym::XK_Page_Up;
    pub const PAGE_DOWN: u32 = keysym::XK_Page_Down;
    pub const END: u32 = keysym::XK_End;
    pub const HOME: u32 = keysym::XK_Home;
    pub const ARROW_LEFT: u32 = keysym::XK_Left;
    pub const ARROW_RIGHT: u32 = keysym::XK_Right;
    pub const ARROW_UP: u32 = keysym::XK_Up;
    pub const ARROW_DOWN: u32 = keysym::XK_Down;
    pub const PRINT_SCREEN: u32 = keysym::XK_Print;
    pub const INSERT: u32 = keysym::XK_Insert;
    pub const DELETE: u32 = keysym::XK_Delete;
    pub const XK_0: u32 = keysym::XK_0;
    pub const XK_1: u32 = keysym::XK_1;
    pub const XK_2: u32 = keysym::XK_2;
    pub const XK_3: u32 = keysym::XK_3;
    pub const XK_4: u32 = keysym::XK_4;
    pub const XK_5: u32 = keysym::XK_5;
    pub const XK_6: u32 = keysym::XK_6;
    pub const XK_7: u32 = keysym::XK_7;
    pub const XK_8: u32 = keysym::XK_8;
    pub const XK_9: u32 = keysym::XK_9;
    pub const A: u32 = keysym::XK_A;
    pub const B: u32 = keysym::XK_B;
    pub const C: u32 = keysym::XK_C;
    pub const D: u32 = keysym::XK_D;
    pub const E: u32 = keysym::XK_E;
    pub const F: u32 = keysym::XK_F;
    pub const G: u32 = keysym::XK_G;
    pub const H: u32 = keysym::XK_H;
    pub const I: u32 = keysym::XK_I;
    pub const J: u32 = keysym::XK_J;
    pub const K: u32 = keysym::XK_K;
    pub const L: u32 = keysym::XK_L;
    pub const M: u32 = keysym::XK_M;
    pub const N: u32 = keysym::XK_N;
    pub const O: u32 = keysym::XK_O;
    pub const P: u32 = keysym::XK_P;
    pub const Q: u32 = keysym::XK_Q;
    pub const R: u32 = keysym::XK_R;
    pub const S: u32 = keysym::XK_S;
    pub const T: u32 = keysym::XK_T;
    pub const U: u32 = keysym::XK_U;
    pub const V: u32 = keysym::XK_V;
    pub const W: u32 = keysym::XK_W;
    pub const X: u32 = keysym::XK_X;
    pub const Y: u32 = keysym::XK_Y;
    pub const Z: u32 = keysym::XK_Z;
    pub const F1: u32 = keysym::XK_F1;
    pub const F2: u32 = keysym::XK_F2;
    pub const F3: u32 = keysym::XK_F3;
    pub const F4: u32 = keysym::XK_F4;
    pub const F5: u32 = keysym::XK_F5;
    pub const F6: u32 = keysym::XK_F6;
    pub const F7: u32 = keysym::XK_F7;
    pub const F8: u32 = keysym::XK_F8;
    pub const F9: u32 = keysym::XK_F9;
    pub const F10: u32 = keysym::XK_F10;
    pub const F11: u32 = keysym::XK_F11;
    pub const F12: u32 = keysym::XK_F12;
}

pub fn get_key(key: &str) -> (u8, u32) {
    match key {
        "ALT" => (1, modifiers::ALT),
        "CTRL" => (1, modifiers::CTRL),
        "SHIFT" => (1, modifiers::SHIFT),
        "SUPER" => (1, modifiers::SUPER),
        //keys
        "BACKSPACE" => (2, keys::BACKSPACE),
        "TAB" => (2, keys::TAB),
        "ENTER" => (2, keys::ENTER),
        "CAPS_LOCK" => (2, keys::CAPS_LOCK),
        "ESCAPE" => (2, keys::ESCAPE),
        "SPACEBAR" => (2, keys::SPACEBAR),
        "PAGE_UP" => (2, keys::PAGE_UP),
        "PAGE_DOWN" => (2, keys::PAGE_DOWN),
        "END" => (2, keys::END),
        "HOME" => (2, keys::HOME),
        "ARROW_LEFT" => (2, keys::ARROW_LEFT),
        "ARROW_RIGHT" => (2, keys::ARROW_RIGHT),
        "ARROW_UP" => (2, keys::ARROW_UP),
        "ARROW_DOWN" => (2, keys::ARROW_DOWN),
        "PRINT_SCREEN" => (2, keys::PRINT_SCREEN),
        "INSERT" => (2, keys::INSERT),
        "DELETE" => (2, keys::DELETE),
        "XK_0" => (2, keys::XK_0),
        "XK_1" => (2, keys::XK_1),
        "XK_2" => (2, keys::XK_2),
        "XK_3" => (2, keys::XK_3),
        "XK_4" => (2, keys::XK_4),
        "XK_5" => (2, keys::XK_5),
        "XK_6" => (2, keys::XK_6),
        "XK_7" => (2, keys::XK_7),
        "XK_8" => (2, keys::XK_8),
        "XK_9" => (2, keys::XK_9),
        "A" => (2, keys::A),
        "B" => (2, keys::B),
        "C" => (2, keys::C),
        "D" => (2, keys::D),
        "E" => (2, keys::E),
        "F" => (2, keys::F),
        "G" => (2, keys::G),
        "H" => (2, keys::H),
        "I" => (2, keys::I),
        "J" => (2, keys::J),
        "K" => (2, keys::K),
        "L" => (2, keys::L),
        "M" => (2, keys::M),
        "N" => (2, keys::N),
        "O" => (2, keys::O),
        "P" => (2, keys::P),
        "Q" => (2, keys::Q),
        "R" => (2, keys::R),
        "S" => (2, keys::S),
        "T" => (2, keys::T),
        "U" => (2, keys::U),
        "V" => (2, keys::V),
        "W" => (2, keys::W),
        "X" => (2, keys::X),
        "Y" => (2, keys::Y),
        "Z" => (2, keys::Z),
        "F1" => (2, keys::F1),
        "F2" => (2, keys::F2),
        "F3" => (2, keys::F3),
        "F4" => (2, keys::F4),
        "F5" => (2, keys::F5),
        "F6" => (2, keys::F6),
        "F7" => (2, keys::F7),
        "F8" => (2, keys::F8),
        "F9" => (2, keys::F9),
        "F10" => (2, keys::F10),
        "F11" => (2, keys::F11),
        "F12" => (2, keys::F12),
        _ => (0, 0),
    }
}

pub type ListenerID = (i32, u32);

pub struct Listener {
    display: *mut xlib::Display,
    root: c_ulong,
    xlib: xlib::Xlib,
    handlers: HashMap<ListenerID, Box<dyn Fn()>>,
}

impl Listener {
    pub fn new() -> Listener {
        let xlib = xlib::Xlib::open().unwrap();
        unsafe {
            let display = (xlib.XOpenDisplay)(ptr::null());

            // Only trigger key release at end of repeated keys
            let mut supported_rtrn: i32 = mem::MaybeUninit::uninit().assume_init();
            (xlib.XkbSetDetectableAutoRepeat)(display, 1, &mut supported_rtrn);

            Listener {
                display: display,
                root: (xlib.XDefaultRootWindow)(display),
                xlib,
                handlers: HashMap::new(),
            }
        }
    }

    pub fn register_hotkey<CB: 'static + Fn()>(
        &mut self,
        modifiers: u32,
        key: u32,
        handler: CB,
    ) -> Result<ListenerID, String> {
        unsafe {
            let keycode = (self.xlib.XKeysymToKeycode)(self.display, key as u64) as i32;
            let result = (self.xlib.XGrabKey)(
                self.display,
                keycode,
                modifiers,
                self.root,
                0,
                xlib::GrabModeAsync,
                xlib::GrabModeAsync,
            );

            if result == 0 {
                return Err("Failed to register hotkey".to_string());
            }

            let id = (keycode, modifiers);
            self.handlers.insert(id, Box::new(handler));
            Ok(id)
        }
    }

    pub fn listen(self) {
        unsafe {
            (self.xlib.XSelectInput)(self.display, self.root, xlib::KeyReleaseMask);
            let mut event: xlib::XEvent = mem::MaybeUninit::uninit().assume_init();
            // let mut s_event: xlib::XEvent = mem::MaybeUninit::uninit().assume_init();
            loop {
                (self.xlib.XNextEvent)(self.display, &mut event);
                match event.get_type() {
                    xlib::KeyRelease => {
                        if let Some(handler) = self
                            .handlers
                            .get(&(event.key.keycode as i32, event.key.state))
                        {
                            handler();
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
