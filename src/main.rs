#![windows_subsystem = "windows"]

mod clipboard;
mod config;
mod engine; // <-- NECESSÁRIO (para crate::engine existir)
mod hotkey;
mod input;
mod translator;

mod ui {
    pub mod notification;
    pub mod tray;
}

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use windows::core::PCWSTR;
use windows::Win32::{
    Foundation::{GetLastError, HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Shell::{Shell_NotifyIconW, NIF_INFO, NIM_MODIFY, NIIF_ERROR, NIIF_INFO, NOTIFYICONDATAW},
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
            GetWindowLongPtrW, PostQuitMessage, RegisterClassW, SetWindowLongPtrW,
            TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
            GWLP_USERDATA, MSG, WM_CREATE, WM_DESTROY, WM_HOTKEY, WM_NCCREATE, WNDCLASSW,
            WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::config::AppConfig;
use crate::hotkey::HOTKEY_ID_TRANSLATE;
use crate::ui::tray::{
    Tray, TRAY_UID, WM_TRAYICON, ID_ENGINE_DEEPL, ID_ENGINE_GOOGLE, ID_ENGINE_MARIAN,
    ID_ENGINE_WINDOWS_LP, ID_EXIT, ID_SRC_EN, ID_SRC_ES, ID_SRC_PT,
};

const CLASS_NAME: &str = "TongYiTranslatorHiddenWindow";
const WINDOW_NAME: &str = "TongYi Translator";

struct AppState {
    cfg: AppConfig,
    config_path: PathBuf,
    tray: Option<Tray>,
}

fn last_error(msg: &str) -> anyhow::Error {
    let e = unsafe { GetLastError() };
    anyhow!("{msg} (GetLastError={})", e.0)
}

fn fill_fixed_utf16(dst: &mut [u16], s: &str) {
    dst.fill(0);
    for (i, ch) in s.encode_utf16().take(dst.len().saturating_sub(1)).enumerate() {
        dst[i] = ch;
    }
}

fn truncate_chars(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    s.chars().take(max_chars).collect::<String>()
}

fn build_balloon(hwnd: HWND, title: &str, msg: &str, is_error: bool) -> NOTIFYICONDATAW {
    let mut nid = NOTIFYICONDATAW::default();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = TRAY_UID;
    nid.uFlags = NIF_INFO;

    let title = truncate_chars(title, 48);
    let msg = truncate_chars(msg, 200);

    fill_fixed_utf16(&mut nid.szInfoTitle, &title);
    fill_fixed_utf16(&mut nid.szInfo, &msg);

    nid.dwInfoFlags = if is_error { NIIF_ERROR } else { NIIF_INFO };
    nid
}

fn maybe_notify(hwnd: HWND, cfg: &AppConfig, title: &str, msg: &str, is_error: bool) {
    if !cfg.ui.show_notifications {
        return;
    }
    unsafe {
        let mut nid = build_balloon(hwnd, title, msg, is_error);
        let _ = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
    }
}

fn get_state(hwnd: HWND) -> Option<&'static AppState> {
    unsafe {
        let p = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const AppState;
        p.as_ref()
    }
}

fn get_state_mut(hwnd: HWND) -> Option<&'static mut AppState> {
    unsafe {
        let p = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
        p.as_mut()
    }
}

fn drop_state(hwnd: HWND) {
    unsafe {
        let p = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState;
        if !p.is_null() {
            let _boxed = Box::from_raw(p);
            let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        }
    }
}

fn apply_menu_command(hwnd: HWND, state: &mut AppState, cmd: u16) {
    let mut changed = false;

    match cmd {
        ID_ENGINE_WINDOWS_LP => {
            state.cfg.general.active_engine = "windows_lp".to_string();
            changed = true;
            maybe_notify(hwnd, &state.cfg, "TongYi", "Engine: Windows Language Pack", false);
        }
        ID_ENGINE_MARIAN => {
            state.cfg.general.active_engine = "marian".to_string();
            changed = true;
            maybe_notify(hwnd, &state.cfg, "TongYi", "Engine: MarianMT (Offline)", false);
        }
        ID_ENGINE_DEEPL => {
            state.cfg.general.active_engine = "deepl".to_string();
            changed = true;
            maybe_notify(hwnd, &state.cfg, "TongYi", "Engine: DeepL (API)", false);
        }
        ID_ENGINE_GOOGLE => {
            state.cfg.general.active_engine = "google".to_string();
            changed = true;
            maybe_notify(hwnd, &state.cfg, "TongYi", "Engine: Google Translate (API)", false);
        }

        ID_SRC_PT => {
            state.cfg.general.source_language = "pt".to_string();
            changed = true;
            maybe_notify(hwnd, &state.cfg, "TongYi", "Source: PT", false);
        }
        ID_SRC_EN => {
            state.cfg.general.source_language = "en".to_string();
            changed = true;
            maybe_notify(hwnd, &state.cfg, "TongYi", "Source: EN", false);
        }
        ID_SRC_ES => {
            state.cfg.general.source_language = "es".to_string();
            changed = true;
            maybe_notify(hwnd, &state.cfg, "TongYi", "Source: ES", false);
        }

        ID_EXIT => unsafe {
            let _ = DestroyWindow(hwnd);
            return;
        },

        _ => {}
    }

    if changed {
        let _ = state.cfg.save(&state.config_path);
        if let Some(tray) = state.tray.as_ref() {
            let _ = tray.update_tip(&state.cfg, state.cfg.general.active);
        }
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_NCCREATE => {
            let cs = lparam.0 as *const CREATESTRUCTW;
            if !cs.is_null() {
                let state_ptr = (*cs).lpCreateParams as *mut AppState;
                let _ = SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
            }
            return DefWindowProcW(hwnd, msg, wparam, lparam);
        }

        WM_CREATE => {
            if let Some(state) = get_state_mut(hwnd) {
                state.tray = Some(Tray::new(hwnd));

                if let Some(tray) = state.tray.as_ref() {
                    if let Err(e) = tray.add(&state.cfg, state.cfg.general.active) {
                        maybe_notify(hwnd, &state.cfg, "TongYi", &format!("Tray init failed: {e:#}"), true);
                    }
                }

                if let Err(e) = crate::hotkey::register_hotkey(hwnd, &state.cfg) {
                    maybe_notify(hwnd, &state.cfg, "TongYi", &format!("{e:#}"), true);
                }
            }
            return LRESULT(0);
        }

        _ if msg == WM_TRAYICON => {
            let notify_code = Tray::extract_notify_code(lparam);
            let icon_id = Tray::extract_icon_id(lparam);

            if icon_id != TRAY_UID {
                return LRESULT(0);
            }

            if Tray::is_left_click(notify_code) {
                if let Some(state) = get_state_mut(hwnd) {
                    state.cfg.general.active = !state.cfg.general.active;
                    let _ = state.cfg.save(&state.config_path);

                    if let Some(tray) = state.tray.as_ref() {
                        let _ = tray.update_icon(state.cfg.general.active);
                        let _ = tray.update_tip(&state.cfg, state.cfg.general.active);
                    }

                    maybe_notify(
                        hwnd,
                        &state.cfg,
                        "TongYi",
                        if state.cfg.general.active { "Translator enabled" } else { "Translator disabled" },
                        false,
                    );
                }
            } else if Tray::is_right_click(notify_code) {
                if let Some(state) = get_state_mut(hwnd) {
                    if let Some(tray) = state.tray.as_ref() {
                        match tray.show_menu_and_get_command(&state.cfg) {
                            Ok(Some(cmd)) => apply_menu_command(hwnd, state, cmd),
                            Ok(None) => {}
                            Err(e) => maybe_notify(hwnd, &state.cfg, "TongYi", &format!("Menu error: {e:#}"), true),
                        }
                    }
                }
            }

            return LRESULT(0);
        }

        WM_HOTKEY => {
            if wparam.0 as i32 == HOTKEY_ID_TRANSLATE {
                if let Some(state) = get_state(hwnd) {
                    if !state.cfg.general.active {
                        return LRESULT(0);
                    }

                    match crate::translator::translate_active_field(&state.cfg) {
                        Ok(r) => {
                            if state.cfg.ui.show_notifications {
                                maybe_notify(hwnd, &state.cfg, "TongYi", &format!("Translated ({} ms)", r.duration_ms), false);
                            }
                            let _ = r;
                        }
                        Err(e) => {
                            maybe_notify(hwnd, &state.cfg, "TongYi", &format!("{e:#}"), true);
                        }
                    }
                }
            }
            return LRESULT(0);
        }

        WM_DESTROY => {
            crate::hotkey::unregister_hotkey(hwnd);

            if let Some(state) = get_state_mut(hwnd) {
                if let Some(tray) = state.tray.as_ref() {
                    tray.remove();
                }
            }

            drop_state(hwnd);
            PostQuitMessage(0);
            return LRESULT(0);
        }

        _ => {}
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn main() -> Result<()> {
    let config_path = PathBuf::from("config.toml");
    let cfg = AppConfig::load_or_create(&config_path)
        .with_context(|| format!("load_or_create {}", config_path.display()))?;

    let state = Box::new(AppState {
        cfg,
        config_path,
        tray: None,
    });

    unsafe {
        let hinstance = GetModuleHandleW(None).context("GetModuleHandleW failed")?;

        let class_name: Vec<u16> = CLASS_NAME.encode_utf16().chain(Some(0)).collect();
        let window_name: Vec<u16> = WINDOW_NAME.encode_utf16().chain(Some(0)).collect();

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        if atom == 0 {
            return Err(last_error("RegisterClassW failed"));
        }

        let hwnd = CreateWindowExW(
            Default::default(),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_name.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            300,
            200,
            None,
            None,
            hinstance,
            Some(Box::into_raw(state) as *const core::ffi::c_void),
        )
        .context("CreateWindowExW failed")?;

        if hwnd.0.is_null() {
            return Err(anyhow!("CreateWindowExW returned NULL hwnd"));
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    Ok(())
}
