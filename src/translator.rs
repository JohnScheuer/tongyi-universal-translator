// src/translator.rs
use anyhow::{anyhow, Context, Result};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

use crate::{clipboard, engine, input};
use crate::config::AppConfig;

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{GetForegroundWindow, SetForegroundWindow},
};

static TRANSLATING: AtomicBool = AtomicBool::new(false);

pub struct TranslationResult {
    pub original: String,
    pub translated: String,
    pub engine_used: String,
    pub duration_ms: u64,
}

struct TranslatingGuard;

impl TranslatingGuard {
    fn enter() -> Result<Self> {
        if TRANSLATING.swap(true, Ordering::SeqCst) {
            return Err(anyhow!("translation already in progress"));
        }
        Ok(Self)
    }
}

impl Drop for TranslatingGuard {
    fn drop(&mut self) {
        TRANSLATING.store(false, Ordering::SeqCst);
    }
}

struct ClipboardRestoreGuard {
    backup: Option<clipboard::ClipboardBackup>,
}

impl ClipboardRestoreGuard {
    fn new() -> Result<Self> {
        let b = clipboard::backup().context("clipboard backup failed")?;
        Ok(Self { backup: Some(b) })
    }
}

impl Drop for ClipboardRestoreGuard {
    fn drop(&mut self) {
        if let Some(b) = self.backup.take() {
            let _ = clipboard::restore(b);
        }
    }
}

fn normalize_phrase(s: &str) -> String {
    let s = s.trim().to_lowercase();
    s.trim_matches(|c: char| c.is_whitespace() || "!?.,;:，。！？".contains(c))
        .to_string()
}

fn phrasebook_zh_cn(text: &str) -> Option<&'static str> {
    let t = normalize_phrase(text);

    // PT
    match t.as_str() {
        "bom dia" => return Some("早上好"),
        "boa tarde" => return Some("下午好"),
        "boa noite" => return Some("晚上好"),
        "olá" | "ola" => return Some("你好"),
        "tudo bem" => return Some("你好吗？"),
        "como vai" => return Some("你好吗？"),
        "obrigado" | "obrigada" => return Some("谢谢"),
        "por favor" => return Some("请"),
        _ => {}
    }

    // EN
    match t.as_str() {
        "good morning" => return Some("早上好"),
        "good afternoon" => return Some("下午好"),
        "good evening" => return Some("晚上好"),
        "good night" => return Some("晚安"),
        "hello" | "hi" => return Some("你好"),
        "thank you" | "thanks" => return Some("谢谢"),
        _ => {}
    }

    // ES
    match t.as_str() {
        "buenos dias" | "buenos días" => return Some("早上好"),
        "buenas tardes" => return Some("下午好"),
        "buenas noches" => return Some("晚上好"),
        "hola" => return Some("你好"),
        "gracias" => return Some("谢谢"),
        _ => {}
    }

    None
}

fn translate_via_selected_engine(cfg: &AppConfig, original: &str) -> Result<(String, String)> {
    let eng = engine::router::get_engine(cfg);

    let out = eng
        .translate(
            original,
            cfg.general.source_language.trim(),
            cfg.general.target_language.trim(),
        )
        .map_err(|e| anyhow!("{e}"))?;

    Ok((out, eng.name().to_string()))
}

fn read_selected_text_via_clipboard() -> Result<String> {
    // Evita ler clipboard velho (best-effort)
    clipboard::clear().ok();
    std::thread::sleep(Duration::from_millis(15));

    // Ctrl+A / Ctrl+C
    input::simulate_select_all().context("Ctrl+A failed")?;
    std::thread::sleep(Duration::from_millis(30));

    input::simulate_copy().context("Ctrl+C failed")?;
    std::thread::sleep(Duration::from_millis(40));

    // Retry: Ctrl+C pode demorar
    let s = clipboard::read_text_retry(Duration::from_millis(500))
        .context("reading clipboard after copy failed")?;

    Ok(s)
}

fn refocus_and_replace_with_clipboard(target_hwnd: HWND) -> Result<()> {
    unsafe {
        let _ = SetForegroundWindow(target_hwnd);
    }
    std::thread::sleep(Duration::from_millis(30));

    // best-effort: neutraliza Shift antes de Ctrl+V
    input::release_shift().ok();
    std::thread::sleep(Duration::from_millis(10));

    // re-seleciona para garantir substituição (importante após tradução lenta)
    input::simulate_select_all().ok();
    std::thread::sleep(Duration::from_millis(15));

    // cola (e tenta uma 2ª vez em apps que perdem o primeiro Ctrl+V)
    input::simulate_paste().context("Ctrl+V failed")?;
    std::thread::sleep(Duration::from_millis(25));
    input::simulate_paste().ok();

    Ok(())
}

pub fn translate_active_field(cfg: &AppConfig) -> Result<TranslationResult> {
    let _translating = TranslatingGuard::enter()?;
    let start = Instant::now();

    let _restore = ClipboardRestoreGuard::new()?;

    // Captura a janela em foco no momento do hotkey.
    // Se durante a tradução (Marian) o usuário trocar de foco, vamos abortar o paste por segurança.
    let target_hwnd = unsafe { GetForegroundWindow() };
    if target_hwnd.0.is_null() {
        return Err(anyhow!("no foreground window"));
    }

    // Hotkey é Shift+Enter: liberar Shift para não interferir nos combos.
    input::release_shift().ok();
    std::thread::sleep(Duration::from_millis(25));

    // 1) Captura texto do campo ativo
    let original = read_selected_text_via_clipboard()?;
    let original = original.trim().to_string();

    if original.is_empty() {
        return Err(anyhow!("no text detected in active field"));
    }

    // 2) Tradução:
    // phrasebook-first para zh-cn (saudações curtas ficam melhores que pivot)
    let (translated, engine_used) = if cfg.general.target_language.trim() == "zh-cn" {
        if let Some(hit) = phrasebook_zh_cn(&original) {
            (hit.to_string(), "phrasebook".to_string())
        } else {
            translate_via_selected_engine(cfg, &original).context("translation failed")?
        }
    } else {
        translate_via_selected_engine(cfg, &original).context("translation failed")?
    };

    // 3) Segurança: se o foco mudou durante a tradução, não colar em lugar errado
    let current_hwnd = unsafe { GetForegroundWindow() };
    if current_hwnd.0 != target_hwnd.0 {
        return Err(anyhow!(
            "focus changed during translation; aborted paste to avoid wrong target"
        ));
    }

    // 4) Escreve tradução no clipboard
    clipboard::write_text(&translated).context("write translated text to clipboard failed")?;
    std::thread::sleep(Duration::from_millis(25));

    // 5) Refocus + substituir via Ctrl+A / Ctrl+V
    refocus_and_replace_with_clipboard(target_hwnd)?;

    Ok(TranslationResult {
        original,
        translated,
        engine_used,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}
