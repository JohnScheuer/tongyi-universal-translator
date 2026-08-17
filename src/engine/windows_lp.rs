use super::{TranslationEngine, TranslationError};

/// Placeholder V1 (até STEP 6):
/// Não usa APIs do Windows ainda; apenas traduz algumas frases comuns
/// para validar pipeline (hotkey → clipboard → translate → paste).
pub struct WindowsLPEngine;

impl WindowsLPEngine {
    pub fn new() -> Self {
        Self
    }
}

impl TranslationEngine for WindowsLPEngine {
    fn name(&self) -> &str {
        "Windows LP"
    }

    fn translate(
        &self,
        text: &str,
        _source: &str,
        target: &str,
    ) -> Result<String, TranslationError> {
        let tgt = target.to_ascii_lowercase();
        if tgt != "zh-cn" && tgt != "zh" {
            return Err(TranslationError::LanguageNotSupported(format!(
                "target={target}"
            )));
        }

        let normalized = text.trim().to_ascii_lowercase();

        // mini-dicionário só pra ter um engine “funcionando” no pipeline
        let out = match normalized.as_str() {
            "hello" => "你好",
            "hi" => "你好",
            "good morning" => "早上好",
            "good night" => "晚安",
            "thank you" => "谢谢",
            "bom dia" => "早上好",
            "boa noite" => "晚安",
            "obrigado" | "obrigada" => "谢谢",
            "buenos días" | "buenos dias" => "早上好",
            "buenas noches" => "晚安",
            _ => return Ok(format!("{}（译）", text.trim())),
        };

        Ok(out.to_string())
    }

    fn is_available(&self) -> bool {
        true
    }

    fn requires_api_key(&self) -> bool {
        false
    }
}
