use tongyi_translator::config::AppConfig;
use tongyi_translator::engine::router;

#[test]
fn test_router_returns_correct_engine_name() {
    let mut cfg = AppConfig::default();

    cfg.general.active_engine = "windows_lp".to_string();
    assert_eq!(router::get_engine(&cfg).name(), "Windows LP");

    cfg.general.active_engine = "marian".to_string();
    assert_eq!(router::get_engine(&cfg).name(), "MarianMT");

    cfg.general.active_engine = "deepl".to_string();
    assert_eq!(router::get_engine(&cfg).name(), "DeepL");

    cfg.general.active_engine = "google".to_string();
    assert_eq!(router::get_engine(&cfg).name(), "Google Translate");

    cfg.general.active_engine = "unknown".to_string();
    assert_eq!(router::get_engine(&cfg).name(), "Windows LP");
}
