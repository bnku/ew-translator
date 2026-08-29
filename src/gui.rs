use mouse_position::mouse_position::Mouse;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
};
use tauri::window::WindowBuilder;
use tauri::{AppHandle, Manager, PhysicalPosition, Position, Window, WindowUrl};

use crate::{settings::WINDOW_LABEL, translate, translator::Translator};

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub fn create_window(handler: &AppHandle, label: &str, url: &str) {
    WindowBuilder::new(handler, label, WindowUrl::App(url.into()))
        .title(label)
        .inner_size(1000.0, 40.0)
        .fullscreen(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .build()
        .expect("cannot create translation window");
}

pub fn hide_window(handler: &AppHandle) {
    let window = handler
        .get_window(WINDOW_LABEL)
        .expect("translation window is missing");
    window.hide().expect("cannot hide translation window");
}

pub fn set_window_position(window: &Window) {
    match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => {
            let _ = window.set_position(Position::Physical(PhysicalPosition { x, y }));
        }
        Mouse::Error => eprintln!("Cannot get mouse position"),
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
    let _ = window.set_focus();
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    request_id: u64,
    message: String,
    loading: bool,
}

pub fn show_window(handler: &AppHandle, translator: Arc<Translator>, target_lang: String) {
    let window = handler
        .get_window(WINDOW_LABEL)
        .expect("translation window is missing");
    let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);

    emit_translation(&window, request_id, "Translating…".into(), true);
    set_window_position(&window);
    window.show().expect("cannot show translation window");
    let _ = window.set_always_on_top(true);
    focus_window(&window);

    let worker_handle = handler.clone();
    let spawn_result = thread::Builder::new()
        .name(format!("translation-{request_id}"))
        .spawn(move || {
            let message = translate(&translator, &target_lang);
            if let Some(window) = worker_handle.get_window(WINDOW_LABEL) {
                emit_translation(&window, request_id, message, false);
            }
        });

    if let Err(error) = spawn_result {
        emit_translation(
            &window,
            request_id,
            format!("Cannot start translation worker: {error}"),
            false,
        );
    }
}

fn emit_translation(window: &Window, request_id: u64, message: String, loading: bool) {
    if let Err(error) = window.emit(
        "translate",
        Payload {
            request_id,
            message,
            loading,
        },
    ) {
        eprintln!("Cannot update translation window: {error}");
    }
}
