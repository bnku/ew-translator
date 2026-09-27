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
    mouse_phys_pos: Option<(i32, i32)>,
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
            current_size: egui::vec2(148.0, 40.0),
            reposition_counter: 0,
            mouse_phys_pos: None,
        })
    }

    fn hide(&mut self, ctx: &egui::Context) {
        self.visible = false;
        self.just_opened = 0;
        self.reposition_counter = 0;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }

    fn compute_geometry(&self, ctx: &egui::Context) -> (egui::Vec2, egui::Pos2, bool) {
        let ppp = ctx.pixels_per_point().max(1.0);
        let (phys_x, phys_y) = self.mouse_phys_pos.unwrap_or((100, 100));
        let lx = (phys_x as f32) / ppp;
        let ly = (phys_y as f32) / ppp;

        #[cfg(target_os = "linux")]
        let mon_rect = get_monitor_rect_for_point(phys_x, phys_y)
            .map(|r| {
                egui::Rect::from_min_max(
                    egui::pos2(r.min.x / ppp, r.min.y / ppp),
                    egui::pos2(r.max.x / ppp, r.max.y / ppp),
                )
            })
            .unwrap_or_else(|| {
                let s = ctx.input(|i| i.screen_rect());
                if s.width() > 10.0 {
                    s
                } else {
                    egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1920.0, 1080.0))
                }
            });

        #[cfg(not(target_os = "linux"))]
        let mon_rect = {
            let s = ctx.input(|i| i.screen_rect());
            if s.width() > 10.0 {
                s
            } else {
                egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1920.0, 1080.0))
            }
        };

        let screen_margin = 12.0;
        let cursor_offset_y = 16.0;
        let cursor_offset_x = 12.0;

        let (content_w, content_h, is_long_text) = if self.loading {
            (120.0, 20.0, false)
        } else {
            let font_id = FontId::proportional(16.0);
            let max_text_width = 380.0;
            let galley = ctx.fonts(|f| {
                f.layout(
                    self.text.clone(),
                    font_id,
                    egui::Color32::WHITE,
                    max_text_width,
                )
            });
            let sz = galley.size();
            (sz.x, sz.y, true)
        };

        let spawn_below_y = ly + cursor_offset_y;
        let space_below = mon_rect.max.y - spawn_below_y - screen_margin;
        let space_above = (ly - cursor_offset_y) - mon_rect.min.y - screen_margin;

        // If at least ~150px below cursor: spawn downward and clamp to screen bottom.
        // If less than 150px below cursor: spawn upward from cursor.
        let spawn_upwards = space_below < 150.0 && space_above > space_below;

        let available_h = if spawn_upwards {
            space_above.clamp(100.0, 600.0)
        } else {
            space_below.clamp(100.0, 600.0)
        };

        let pad_x = 28.0;
        let pad_y = 20.0;
        let needed_win_h = content_h + pad_y;
        let needs_scroll = is_long_text && (needed_win_h > available_h);

        let win_h = if needs_scroll {
            available_h
        } else {
            needed_win_h.clamp(36.0, available_h)
        };

        let scrollbar_extra = if needs_scroll { 14.0 } else { 0.0 };
        let win_w = (content_w + pad_x + scrollbar_extra).clamp(120.0, 440.0);

        let raw_x = lx + cursor_offset_x;
        let min_x = mon_rect.min.x + screen_margin;
        let max_x = (mon_rect.max.x - screen_margin - win_w).max(min_x);
        let win_x = raw_x.clamp(min_x, max_x);

        let win_y = if spawn_upwards {
            (ly - cursor_offset_y - win_h).max(mon_rect.min.y + screen_margin)
        } else {
            spawn_below_y
        };

        (egui::vec2(win_w, win_h), egui::pos2(win_x, win_y), needs_scroll)
    }

    fn trigger_translation(&mut self, ctx: &egui::Context) {
        let request_id = NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
        self.current_request_id = request_id;

        let phys_mouse = match Mouse::get_mouse_position() {
            Mouse::Position { x, y } => Some((x, y)),
            Mouse::Error => None,
        };
        self.mouse_phys_pos = phys_mouse;

        self.visible = true;
        self.loading = true;
        self.text = "Translating…".to_string();
        self.just_opened = 4;
        self.reposition_counter = 4;

        let (init_size, init_pos, _) = self.compute_geometry(ctx);
        self.current_size = init_size;
        self.target_pos = Some(init_pos);
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(init_size));
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(init_pos));
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);

        match selection::get_selected_text() {
            Ok(phrase) => {
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
                self.reposition_counter = 4;
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
                        self.reposition_counter = 4;
                    }
                }
            }
        }

        if !self.visible {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            return;
        }

        let (target_size, target_pos, needs_scroll) = self.compute_geometry(ctx);
        self.current_size = target_size;
        self.target_pos = Some(target_pos);

        // Keep position and size aligned during transition frames
        if self.reposition_counter > 0 {
            self.reposition_counter -= 1;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(target_size));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(target_pos));
        }

        // Handle Escape key
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.hide(ctx);
            return;
        }

        let ppp = ctx.pixels_per_point().max(1.0);

        // Detect mouse click outside window to hide popup (unless dragging inside)
        let is_dragging = ctx.dragged_id().is_some();
        if self.just_opened > 0 {
            self.just_opened -= 1;
        } else if !is_dragging {
            let window_rect = egui::Rect::from_min_size(target_pos, target_size);
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
                if self.loading {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(&self.text)
                                .font(FontId::proportional(16.0))
                                .color(egui::Color32::from_rgb(180, 180, 190)),
                        );
                    });
                } else if needs_scroll {
                    let max_content_h = (target_size.y - 20.0).max(30.0);
                    egui::ScrollArea::vertical()
                        .id_salt(self.current_request_id)
                        .max_height(max_content_h)
                        .drag_to_scroll(false)
                        .show(ui, |ui| {
                            let label = egui::Label::new(
                                RichText::new(&self.text)
                                    .font(FontId::proportional(16.0))
                                    .color(egui::Color32::from_rgb(245, 245, 245)),
                            )
                            .wrap()
                            .selectable(true);
                            ui.add(label);
                        });
                } else {
                    let label = egui::Label::new(
                        RichText::new(&self.text)
                            .font(FontId::proportional(16.0))
                            .color(egui::Color32::from_rgb(245, 245, 245)),
                    )
                    .wrap()
                    .selectable(true);
                    ui.add(label);
                }
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

    ctx.style_mut(|s| {
        s.spacing.scroll = egui::style::ScrollStyle::solid();
        s.spacing.scroll.bar_width = 6.0;
        s.spacing.scroll.bar_inner_margin = 4.0;
    });
}

#[cfg(target_os = "linux")]
pub fn get_monitor_rect_for_point(px: i32, py: i32) -> Option<egui::Rect> {
    use x11_dl::{xinerama, xlib};
    let xlib = xlib::Xlib::open().ok()?;
    let xinerama = xinerama::Xlib::open().ok()?;
    let display = unsafe { (xlib.XOpenDisplay)(std::ptr::null()) };
    if display.is_null() {
        return None;
    }
    let rect = unsafe {
        let is_active = (xinerama.XineramaIsActive)(display);
        if is_active != 0 {
            let mut count: std::os::raw::c_int = 0;
            let screens_ptr = (xinerama.XineramaQueryScreens)(display, &mut count);
            if !screens_ptr.is_null() && count > 0 {
                let screens = std::slice::from_raw_parts(screens_ptr, count as usize);
                let mut found = None;
                for s in screens {
                    let min_x = s.x_org as f32;
                    let min_y = s.y_org as f32;
                    let max_x = (s.x_org + s.width) as f32;
                    let max_y = (s.y_org + s.height) as f32;
                    let r = egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y));
                    if (px as f32) >= min_x && (px as f32) < max_x && (py as f32) >= min_y && (py as f32) < max_y {
                        found = Some(r);
                        break;
                    }
                }
                (xlib.XFree)(screens_ptr as *mut _);
                found
            } else {
                None
            }
        } else {
            None
        }
    };
    unsafe { (xlib.XCloseDisplay)(display); }
    rect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_monitor_rect() {
        if let Some(rect) = get_monitor_rect_for_point(2500, 500) {
            println!("Found monitor rect for (2500, 500): {:?}", rect);
            assert!(rect.width() > 0.0);
        }
    }
}
