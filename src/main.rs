mod gui;
mod selection;
mod settings;
mod translator;

use std::sync::Arc;
use eframe::egui;
use translator::Translator;

fn main() -> eframe::Result<()> {
    let settings = settings::load().unwrap_or_else(|error| {
        eprintln!("Configuration error: {error}");
        std::process::exit(2);
    });

    let translator = Arc::new(Translator::new(settings.translation.clone()).unwrap_or_else(|error| {
        eprintln!("Translator initialization error: {error}");
        std::process::exit(1);
    }));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ew-translator")
            .with_decorations(false)
            .with_always_on_top()
            .with_inner_size([300.0, 60.0])
            .with_visible(false)
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "ew-translator",
        options,
        Box::new(move |cc| {
            let app = gui::TranslatorApp::new(cc, settings, translator)
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.to_string().into() })?;
            Ok(Box::new(app))
        }),
    )
}
