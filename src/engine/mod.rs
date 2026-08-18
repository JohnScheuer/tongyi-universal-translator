use thiserror::Error;

pub mod router;
pub mod deepl;
pub mod google;
pub mod marian;
pub mod windows_lp;

pub trait TranslationEngine {
    fn name(&self) -> &'static str;

    fn is_available(&self) -> bool {
        true
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn translate(&self, text: &str, source: &str, target: &str) -> Result<String, TranslationError>;
}

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("api key missing")]
    ApiKeyMissing,

    #[error("network error: {0}")]
    NetworkError(String),

    #[error("language not supported: {0}")]
    LanguageNotSupported(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("engine unavailable: {0}")]
    EngineUnavailable(String),

    #[error("translation failed: {0}")]
    TranslationFailed(String),
}
