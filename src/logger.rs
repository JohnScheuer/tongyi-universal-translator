use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::config::AppConfig;
use crate::translator::TranslationResult;

const MAX_LOG_BYTES: u64 = 1_000_000;

fn rotate_if_needed(path: &Path) -> Result<()> {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };

    if meta.len() <= MAX_LOG_BYTES {
        return Ok(());
    }

    let rotated = path.with_extension("log.1");
    // remove rotated if exists
    let _ = fs::remove_file(&rotated);
    fs::rename(path, rotated).context("Failed to rotate log file")?;
    Ok(())
}

pub fn log_translation(config: &AppConfig, r: &TranslationResult) -> Result<()> {
    if !config.ui.log_translations {
        return Ok(());
    }

    let path = &config.ui.log_file;

    rotate_if_needed(path)?;

    let ts = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let line = format!(
        "[{ts}] {}→{} | {} | {}ms | {:?} -> {:?}\n",
        config.general.source_language,
        config.general.target_language,
        r.engine_used,
        r.duration_ms,
        r.original.trim(),
        r.translated.trim()
    );

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Failed to open log file: {}", path.display()))?;

    f.write_all(line.as_bytes())
        .with_context(|| format!("Failed to write log file: {}", path.display()))?;

    Ok(())
}
