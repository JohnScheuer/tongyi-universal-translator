use std::time::Duration;

use anyhow::Context;
use serde::Deserialize;

use super::{TranslationEngine, TranslationError};

pub struct GoogleEngine {
    api_key: String,
}

impl GoogleEngine {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    fn map_lang(lang: &str) -> Result<&'static str, TranslationError> {
        match lang {
            "pt" => Ok("pt"),
            "en" => Ok("en"),
            "es" => Ok("es"),
            "zh-cn" => Ok("zh-CN"),
            _ => Err(TranslationError::LanguageNotSupported(lang.to_string())),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GoogleResp {
    data: GoogleData,
}

#[derive(Debug, Deserialize)]
struct GoogleData {
    translations: Vec<GoogleTranslation>,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslation {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

impl TranslationEngine for GoogleEngine {
    fn name(&self) -> &'static str {
        "google"
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn translate(&self, text: &str, source: &str, target: &str) -> Result<String, TranslationError> {
        let api_key = self.api_key.trim();
        if api_key.is_empty() {
            return Err(TranslationError::ApiKeyMissing);
        }

        let source = Self::map_lang(source)?;
        let target = Self::map_lang(target)?;

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(12))
            .build()
            .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        let url = "https://translation.googleapis.com/language/translate/v2";

        let resp = client
            .post(url)
            .query(&[("key", api_key)])
            .form(&[
                ("q", text),
                ("source", source),
                ("target", target),
                ("format", "text"),
            ])
            .send()
            .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        let status = resp.status();
        let body = resp.text().unwrap_or_default();

        if !status.is_success() {
            return Err(TranslationError::TranslationFailed(format!(
                "Google HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }

        let parsed: GoogleResp = serde_json::from_str(&body)
            .with_context(|| format!("Google JSON parse failed: {body}"))
            .map_err(|e| TranslationError::TranslationFailed(e.to_string()))?;

        let out = parsed
            .data
            .translations
            .get(0)
            .map(|t| t.translated_text.trim().to_string())
            .unwrap_or_default();

        if out.is_empty() {
            return Err(TranslationError::TranslationFailed(
                "Google returned empty translation".to_string(),
            ));
        }

        Ok(out)
    }
}
