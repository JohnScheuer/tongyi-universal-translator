use super::{TranslationEngine, TranslationError};

pub struct GoogleEngine {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl GoogleEngine {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn map_lang_src(src: &str) -> Result<&'static str, TranslationError> {
        match src.to_ascii_lowercase().as_str() {
            "pt" => Ok("pt"),
            "en" => Ok("en"),
            "es" => Ok("es"),
            other => Err(TranslationError::LanguageNotSupported(format!(
                "source={other}"
            ))),
        }
    }

    fn map_lang_tgt(tgt: &str) -> Result<&'static str, TranslationError> {
        match tgt.to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" => Ok("zh-CN"),
            other => Err(TranslationError::LanguageNotSupported(format!(
                "target={other}"
            ))),
        }
    }
}

impl TranslationEngine for GoogleEngine {
    fn name(&self) -> &str {
        "Google Translate"
    }

    fn translate(
        &self,
        text: &str,
        source: &str,
        target: &str,
    ) -> Result<String, TranslationError> {
        if self.api_key.trim().is_empty() {
            return Err(TranslationError::ApiKeyMissing);
        }

        let src = Self::map_lang_src(source)?;
        let tgt = Self::map_lang_tgt(target)?;

        #[derive(serde::Deserialize)]
        struct GoogleResp {
            data: GoogleData,
        }

        #[derive(serde::Deserialize)]
        struct GoogleData {
            translations: Vec<GoogleTranslation>,
        }

        #[derive(serde::Deserialize)]
        struct GoogleTranslation {
            #[serde(rename = "translatedText")]
            translated_text: String,
        }

        let resp = self
            .client
            .post("https://translation.googleapis.com/language/translate/v2")
            .query(&[
                ("q", text),
                ("source", src),
                ("target", tgt),
                ("format", "text"),
                ("key", self.api_key.trim()),
            ])
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return Err(TranslationError::TranslationFailed(format!(
                "Google HTTP {}: {}",
                status.as_u16(),
                body
            )));
        }

        let parsed: GoogleResp = serde_json::from_str(&body)
            .map_err(|e| TranslationError::TranslationFailed(e.to_string()))?;

        parsed
            .data
            .translations
            .into_iter()
            .next()
            .map(|t| t.translated_text)
            .ok_or_else(|| TranslationError::TranslationFailed("Google: empty response".to_string()))
    }

    fn is_available(&self) -> bool {
        !self.api_key.trim().is_empty()
    }

    fn requires_api_key(&self) -> bool {
        true
    }
}
