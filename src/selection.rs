use arboard::Clipboard;

pub fn get_selected_text() -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Clipboard error: {e}"))?;

    #[cfg(target_os = "linux")]
    {
        use arboard::{GetExtLinux, LinuxClipboardKind};
        if let Ok(text) = clipboard.get().clipboard(LinuxClipboardKind::Primary).text() {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }

    // Fallback to standard clipboard
    if let Ok(text) = clipboard.get_text() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }

    Err("No text selected or clipboard is empty".to_string())
}
