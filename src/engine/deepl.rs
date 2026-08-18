use std::time::Duration;

use anyhow::Context;
use serde::Deserialize;

use super::{TranslationEngine, TranslationError};

pub struct DeepLEngine {
    api_key: String,
}

impl DeepLEngine {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    fn endpoint(&self) -> &'static str {
        // chaves free frequentemente terminam em ":fx"
        if self.api_key.contains(":fx") {
            "https://api-free.deepl.com/v2/translate"
        } else {
            "https://api.deepl.com/v2/translate"
        }
    }

    fn map_source(src: &str) -> Option<&'static str> {
        match src {
            "pt" => Some("PT"),
            "en" => Some("EN"),
            "es" => Some("ES"),
            _ => None, // deixa auto-detect
        }
    }

    fn map_target(tgt: &str) -> Result<&'static str, TranslationError> {
        match tgt {
            "zh-cn" => Ok("ZH"),
            _ => Err(TranslationError::LanguageNotSupported(tgt.to_string())),
        }
    }
}

#[derive(Debug, Deserialize)]
struct DeepLResp {
    translations: Vec<DeepLTranslation>,
}

#[derive(Debug, Deserialize)]
struct DeepLTranslation {
    text: String,
}

impl TranslationEngine for DeepLEngine {
    fn name(&self) -> &'static str {
        "deepl"
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn translate(&self, text: &str, source: &str, target: &str) -> Result<String, TranslationError> {
        let api_key = self.api_key.trim();
        if api_key.is_empty() {
            return Err(TranslationError::ApiKeyMissing);
        }

        let target = Self::map_target(target)?;
        let source = Self::map_source(source);

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(12))
            .build()
            .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        let mut params: Vec<(&str, String)> = vec![
            ("auth_key", api_key.to_string()),
            ("text", text.to_string()),
            ("target_lang", target.to_string()),
        ];
        if let Some(src) = source {
            params.push(("source_lang", src.to_string()));
        }

        let resp = client
            .post(self.endpoint())
            .form(&params)
            .send()
            .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        let status = resp.status();
        let body = resp.text().unwrap_or_default();

        if !status.is_success() {
            return Err(TranslationError::TranslationFailed(format!(
                "DeepL HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }

        let parsed: DeepLResp = serde_json::from_str(&body)
            .with_context(|| format!("DeepL JSON parse failed: {body}"))
            .map_err(|e| TranslationError::TranslationFailed(e.to_string()))?;

        let out = parsed
            .translations
            .get(0)
            .map(|t| t.text.trim().to_string())
            .unwrap_or_default();

        if out.is_empty() {
            return Err(TranslationError::TranslationFailed(
                "DeepL returned empty translation".to_string(),
            ));
        }

        Ok(out)
    }
}
