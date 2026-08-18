#![cfg(windows)]

use anyhow::{anyhow, Context, Result};
use std::mem::size_of;
use std::ptr;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, POINT, WPARAM, LPARAM};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIF_SHOWTIP, NIM_ADD, NIM_DELETE,
    NIM_MODIFY, NIM_SETVERSION, NIN_SELECT, NOTIFYICONDATAW, NOTIFYICON_VERSION_4,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, LoadIconW, PostMessageW,
    SetForegroundWindow, TrackPopupMenu, HICON, HMENU, IDI_APPLICATION, MENU_ITEM_FLAGS,
    MF_CHECKED, MF_DISABLED, MF_GRAYED, MF_SEPARATOR, MF_STRING, TPM_LEFTALIGN, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, WM_CONTEXTMENU, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP, WM_USER,
};

use crate::config::AppConfig;

pub const WM_TRAYICON: u32 = WM_USER + 1;
pub const TRAY_UID: u32 = 1;

// =========================================================
// Menu IDs (exportados para o main.rs não duplicar números)
// =========================================================
pub const ID_ENGINE_WINDOWS_LP: u16 = 1001;
pub const ID_ENGINE_MARIAN: u16 = 1002;
pub const ID_ENGINE_DEEPL: u16 = 1003;
pub const ID_ENGINE_GOOGLE: u16 = 1004;

pub const ID_SRC_PT: u16 = 2001;
pub const ID_SRC_EN: u16 = 2002;
pub const ID_SRC_ES: u16 = 2003;

pub const ID_EXIT: u16 = 9999;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn fill_tip(buf: &mut [u16], s: &str) {
    buf.fill(0);
    for (i, ch) in s.encode_utf16().take(buf.len().saturating_sub(1)).enumerate() {
        buf[i] = ch;
    }
}

fn icon_for_state(_active: bool) -> HICON {
    unsafe { LoadIconW(HINSTANCE::default(), IDI_APPLICATION) }
        .unwrap_or(HICON(ptr::null_mut()))
}

fn menu_flags(checked: bool) -> MENU_ITEM_FLAGS {
    if checked {
        MF_STRING | MF_CHECKED
    } else {
        MF_STRING
    }
}

pub struct Tray {
    hwnd: HWND,
}

impl Tray {
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    pub fn add(&self, config: &AppConfig, active: bool) -> Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = TRAY_UID;
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = icon_for_state(active);

            let tip = format!(
                "TongYi - {} ({})",
                if active { "Active" } else { "Inactive" },
                config.general.active_engine
            );
            fill_tip(&mut nid.szTip, &tip);

            if !Shell_NotifyIconW(NIM_ADD, &mut nid).as_bool() {
                return Err(anyhow!("Shell_NotifyIconW NIM_ADD failed"));
            }

            // VERSION_4 fix (confirmado)
            nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;
            let _ = Shell_NotifyIconW(NIM_SETVERSION, &mut nid);

            nid.uFlags = NIF_TIP | NIF_SHOWTIP;
            if !Shell_NotifyIconW(NIM_MODIFY, &mut nid).as_bool() {
                return Err(anyhow!("Shell_NotifyIconW NIM_MODIFY tip failed"));
            }
        }
        Ok(())
    }

    pub fn remove(&self) {
        unsafe {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = TRAY_UID;
            let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);
        }
    }

    pub fn update_tip(&self, config: &AppConfig, active: bool) -> Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = TRAY_UID;
            nid.uFlags = NIF_TIP | NIF_SHOWTIP;

            let tip = format!(
                "TongYi - {} ({})",
                if active { "Active" } else { "Inactive" },
                config.general.active_engine
            );
            fill_tip(&mut nid.szTip, &tip);

            if !Shell_NotifyIconW(NIM_MODIFY, &mut nid).as_bool() {
                return Err(anyhow!("Shell_NotifyIconW NIM_MODIFY failed"));
            }
        }
        Ok(())
    }

    pub fn update_icon(&self, active: bool) -> Result<()> {
        unsafe {
            let mut nid = NOTIFYICONDATAW::default();
            nid.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd;
            nid.uID = TRAY_UID;
            nid.uFlags = NIF_ICON;
            nid.hIcon = icon_for_state(active);

            if !Shell_NotifyIconW(NIM_MODIFY, &mut nid).as_bool() {
                return Err(anyhow!("Shell_NotifyIconW NIM_MODIFY icon failed"));
            }
        }
        Ok(())
    }

    // =========================================================
    // NOTIFYICON_VERSION_4 format:
    //   LOWORD(LPARAM) = notification code
    //   HIWORD(LPARAM) = icon ID
    //   WPARAM = anchor coordinates
    // =========================================================
    pub fn extract_notify_code(lparam: LPARAM) -> u32 {
        (lparam.0 as u32) & 0xFFFF
    }

    pub fn extract_icon_id(lparam: LPARAM) -> u32 {
        ((lparam.0 as u32) >> 16) & 0xFFFF
    }

    pub fn extract_anchor_x(wparam: WPARAM) -> i32 {
        ((wparam.0 as u32) & 0xFFFF) as i16 as i32
    }

    pub fn extract_anchor_y(wparam: WPARAM) -> i32 {
        (((wparam.0 as u32) >> 16) & 0xFFFF) as i16 as i32
    }

    pub fn is_left_click(code: u32) -> bool {
        code == NIN_SELECT || code == WM_LBUTTONUP
    }

    pub fn is_right_click(code: u32) -> bool {
        code == WM_CONTEXTMENU || code == WM_RBUTTONUP
    }

    fn append(menu: HMENU, flags: MENU_ITEM_FLAGS, id: u16, text: PCWSTR) -> Result<()> {
        unsafe { AppendMenuW(menu, flags, id as usize, text) }.context("AppendMenuW")
    }

    fn append_disabled_title(menu: HMENU, title: &str) -> Result<()> {
        let w = to_wide(title);
        Self::append(
            menu,
            MF_STRING | MF_DISABLED | MF_GRAYED,
            0,
            PCWSTR(w.as_ptr()),
        )
    }

    fn build_menu(&self, config: &AppConfig) -> Result<HMENU> {
        unsafe {
            let menu = CreatePopupMenu().context("CreatePopupMenu")?;

            Self::append_disabled_title(menu, "Engine:")?;

            let s1 = to_wide("Windows Language Pack");
            Self::append(
                menu,
                menu_flags(config.general.active_engine == "windows_lp"),
                ID_ENGINE_WINDOWS_LP,
                PCWSTR(s1.as_ptr()),
            )?;

            let s2 = to_wide("MarianMT (Offline)");
            Self::append(
                menu,
                menu_flags(config.general.active_engine == "marian"),
                ID_ENGINE_MARIAN,
                PCWSTR(s2.as_ptr()),
            )?;

            let s3 = to_wide("DeepL (API)");
            Self::append(
                menu,
                menu_flags(config.general.active_engine == "deepl"),
                ID_ENGINE_DEEPL,
                PCWSTR(s3.as_ptr()),
            )?;

            let s4 = to_wide("Google Translate (API)");
            Self::append(
                menu,
                menu_flags(config.general.active_engine == "google"),
                ID_ENGINE_GOOGLE,
                PCWSTR(s4.as_ptr()),
            )?;

            Self::append(menu, MF_SEPARATOR, 0, w!(""))?;
            Self::append_disabled_title(menu, "Source language:")?;

            let s5 = to_wide("Portugues (pt)");
            Self::append(
                menu,
                menu_flags(config.general.source_language == "pt"),
                ID_SRC_PT,
                PCWSTR(s5.as_ptr()),
            )?;

            let s6 = to_wide("English (en)");
            Self::append(
                menu,
                menu_flags(config.general.source_language == "en"),
                ID_SRC_EN,
                PCWSTR(s6.as_ptr()),
            )?;

            let s7 = to_wide("Espanol (es)");
            Self::append(
                menu,
                menu_flags(config.general.source_language == "es"),
                ID_SRC_ES,
                PCWSTR(s7.as_ptr()),
            )?;

            Self::append(menu, MF_SEPARATOR, 0, w!(""))?;

            let s8 = to_wide("Exit");
            Self::append(menu, MF_STRING, ID_EXIT, PCWSTR(s8.as_ptr()))?;

            Ok(menu)
        }
    }

    pub fn show_menu_and_get_command(&self, config: &AppConfig) -> Result<Option<u16>> {
        unsafe {
            let menu = self.build_menu(config)?;

            let mut pt = POINT::default();
            GetCursorPos(&mut pt).context("GetCursorPos")?;

            let _ = SetForegroundWindow(self.hwnd);

            let cmd = TrackPopupMenu(
                menu,
                TPM_LEFTALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
                pt.x,
                pt.y,
                0,
                self.hwnd,
                None,
            );

            let _ = DestroyMenu(menu);
            let _ = PostMessageW(self.hwnd, WM_NULL, WPARAM(0), LPARAM(0));

            if cmd.0 == 0 {
                return Ok(None);
            }
            Ok(Some(cmd.0 as u16))
        }
    }
}
