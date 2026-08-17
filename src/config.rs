use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub engines: EngineConfigs,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GeneralConfig {
    /// master on/off do app (tray icon + hotkey continua, mas tradução é ignorada quando false)
    pub active: bool,

    /// "pt" | "en" | "es"
    pub source_language: String,
    /// "zh-cn"
    pub target_language: String,
    /// "windows_lp" | "marian" | "deepl" | "google"
    pub active_engine: String,
    /// "shift" | "ctrl+shift" | "alt"
    pub hotkey_modifier: String,
    /// "enter"
    pub hotkey_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct EngineConfigs {
    pub deepl: DeepLConfig,
    pub google: GoogleConfig,
    pub marian: MarianConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DeepLConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GoogleConfig {
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct MarianConfig {
    pub model_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct UiConfig {
    pub show_notifications: bool,
    pub notification_duration_ms: u64,
    pub show_original_in_notification: bool,
    pub log_translations: bool,
    pub log_file: PathBuf,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            engines: EngineConfigs::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            active: true,
            source_language: "pt".to_string(),
            target_language: "zh-cn".to_string(),
            active_engine: "windows_lp".to_string(),
            hotkey_modifier: "shift".to_string(),
            hotkey_key: "enter".to_string(),
        }
    }
}

impl Default for EngineConfigs {
    fn default() -> Self {
        Self {
            deepl: DeepLConfig::default(),
            google: GoogleConfig::default(),
            marian: MarianConfig::default(),
        }
    }
}

impl Default for DeepLConfig {
    fn default() -> Self {
        Self {
            api_key: "".to_string(),
        }
    }
}

impl Default for GoogleConfig {
    fn default() -> Self {
        Self {
            api_key: "".to_string(),
        }
    }
}

impl Default for MarianConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("./models"),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_notifications: true,
            notification_duration_ms: 2000,
            show_original_in_notification: true,
            log_translations: true,
            log_file: PathBuf::from("translations.log"),
        }
    }
}

impl AppConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let cfg: AppConfig = toml::from_str(&raw)
            .with_context(|| format!("Failed to parse TOML: {}", path.display()))?;

        Ok(cfg)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create config directory: {}", parent.display())
                })?;
            }
        }

        let raw = toml::to_string_pretty(self).context("Failed to serialize config as TOML")?;
        fs::write(path, raw)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Load config; if missing, write defaults and return them.
    pub fn load_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        match Self::load(path) {
            Ok(cfg) => Ok(cfg),
            Err(_err) if !path.exists() => {
                let cfg = Self::default();
                cfg.save(path).with_context(|| {
                    format!(
                        "Config not found; tried to create default at {}",
                        path.display()
                    )
                })?;
                Ok(cfg)
            }
            Err(err) => Err(err),
        }
    }
}
