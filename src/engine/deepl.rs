use super::{TranslationEngine, TranslationError};

pub struct DeepLEngine {
    api_key: String,
    client: reqwest::blocking::Client,
}

impl DeepLEngine {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn map_lang_src(src: &str) -> Result<&'static str, TranslationError> {
        match src.to_ascii_lowercase().as_str() {
            "pt" => Ok("PT"),
            "en" => Ok("EN"),
            "es" => Ok("ES"),
            other => Err(TranslationError::LanguageNotSupported(format!(
                "source={other}"
            ))),
        }
    }

    fn map_lang_tgt(tgt: &str) -> Result<&'static str, TranslationError> {
        match tgt.to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" => Ok("ZH"),
            other => Err(TranslationError::LanguageNotSupported(format!(
                "target={other}"
            ))),
        }
    }
}

impl TranslationEngine for DeepLEngine {
    fn name(&self) -> &str {
        "DeepL"
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

        #[derive(serde::Serialize)]
        struct DeepLReq<'a> {
            text: Vec<&'a str>,
            source_lang: &'a str,
            target_lang: &'a str,
        }

        #[derive(serde::Deserialize)]
        struct DeepLResp {
            translations: Vec<DeepLTranslation>,
        }

        #[derive(serde::Deserialize)]
        struct DeepLTranslation {
            text: String,
        }

        let req = DeepLReq {
            text: vec![text],
            source_lang: src,
            target_lang: tgt,
        };

        let resp = self
            .client
            .post("https://api-free.deepl.com/v2/translate")
            .header("Authorization", format!("DeepL-Auth-Key {}", self.api_key.trim()))
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(5))
            .json(&req)
            .send()
            .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| TranslationError::NetworkError(e.to_string()))?;

        if !status.is_success() {
            return match status.as_u16() {
                403 => Err(TranslationError::TranslationFailed(
                    "DeepL: invalid API key (403)".to_string(),
                )),
                456 => Err(TranslationError::TranslationFailed(
                    "DeepL: quota exceeded (456)".to_string(),
                )),
                code => Err(TranslationError::TranslationFailed(format!(
                    "DeepL HTTP {code}: {body}"
                ))),
            };
        }

        let parsed: DeepLResp = serde_json::from_str(&body)
            .map_err(|e| TranslationError::TranslationFailed(e.to_string()))?;

        parsed
            .translations
            .into_iter()
            .next()
            .map(|t| t.text)
            .ok_or_else(|| TranslationError::TranslationFailed("DeepL: empty response".to_string()))
    }

    fn is_available(&self) -> bool {
        !self.api_key.trim().is_empty()
    }

    fn requires_api_key(&self) -> bool {
        true
    }
}
