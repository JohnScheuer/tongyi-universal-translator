use crate::config::AppConfig;

use super::deepl::DeepLEngine;
use super::google::GoogleEngine;
use super::marian::MarianEngine;
use super::windows_lp::WindowsLPEngine;
use super::TranslationEngine;

#[derive(Debug, Clone)]
pub struct EngineInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub available: bool,
    pub requires_api_key: bool,
}

pub fn get_engine(config: &AppConfig) -> Box<dyn TranslationEngine> {
    get_engine_by_id(config, &config.general.active_engine)
}

pub fn get_engine_by_id(config: &AppConfig, id: &str) -> Box<dyn TranslationEngine> {
    match id {
        "windows_lp" => Box::new(WindowsLPEngine::new()),
        "marian" => Box::new(MarianEngine::new(config.engines.marian.model_path.clone())),
        "deepl" => Box::new(DeepLEngine::new(config.engines.deepl.api_key.clone())),
        "google" => Box::new(GoogleEngine::new(config.engines.google.api_key.clone())),
        _ => Box::new(WindowsLPEngine::new()),
    }
}

pub fn list_engines(config: &AppConfig) -> Vec<EngineInfo> {
    let engines: Vec<(&'static str, &'static str, Box<dyn TranslationEngine>)> = vec![
        ("windows_lp", "Windows Language Pack", Box::new(WindowsLPEngine::new())),
        (
            "marian",
            "MarianMT (Offline)",
            Box::new(MarianEngine::new(config.engines.marian.model_path.clone())),
        ),
        (
            "deepl",
            "DeepL (API)",
            Box::new(DeepLEngine::new(config.engines.deepl.api_key.clone())),
        ),
        (
            "google",
            "Google Translate (API)",
            Box::new(GoogleEngine::new(config.engines.google.api_key.clone())),
        ),
    ];

    engines
        .into_iter()
        .map(|(id, display_name, e)| EngineInfo {
            id,
            display_name,
            available: e.is_available(),
            requires_api_key: e.requires_api_key(),
        })
        .collect()
}

/// Ordem sugerida de fallback (quando um engine falhar).
pub fn fallback_order() -> Vec<&'static str> {
    vec!["windows_lp", "marian", "deepl", "google"]
}
