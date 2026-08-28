// use std::sync::Mutex;

use mouse_position::mouse_position::Mouse;
use tauri::window::WindowBuilder;
use tauri::{AppHandle, Manager, PhysicalPosition, Position, Window, WindowUrl};

use crate::settings::WINDOW_LABEL;
use crate::translate;

pub fn create_window(handler: &AppHandle, label: &str, url: &str) {
    WindowBuilder::new(handler, label, WindowUrl::App(url.into()))
        .title(label)
        .inner_size(1000.0, 40.0)
        // .resizable(true)
        .fullscreen(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .build()
        .ok();
}

pub fn hide_window(handler: &AppHandle) {
    let window = handler.get_window(&WINDOW_LABEL.read().unwrap()).unwrap();
    window.hide().unwrap();
}

pub fn set_window_position(window: &Window) {
    let mouse_position = Mouse::get_mouse_position();
    match mouse_position {
        Mouse::Position { x, y } => {
            let _ = window.set_position(Position::Physical(PhysicalPosition { x, y }));
        }
        Mouse::Error => println!("Error getting mouse position"),
    }
}

#[cfg(target_os = "linux")]
fn focus_window(window: &Window) {
    use gtk::prelude::*;

    if let Ok(gtk_window) = window.gtk_window() {
        gtk_window.set_accept_focus(true);
        gtk_window.set_focus_on_map(true);
        gtk_window.present();

        if let Some(gdk_window) = gtk_window.window() {
            gdk_window.set_accept_focus(true);
            gdk_window.set_focus_on_map(true);
            gdk_window.focus(0);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn focus_window(window: &Window) {
    _ = window.set_focus();
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

pub fn show_window(handler: &AppHandle) {
    let window = handler.get_window(&WINDOW_LABEL.read().unwrap()).unwrap();

    // window.open_devtools();

    let translation = translate();
    // println!("{}", translation);

    window
        .emit(
            "translate",
            Payload {
                message: translation.into(),
            },
        )
        .unwrap();
    set_window_position(&window);
    window.show().unwrap();
    _ = window.set_always_on_top(true);
    focus_window(&window);
}
