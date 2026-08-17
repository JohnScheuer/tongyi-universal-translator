#[cfg(windows)]
mod imp {
    use anyhow::{anyhow, Context, Result};

    use crate::config::AppConfig;

    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
        MOD_SHIFT, VK_RETURN,
    };

    pub const HOTKEY_ID_TRANSLATE: i32 = 1;

    fn parse_modifiers(s: &str) -> Result<HOT_KEY_MODIFIERS> {
        match s.trim().to_ascii_lowercase().as_str() {
            "shift" => Ok(MOD_SHIFT | MOD_NOREPEAT),
            "ctrl+shift" | "shift+ctrl" => Ok(MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT),
            "alt" => Ok(MOD_ALT | MOD_NOREPEAT),
            other => Err(anyhow!("Unsupported hotkey_modifier: {other}")),
        }
    }

    fn parse_key(s: &str) -> Result<u32> {
        match s.trim().to_ascii_lowercase().as_str() {
            "enter" | "return" => Ok(VK_RETURN.0 as u32),
            other => Err(anyhow!("Unsupported hotkey_key: {other}")),
        }
    }

    pub fn register_hotkey(hwnd: HWND, config: &AppConfig) -> Result<()> {
        let modifiers = parse_modifiers(&config.general.hotkey_modifier)?;
        let vk = parse_key(&config.general.hotkey_key)?;

        unsafe { RegisterHotKey(hwnd, HOTKEY_ID_TRANSLATE, modifiers, vk) }
            .context("RegisterHotKey failed (is another app using this hotkey?)")?;

        Ok(())
    }

    pub fn unregister_hotkey(hwnd: HWND) {
        unsafe {
            let _ = UnregisterHotKey(hwnd, HOTKEY_ID_TRANSLATE);
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use anyhow::{anyhow, Result};
    use crate::config::AppConfig;

    pub const HOTKEY_ID_TRANSLATE: i32 = 1;

    pub fn register_hotkey(_hwnd: usize, _config: &AppConfig) -> Result<()> {
        Err(anyhow!("Hotkey not supported on non-Windows targets."))
    }

    pub fn unregister_hotkey(_hwnd: usize) {}
}

pub use imp::{register_hotkey, unregister_hotkey, HOTKEY_ID_TRANSLATE};
