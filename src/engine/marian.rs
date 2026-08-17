use super::{TranslationEngine, TranslationError};
use std::path::PathBuf;

pub struct MarianEngine {
    model_path: PathBuf,
}

impl MarianEngine {
    pub fn new(model_path: PathBuf) -> Self {
        Self { model_path }
    }
}

impl TranslationEngine for MarianEngine {
    fn name(&self) -> &str {
        "MarianMT"
    }

    fn translate(
        &self,
        _text: &str,
        _source: &str,
        _target: &str,
    ) -> Result<String, TranslationError> {
        Err(TranslationError::EngineUnavailable(
            "MarianMT engine not implemented yet (STEP 9).".to_string(),
        ))
    }

    fn is_available(&self) -> bool {
        // no futuro: verificar arquivos do modelo
        self.model_path.exists()
    }

    fn requires_api_key(&self) -> bool {
        false
    }
}
