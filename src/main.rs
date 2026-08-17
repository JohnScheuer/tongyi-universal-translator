#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use anyhow::Result;

#[cfg(not(windows))]
fn main() -> Result<()> {
    let _ = env_logger::try_init();
    println!("TongYi Translator v0.1.0 (non-Windows stub)");
    Ok(())
}

#[cfg(windows)]
fn main() -> Result<()> {
    use std::sync::atomic::{AtomicBool, Ordering};

    use anyhow::{anyhow, Context};

    use tongyi_translator::config::AppConfig;
    use tongyi_translator::{hotkey, translator};
    use tongyi_translator::ui::notification;
    use tongyi_translator::ui::tray::{Tray, TRAY_UID, WM_TRAYICON};

    use windows::core::w;
    use windows::Win32::Foundation::{HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::Shell::IsUserAnAdmin;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, MessageBoxW,
        PostQuitMessage, RegisterClassW, ShowWindow, TranslateMessage, MB_ICONWARNING,
        MB_OK, MSG, SW_HIDE, WINDOW_EX_STYLE, WM_DESTROY, WM_HOTKEY, WNDCLASSW,
        WS_OVERLAPPEDWINDOW,
    };

    static TRANSLATING: AtomicBool = AtomicBool::new(false);
    static ACTIVE: AtomicBool = AtomicBool::new(true);

    extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match msg {
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }

                WM_TRAYICON => {
                    // VERSION_4 format:
                    //   LOWORD(LPARAM) = notification code
                    //   HIWORD(LPARAM) = icon ID
                    //   WPARAM = anchor coordinates
                    let icon_id = Tray::extract_icon_id(lparam);
                    let notify_code = Tray::extract_notify_code(lparam);

                    if icon_id != TRAY_UID {
                        return LRESULT(0);
                    }

                    let mut cfg = match AppConfig::load_or_create("config.toml") {
                        Ok(c) => c,
                        Err(_) => AppConfig::default(),
                    };

                    let tray = Tray::new(hwnd);

                    // Left click: toggle active/inactive
                    if Tray::is_left_click(notify_code) {
                        let new_active = !ACTIVE.load(Ordering::SeqCst);
                        ACTIVE.store(new_active, Ordering::SeqCst);
                        cfg.general.active = new_active;
                        let _ = cfg.save("config.toml");
                        let _ = tray.update_tip(&cfg, new_active);
                        let _ = tray.update_icon(new_active);
                        return LRESULT(0);
                    }

                    // Right click: show context menu
                    if Tray::is_right_click(notify_code) {
                        match tray.show_menu_and_get_command(&cfg) {
                            Ok(Some(cmd)) => {
                                match cmd {
                                    1001 => cfg.general.active_engine = "windows_lp".to_string(),
                                    1002 => cfg.general.active_engine = "marian".to_string(),
                                    1003 => cfg.general.active_engine = "deepl".to_string(),
                                    1004 => cfg.general.active_engine = "google".to_string(),
                                    2001 => cfg.general.source_language = "pt".to_string(),
                                    2002 => cfg.general.source_language = "en".to_string(),
                                    2003 => cfg.general.source_language = "es".to_string(),
                                    9999 => {
                                        tray.remove();
                                        hotkey::unregister_hotkey(hwnd);
                                        PostQuitMessage(0);
                                        return LRESULT(0);
                                    }
                                    _ => {}
                                }
                                let _ = cfg.save("config.toml");
                                let _ = tray.update_tip(&cfg, ACTIVE.load(Ordering::SeqCst));
                            }
                            Ok(None) => {}
                            Err(_) => {}
                        }
                        return LRESULT(0);
                    }

                    LRESULT(0)
                }

                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    fn create_hidden_window() -> Result<HWND> {
        let hmodule: HMODULE =
            unsafe { GetModuleHandleW(None) }.context("GetModuleHandleW")?;
        let hinstance = HINSTANCE(hmodule.0);

        let class_name = w!("TongyiHiddenWindow");

        let wc = WNDCLASSW {
            hInstance: hinstance,
            lpszClassName: class_name,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = unsafe { RegisterClassW(&wc) };
        if atom == 0 {
            return Err(anyhow!("RegisterClassW failed"));
        }

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                w!("Tongyi"),
                WS_OVERLAPPEDWINDOW,
                0, 0, 300, 200,
                HWND::default(),
                None,
                hinstance,
                None,
            )
        }
        .context("CreateWindowExW")?;

        unsafe {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }

        Ok(hwnd)
    }

    let _ = env_logger::try_init();

    let elevated = unsafe { IsUserAnAdmin().as_bool() };
    if elevated {
        unsafe {
            let _ = MessageBoxW(
                HWND::default(),
                w!("TongYi: Do not run as Administrator.\nOpen a normal PowerShell and run again."),
                w!("TongYi Translator"),
                MB_OK | MB_ICONWARNING,
            );
        }
        return Ok(());
    }

    let mut cfg = AppConfig::load_or_create("config.toml")?;
    ACTIVE.store(cfg.general.active, Ordering::SeqCst);

    let hwnd = create_hidden_window()?;
    let tray = Tray::new(hwnd);

    tray.add(&cfg, ACTIVE.load(Ordering::SeqCst))?;
    hotkey::register_hotkey(hwnd, &cfg)?;

    // Main message loop
    unsafe {
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, HWND::default(), 0, 0);
            if ret.0 == -1 {
                break Err(anyhow!("GetMessageW failed"));
            }
            if ret.0 == 0 {
                break Ok(());
            }

            if msg.message == WM_HOTKEY {
                if ACTIVE.load(Ordering::SeqCst)
                    && TRANSLATING
                        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                        .is_ok()
                {
                    match AppConfig::load_or_create("config.toml") {
                        Ok(loaded) => cfg = loaded,
                        Err(e) => {
                            let _ = notification::show_error(&cfg, &format!("Config: {e:#}"));
                        }
                    }

                    match translator::translate_active_field(&cfg) {
                        Ok(_) => {}
                        Err(e) => {
                            let _ = notification::show_error(&cfg, &format!("{e:#}"));
                        }
                    }

                    TRANSLATING.store(false, Ordering::SeqCst);
                }
            }

            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }?;

    tray.remove();
    hotkey::unregister_hotkey(hwnd);
    Ok(())
}
