#![cfg(windows)]

use anyhow::{anyhow, Result};
use windows::Win32::Foundation::{BOOL, HWND};

use crate::config::AppConfig;

pub const HOTKEY_ID_TRANSLATE: i32 = 1;

// MOD_* (WinUser.h)
const MOD_ALT: u32 = 0x0001;
const MOD_CONTROL: u32 = 0x0002;
const MOD_SHIFT: u32 = 0x0004;

// VK_RETURN
const VK_RETURN: u32 = 0x0D;

#[link(name = "user32")]
extern "system" {
    fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: u32, vk: u32) -> BOOL;
    fn UnregisterHotKey(hWnd: HWND, id: i32) -> BOOL;
}

fn modifier_flags(cfg: &AppConfig) -> u32 {
    match cfg.general.hotkey_modifier.as_str() {
        "shift" => MOD_SHIFT,
        "ctrl+shift" => MOD_CONTROL | MOD_SHIFT,
        "alt" => MOD_ALT,
        _ => MOD_SHIFT,
    }
}

fn vk_code(cfg: &AppConfig) -> u32 {
    match cfg.general.hotkey_key.as_str() {
        "enter" => VK_RETURN,
        _ => VK_RETURN,
    }
}

pub fn register_hotkey(hwnd: HWND, cfg: &AppConfig) -> Result<()> {
    let mods = modifier_flags(cfg);
    let vk = vk_code(cfg);

    let ok = unsafe { RegisterHotKey(hwnd, HOTKEY_ID_TRANSLATE, mods, vk) }.as_bool();
    if !ok {
        return Err(anyhow!("RegisterHotKey failed (mods=0x{:X}, vk=0x{:X})", mods, vk));
    }
    Ok(())
}

pub fn unregister_hotkey(hwnd: HWND) {
    unsafe {
        let _ = UnregisterHotKey(hwnd, HOTKEY_ID_TRANSLATE);
    }
}
