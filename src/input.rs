// src/input.rs
use anyhow::{anyhow, Result};
use std::mem::size_of;
use windows::Win32::{
    Foundation::GetLastError,
    UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_SHIFT,
    },
};

fn win_last_error(msg: &str) -> anyhow::Error {
    let e = unsafe { GetLastError() };
    anyhow!("{msg} (GetLastError={})", e.0)
}

fn key_input(vk: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send(inputs: &[INPUT]) -> Result<()> {
    let sent = unsafe { SendInput(inputs, size_of::<INPUT>() as i32) };
    if sent == 0 {
        return Err(win_last_error("SendInput failed"));
    }
    Ok(())
}

pub fn send_key_down(vk: VIRTUAL_KEY) -> Result<()> {
    send(&[key_input(vk, KEYBD_EVENT_FLAGS(0))])
}

pub fn send_key_up(vk: VIRTUAL_KEY) -> Result<()> {
    send(&[key_input(vk, KEYEVENTF_KEYUP)])
}

pub fn send_key_combo(modifier: VIRTUAL_KEY, key: VIRTUAL_KEY) -> Result<()> {
    send_key_down(modifier)?;
    send_key_down(key)?;
    send_key_up(key)?;
    send_key_up(modifier)?;
    Ok(())
}

pub fn release_shift() -> Result<()> {
    send_key_up(VK_SHIFT)
}

pub fn simulate_select_all() -> Result<()> {
    send_key_combo(VK_CONTROL, VIRTUAL_KEY(0x41)) // 'A'
}

pub fn simulate_copy() -> Result<()> {
    send_key_combo(VK_CONTROL, VIRTUAL_KEY(0x43)) // 'C'
}

pub fn simulate_paste() -> Result<()> {
    send_key_combo(VK_CONTROL, VIRTUAL_KEY(0x56)) // 'V'
}
