use super::{TranslationEngine, TranslationError};

pub struct WindowsLpEngine;

impl WindowsLpEngine {
    pub fn new() -> Self {
        Self
    }
}

impl TranslationEngine for WindowsLpEngine {
    fn name(&self) -> &'static str {
        "windows_lp"
    }

    fn is_available(&self) -> bool {
        false
    }

    fn translate(&self, _text: &str, _source: &str, _target: &str) -> Result<String, TranslationError> {
        Err(TranslationError::EngineUnavailable(
            "Windows Language Pack translation is not implemented yet on this build".to_string(),
        ))
    }
}
