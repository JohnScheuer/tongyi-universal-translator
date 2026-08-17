use thiserror::Error;

pub mod deepl;
pub mod google;
pub mod marian;
pub mod router;
pub mod windows_lp;

pub trait TranslationEngine: Send + Sync {
    fn name(&self) -> &str;

    fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<String, TranslationError>;

    fn is_available(&self) -> bool;

    fn requires_api_key(&self) -> bool;
}

#[derive(Debug, Error)]
pub enum TranslationError {
    #[error("network error: {0}")]
    NetworkError(String),

    #[error("api key missing")]
    ApiKeyMissing,

    #[error("language not supported: {0}")]
    LanguageNotSupported(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("engine unavailable: {0}")]
    EngineUnavailable(String),

    #[error("translation failed: {0}")]
    TranslationFailed(String),
}
