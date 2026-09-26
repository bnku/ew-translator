use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread,
};

use eframe::egui::{self, CornerRadius, FontData, FontDefinitions, FontFamily, Frame, Margin};
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
    has_focus: bool,
    just_opened: u8,
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
            has_focus: false,
            just_opened: 0,
        })
    }

    fn hide(&mut self, ctx: &egui::Context) {
        self.visible = false;
        self.has_focus = false;
        self.just_opened = 0;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }

    fn trigger_translation(&mut self, ctx: &egui::Context) {
        let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        self.current_request_id = request_id;

        let mouse_pos = match Mouse::get_mouse_position() {
            Mouse::Position { x, y } => Some(egui::pos2(x as f32, y as f32)),
            Mouse::Error => None,
        };

        if let Some(pos) = mouse_pos {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(pos + egui::vec2(10.0, 15.0)));
        }

        self.visible = true;
        self.has_focus = false;
        self.just_opened = 3;
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

        // Handle Escape key
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide(ctx);
            return;
        }

        // Handle focus loss
        let is_focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if is_focused {
            self.has_focus = true;
        }

        if self.just_opened > 0 {
            self.just_opened -= 1;
        } else if self.has_focus && !is_focused {
            self.hide(ctx);
            return;
        }

        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ctx, |ui| {
                let card = Frame::NONE
                    .fill(egui::Color32::from_rgba_unmultiplied(22, 22, 26, 242))
                    .corner_radius(CornerRadius::same(8))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(35)))
                    .inner_margin(Margin::symmetric(14, 10));

                let resp = card.show(ui, |ui| {
                    ui.set_max_width(420.0);
                    if self.loading {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(&self.text)
                                    .size(15.0)
                                    .color(egui::Color32::from_rgb(180, 180, 190)),
                            );
                        });
                    } else {
                        let label = egui::Label::new(
                            egui::RichText::new(&self.text)
                                .size(15.0)
                                .color(egui::Color32::from_rgb(245, 245, 245)),
                        )
                        .wrap();
                        ui.add(label);
                    }
                });

                if !self.loading && resp.response.clicked() {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(&self.text);
                    }
                    self.hide(ctx);
                    return;
                }

                let content_size = resp.response.rect.size();
                let target_size = egui::vec2(
                    (content_size.x + 2.0).clamp(160.0, 460.0),
                    (content_size.y + 2.0).clamp(36.0, 600.0),
                );
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target_size));
            });
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
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
