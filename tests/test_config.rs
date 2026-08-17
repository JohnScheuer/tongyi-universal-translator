use tongyi_translator::config::AppConfig;

#[test]
fn test_load_default_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let cfg = AppConfig::default();
    cfg.save(&path).unwrap();

    let loaded = AppConfig::load(&path).unwrap();
    assert_eq!(cfg, loaded);
}

#[test]
fn test_save_and_reload() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let mut cfg = AppConfig::default();
    cfg.general.source_language = "en".to_string();
    cfg.general.active_engine = "google".to_string();
    cfg.engines.google.api_key = "abc123".to_string();

    cfg.save(&path).unwrap();

    let loaded = AppConfig::load(&path).unwrap();
    assert_eq!(loaded.general.source_language, "en");
    assert_eq!(loaded.general.active_engine, "google");
    assert_eq!(loaded.engines.google.api_key, "abc123");
}

#[test]
fn test_missing_file_creates_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    assert!(!path.exists());
    let cfg = AppConfig::load_or_create(&path).unwrap();
    assert!(path.exists());

    assert_eq!(cfg, AppConfig::default());
}
