use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use eframe::egui::{self, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Frame, Margin, RichText};
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

enum TranslationEvent {
    Result {
        request_id: u64,
        result: Result<String, String>,
    },
}

pub struct TranslatorApp {
    translator: Arc<Translator>,
    settings: Settings,
    hotkey: HotKey,
    hotkey_event_rx: Receiver<GlobalHotKeyEvent>,
    _hotkey_manager: GlobalHotKeyManager,

    translation_tx: Sender<TranslationEvent>,
    translation_rx: Receiver<TranslationEvent>,

    visible: bool,
    loading: bool,
    text: String,
    current_request_id: u64,
    just_opened: u8,
    target_pos: Option<egui::Pos2>,
    current_size: egui::Vec2,
    reposition_counter: u8,
}

impl TranslatorApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        settings: Settings,
        translator: Arc<Translator>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        setup_fonts(&cc.egui_ctx);

        let hotkey_manager = GlobalHotKeyManager::new()
            .map_err(|e| format!("Failed to initialize global hotkey manager: {e}"))?;

        let hotkey = HotKey::from_str(&settings.hotkeys)
            .or_else(|_| HotKey::from_str(&settings.hotkeys.to_uppercase()))
            .map_err(|e| format!("Invalid hotkey string '{}': {e}", settings.hotkeys))?;

        hotkey_manager
            .register(hotkey)
            .map_err(|e| format!("Failed to register hotkey '{}': {e}", settings.hotkeys))?;

        let (hotkey_tx, hotkey_event_rx) = channel();
        let hotkey_receiver = GlobalHotKeyEvent::receiver();
        let ctx_clone = cc.egui_ctx.clone();

        thread::Builder::new()
            .name("hotkey-listener".into())
            .spawn(move || {
                while let Ok(event) = hotkey_receiver.recv() {
                    let _ = hotkey_tx.send(event);
                    ctx_clone.request_repaint();
                }
            })?;

        let (translation_tx, translation_rx) = channel();

        Ok(Self {
            translator,
            settings,
            hotkey,
            hotkey_event_rx,
            _hotkey_manager: hotkey_manager,
            translation_tx,
            translation_rx,
            visible: false,
            loading: false,
            text: String::new(),
            current_request_id: 0,
            just_opened: 0,
            target_pos: None,
            current_size: egui::vec2(130.0, 36.0),
            reposition_counter: 0,
        })
    }

    fn hide(&mut self, ctx: &egui::Context) {
        self.visible = false;
        self.just_opened = 0;
        self.reposition_counter = 0;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }

    fn trigger_translation(&mut self, ctx: &egui::Context) {
        let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        self.current_request_id = request_id;

        // Reset window size for new request so it doesn't preserve previous large dimensions
        self.current_size = egui::vec2(130.0, 36.0);
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.current_size));

        // Mouse::get_mouse_position returns physical screen coordinates.
        // egui ViewportCommand::OuterPosition takes logical points!
        // Across multi-monitor setups, convert physical -> logical:
        let ppp = ctx.pixels_per_point().max(1.0);
        let logical_pos = match Mouse::get_mouse_position() {
            Mouse::Position { x, y } => {
                let lx = (x as f32) / ppp + 12.0;
                let ly = (y as f32) / ppp + 16.0;
                Some(egui::pos2(lx, ly))
            }
            Mouse::Error => None,
        };

        self.target_pos = logical_pos;
        self.reposition_counter = 4;
        self.visible = true;
        self.just_opened = 4; // Ignore clicks for first 4 frames while popping up

        if let Some(pos) = self.target_pos {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);

        match selection::get_selected_text() {
            Ok(phrase) => {
                self.loading = true;
                self.text = "Translating…".to_string();

                let translator = Arc::clone(&self.translator);
                let target_lang = self.settings.target_lang.clone();
                let tx = self.translation_tx.clone();
                let ctx_clone = ctx.clone();

                let spawn_res = thread::Builder::new()
                    .name(format!("translation-{request_id}"))
                    .spawn(move || {
                        let result = translator
                            .translate(&phrase, &target_lang)
                            .map_err(|e| e.to_string());
                        let _ = tx.send(TranslationEvent::Result { request_id, result });
                        ctx_clone.request_repaint();
                    });

                if let Err(e) = spawn_res {
                    let _ = self.translation_tx.send(TranslationEvent::Result {
                        request_id,
                        result: Err(format!("Worker error: {e}")),
                    });
                    ctx.request_repaint();
                }
            }
            Err(err) => {
                self.loading = false;
                self.text = err;
            }
        }
    }
}

impl eframe::App for TranslatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process hotkey events
        while let Ok(event) = self.hotkey_event_rx.try_recv() {
            if event.id == self.hotkey.id() && event.state == HotKeyState::Pressed {
                self.trigger_translation(ctx);
            }
        }

        // Process translation results
        while let Ok(event) = self.translation_rx.try_recv() {
            match event {
                TranslationEvent::Result { request_id, result } => {
                    if request_id >= self.current_request_id {
                        self.loading = false;
                        match result {
                            Ok(translation) => self.text = translation,
                            Err(error) => self.text = format!("Error: {error}"),
                        }
                    }
                }
            }
        }

        if !self.visible {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            return;
        }

        // Keep position aligned on multi-monitor setups during initial frames
        if self.reposition_counter > 0 {
            self.reposition_counter -= 1;
            if let Some(pos) = self.target_pos {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos));
            }
        }

        // Handle Escape key
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide(ctx);
            return;
        }

        let ppp = ctx.pixels_per_point().max(1.0);

        // Detect mouse click outside window to hide popup
        if self.just_opened > 0 {
            self.just_opened -= 1;
        } else if let Some(target_pos) = self.target_pos {
            let window_rect = egui::Rect::from_min_size(target_pos, self.current_size);
            #[cfg(target_os = "linux")]
            if is_mouse_button_pressed_outside(window_rect, ppp) {
                self.hide(ctx);
                return;
            }
        }

        egui::CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(egui::Color32::from_rgb(18, 18, 22))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(14, 10)),
            )
            .show(ctx, |ui| {
                ui.set_max_width(380.0);

                let resp = if self.loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(&self.text)
                                .font(FontId::proportional(16.0))
                                .color(egui::Color32::from_rgb(180, 180, 190)),
                        );
                    })
                    .response
                } else {
                    let label = egui::Label::new(
                        RichText::new(&self.text)
                            .font(FontId::proportional(16.0))
                            .color(egui::Color32::from_rgb(245, 245, 245)),
                    )
                    .wrap();
                    ui.add(label)
                };

                // Adjust window dimensions tightly and strictly to the rendered widget size!
                let text_size = resp.rect.size();
                let target_size = egui::vec2(
                    (text_size.x + 28.0).clamp(60.0, 420.0),
                    (text_size.y + 20.0).clamp(32.0, 600.0),
                );
                self.current_size = target_size;
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target_size));
            });

        // Request update while visible to detect clicks outside
        ctx.request_repaint_after(Duration::from_millis(50));
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from_rgb(18.0 / 255.0, 18.0 / 255.0, 22.0 / 255.0).to_array()
    }
}

#[cfg(target_os = "linux")]
fn is_mouse_button_pressed_outside(window_rect: egui::Rect, ppp: f32) -> bool {
    use x11_dl::xlib;

    let Ok(xlib) = xlib::Xlib::open() else {
        return false;
    };
    let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };
    if display.is_null() {
        return false;
    }

    let screen = unsafe { (xlib.XDefaultScreen)(display) };
    let root = unsafe { (xlib.XRootWindow)(display, screen) };

    let mut root_return: xlib::Window = 0;
    let mut child_return: xlib::Window = 0;
    let mut root_x: i32 = -1;
    let mut root_y: i32 = -1;
    let mut win_x: i32 = 0;
    let mut win_y: i32 = 0;
    let mut mask: u32 = 0;

    unsafe {
        (xlib.XQueryPointer)(
            display,
            root,
            &mut root_return,
            &mut child_return,
            &mut root_x,
            &mut root_y,
            &mut win_x,
            &mut win_y,
            &mut mask,
        );
        (xlib.XCloseDisplay)(display);
    }

    if root_x < 0 || root_y < 0 {
        return false;
    }

    // Check if mouse buttons (Left, Middle, Right) are pressed
    let buttons = (xlib::Button1Mask | xlib::Button2Mask | xlib::Button3Mask) as u32;
    if (mask & buttons) != 0 {
        let logical_mouse = egui::pos2(root_x as f32 / ppp, root_y as f32 / ppp);
        // If mouse is OUTSIDE the window rect when clicked -> true!
        if !window_rect.contains(logical_mouse) {
            return true;
        }
    }

    false
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "roboto".to_owned(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Roboto-Regular.ttf"
        ))),
    );

    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "roboto".to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .push("roboto".to_owned());

    ctx.set_fonts(fonts);
}
