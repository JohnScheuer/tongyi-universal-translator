use tongyi_translator::engine::windows_lp::WindowsLPEngine;
use tongyi_translator::engine::TranslationEngine;

#[test]
fn test_windows_lp_placeholder_translates_common_phrases() {
    let e = WindowsLPEngine::new();
    assert_eq!(e.translate("Hello", "en", "zh-cn").unwrap(), "你好");
    assert_eq!(e.translate("Bom dia", "pt", "zh-cn").unwrap(), "早上好");
}
