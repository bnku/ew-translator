mod gui;
mod selection;
mod settings;
mod translator;

use std::sync::Arc;
use translator::Translator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = settings::load().unwrap_or_else(|error| {
        eprintln!("Configuration error: {error}");
        std::process::exit(2);
    });

    let translator = Arc::new(Translator::new(settings.translation.clone()).unwrap_or_else(|error| {
        eprintln!("Translator initialization error: {error}");
        std::process::exit(1);
    }));

    let app = gui::Gui::new(settings, translator)?;
    app.run();

    Ok(())
}
