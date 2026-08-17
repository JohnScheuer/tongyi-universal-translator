use anyhow::{anyhow, Context, Result};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::clipboard;
use crate::config::AppConfig;
use crate::engine::router;
use crate::{logger, ui};

#[derive(Debug, Clone)]
pub struct TranslationResult {
    pub original: String,
    pub translated: String,
    pub engine_used: String,
    pub duration_ms: u64,
}

struct ClipboardRestoreGuard {
    backup: Option<clipboard::ClipboardBackup>,
}

impl ClipboardRestoreGuard {
    fn new(backup: clipboard::ClipboardBackup) -> Self {
        Self {
            backup: Some(backup),
        }
    }

    fn restore_now(&mut self) {
        if let Some(b) = self.backup.take() {
            let _ = clipboard::restore(b);
        }
    }
}

impl Drop for ClipboardRestoreGuard {
    fn drop(&mut self) {
        self.restore_now();
    }
}

fn translate_with_fallback(config: &AppConfig, text: &str) -> Result<(String, String)> {
    let src = config.general.source_language.as_str();
    let tgt = config.general.target_language.as_str();

    let mut tried: Vec<String> = Vec::new();

    let mut ids: Vec<String> = Vec::new();
    ids.push(config.general.active_engine.clone());
    for id in router::fallback_order() {
        if !ids.iter().any(|x| x == id) {
            ids.push(id.to_string());
        }
    }

    for id in ids {
        let engine = router::get_engine_by_id(config, &id);

        if !engine.is_available() {
            tried.push(format!("{id}: unavailable"));
            continue;
        }

        match engine.translate(text, src, tgt) {
            Ok(out) => return Ok((out, engine.name().to_string())),
            Err(e) => {
                tried.push(format!("{id}: {e}"));
                continue;
            }
        }
    }

    Err(anyhow!(
        "All translation engines failed. Tried: {}",
        tried.join(" | ")
    ))
}

pub fn translate_active_field(config: &AppConfig) -> Result<TranslationResult> {
    let start = Instant::now();

    let backup = clipboard::backup().context("Failed to backup clipboard")?;
    let mut restore_guard = ClipboardRestoreGuard::new(backup);

    // garantir que Shift do hotkey não contamine os combos
    let _ = crate::input::wait_for_modifiers_released(250);
    let _ = crate::input::release_stuck_modifiers();

    crate::input::simulate_select_all().context("Failed to simulate Ctrl+A")?;
    sleep(Duration::from_millis(50));

    crate::input::simulate_copy().context("Failed to simulate Ctrl+C")?;
    sleep(Duration::from_millis(50));

    let original = clipboard::read_text().context("Failed to read text from clipboard")?;
    if original.trim().is_empty() {
        return Err(anyhow!("No text selected / clipboard empty"));
    }

    let (translated, engine_used) =
        translate_with_fallback(config, &original).context("Translation failed")?;

    clipboard::write_text(&translated).context("Failed to write translated text to clipboard")?;
    sleep(Duration::from_millis(20));

    crate::input::simulate_paste().context("Failed to simulate Ctrl+V")?;
    sleep(Duration::from_millis(180));

    restore_guard.restore_now();

    let result = TranslationResult {
        original,
        translated,
        engine_used,
        duration_ms: start.elapsed().as_millis() as u64,
    };

    // pós-ação: notificação e log (não deixar crashar)
    let _ = ui::notification::show_translation(config, &result);
    let _ = logger::log_translation(config, &result);

    Ok(result)
}
