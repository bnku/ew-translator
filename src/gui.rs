use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
};

use gtk::{gdk, glib, pango, prelude::*};
use global_hotkey::{
    hotkey::HotKey,
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use mouse_position::mouse_position::Mouse;

use crate::{
    selection,
    settings::Settings,
    translator::Translator,
};

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

pub struct Gui {
    window: gtk::Window,
    label: gtk::Label,
    translator: Arc<Translator>,
    settings: Settings,
    hotkey: HotKey,
    _hotkey_manager: GlobalHotKeyManager,
}

impl Gui {
    pub fn new(settings: Settings, translator: Arc<Translator>) -> Result<Self, Box<dyn std::error::Error>> {
        gtk::init()?;

        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_title("ew-translator");
        window.set_decorated(false);
        window.set_keep_above(true);
        window.set_skip_taskbar_hint(true);
        window.set_skip_pager_hint(true);
        window.set_app_paintable(true);
        window.set_accept_focus(true);
        window.set_focus_on_map(true);
        window.add_events(gdk::EventMask::BUTTON_PRESS_MASK | gdk::EventMask::FOCUS_CHANGE_MASK);

        if let Some(screen) = GtkWindowExt::screen(&window) {
            if let Some(visual) = screen.rgba_visual() {
                window.set_visual(Some(&visual));
            }
        }

        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_data(b"
            window {
                background-color: transparent;
            }
            .translation-box {
                background-color: rgba(0, 0, 0, 0.9);
                border-radius: 5px;
                padding: 10px;
            }
            .translation-label {
                color: #ffffff;
                font-size: 16px;
            }
        ")?;

        if let Some(screen) = GtkWindowExt::screen(&window) {
            gtk::StyleContext::add_provider_for_screen(
                &screen,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.style_context().add_class("translation-box");

        let label = gtk::Label::new(None);
        label.style_context().add_class("translation-label");
        label.set_line_wrap(true);
        label.set_line_wrap_mode(pango::WrapMode::WordChar);
        label.set_max_width_chars(45);
        label.set_xalign(0.0);

        container.add(&label);
        window.add(&container);

        // Hide on focus loss
        window.connect_focus_out_event(|win, _| {
            win.hide();
            glib::Propagation::Proceed
        });

        // Hide on Escape
        window.connect_key_press_event(|win, event| {
            if event.keyval() == gdk::keys::constants::Escape {
                win.hide();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });

        // Click to copy translation and hide
        let label_clone = label.clone();
        window.connect_button_press_event(move |win, _| {
            let text = label_clone.text();
            if !text.is_empty() && text.as_str() != "Translating…" {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(text.as_str());
                }
            }
            win.hide();
            glib::Propagation::Proceed
        });

        // Register global hotkey
        let hotkey_manager = GlobalHotKeyManager::new()
            .map_err(|e| format!("Failed to initialize global hotkey manager: {e}"))?;

        let hotkey = HotKey::from_str(&settings.hotkeys)
            .or_else(|_| HotKey::from_str(&settings.hotkeys.to_uppercase()))
            .map_err(|e| format!("Invalid hotkey string '{}': {e}", settings.hotkeys))?;

        hotkey_manager
            .register(hotkey)
            .map_err(|e| format!("Failed to register hotkey '{}': {e}", settings.hotkeys))?;

        Ok(Self {
            window,
            label,
            translator,
            settings,
            hotkey,
            _hotkey_manager: hotkey_manager,
        })
    }

    pub fn run(self) {
        let hotkey_receiver = GlobalHotKeyEvent::receiver();
        let hotkey_id = self.hotkey.id();

        let (hotkey_sender, hotkey_main_rx) = glib::MainContext::channel::<()>(glib::Priority::default());

        let window = self.window.clone();
        let label = self.label.clone();
        let translator = self.translator.clone();
        let target_lang = self.settings.target_lang.clone();

        hotkey_main_rx.attach(None, move |_| {
            Self::trigger_translation(&window, &label, &translator, &target_lang);
            glib::ControlFlow::Continue
        });

        thread::Builder::new()
            .name("hotkey-listener".into())
            .spawn(move || {
                while let Ok(event) = hotkey_receiver.recv() {
                    if event.id == hotkey_id && event.state == HotKeyState::Pressed {
                        let _ = hotkey_sender.send(());
                    }
                }
            })
            .expect("Failed to spawn hotkey listener thread");

        gtk::main();
    }

    fn trigger_translation(
        window: &gtk::Window,
        label: &gtk::Label,
        translator: &Arc<Translator>,
        target_lang: &str,
    ) {
        let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);

        // Move window to mouse position
        match Mouse::get_mouse_position() {
            Mouse::Position { x, y } => {
                window.move_(x, y);
            }
            Mouse::Error => eprintln!("Cannot get mouse position"),
        }

        match selection::get_selected_text() {
            Ok(phrase) => {
                label.set_text("Translating…");
                window.show_all();
                window.present();

                if let Some(gdk_win) = window.window() {
                    gdk_win.set_accept_focus(true);
                    gdk_win.set_focus_on_map(true);
                    gdk_win.focus(0);
                }

                let tr = Arc::clone(translator);
                let lang = target_lang.to_string();
                let (tx, rx) = glib::MainContext::channel::<(u64, Result<String, String>)>(glib::Priority::default());

                let lbl = label.clone();
                let win = window.clone();

                rx.attach(None, move |(req_id, result)| {
                    if NEXT_REQUEST_ID.load(Ordering::Relaxed) - 1 == req_id {
                        match result {
                            Ok(translation) => lbl.set_text(&translation),
                            Err(error) => lbl.set_text(&format!("Error: {error}")),
                        }
                        win.queue_resize();
                    }
                    glib::ControlFlow::Break
                });

                thread::Builder::new()
                    .name(format!("translation-{request_id}"))
                    .spawn(move || {
                        let result = tr.translate(&phrase, &lang).map_err(|e| e.to_string());
                        let _ = tx.send((request_id, result));
                    })
                    .expect("Failed to spawn translation worker");
            }
            Err(err) => {
                label.set_text(&err);
                window.show_all();
                window.present();
            }
        }
    }
}
