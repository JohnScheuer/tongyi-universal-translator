use crate::config::AppConfig;

use super::{
    deepl::DeepLEngine, google::GoogleEngine, marian::MarianEngine, windows_lp::WindowsLpEngine,
    TranslationEngine,
};

pub fn get_engine(cfg: &AppConfig) -> Box<dyn TranslationEngine + Send + Sync> {
    match cfg.general.active_engine.as_str() {
        "deepl" => Box::new(DeepLEngine::new(cfg.engines.deepl.api_key.clone())),
        "google" => Box::new(GoogleEngine::new(cfg.engines.google.api_key.clone())),
        "marian" => Box::new(MarianEngine::new(cfg.engines.marian.model_path.clone())),
        "windows_lp" => Box::new(WindowsLpEngine::new()),
        _ => Box::new(WindowsLpEngine::new()),
    }
}
