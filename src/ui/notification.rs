use crate::config::AppConfig;
use crate::translator::TranslationResult;

#[cfg(windows)]
pub fn show_translation(config: &AppConfig, r: &TranslationResult) -> anyhow::Result<()> {
    use winrt_notification::{Duration, Toast};

    if !config.ui.show_notifications {
        return Ok(());
    }

    const APP_ID: &str = "Tongyi.Translator";

    let title = "TōngYì 通译";
    let body_line = if config.ui.show_original_in_notification {
        format!("{} → {}", r.original.trim(), r.translated.trim())
    } else {
        r.translated.trim().to_string()
    };
    let footer = format!("Engine: {} • {}ms", r.engine_used, r.duration_ms);

    // Builder pattern: cada chamada consome self e retorna novo Toast
    let toast = Toast::new(APP_ID)
        .title(title)
        .text1(&body_line)
        .text2(&footer)
        .duration(Duration::Short);

    let _ = toast.show();
    Ok(())
}

#[cfg(windows)]
pub fn show_error(config: &AppConfig, msg: &str) -> anyhow::Result<()> {
    use winrt_notification::{Duration, Toast};

    if !config.ui.show_notifications {
        return Ok(());
    }

    const APP_ID: &str = "Tongyi.Translator";

    let title = "TōngYì 通译 — Error";
    let body = msg.to_string();

    let toast = Toast::new(APP_ID)
        .title(title)
        .text1(&body)
        .duration(Duration::Short);

    let _ = toast.show();
    Ok(())
}

#[cfg(not(windows))]
pub fn show_translation(_config: &AppConfig, _r: &TranslationResult) -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
pub fn show_error(_config: &AppConfig, _msg: &str) -> anyhow::Result<()> {
    Ok(())
}
