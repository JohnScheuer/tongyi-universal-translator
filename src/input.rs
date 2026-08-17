#[cfg(windows)]
mod imp {
    use anyhow::{anyhow, Result};
    use std::mem::size_of;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
        KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_A, VK_C, VK_CONTROL, VK_MENU, VK_SHIFT, VK_V,
    };

    fn is_key_down(vk: VIRTUAL_KEY) -> bool {
        unsafe { (GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000) != 0 }
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

    fn send_inputs(inputs: &[INPUT]) -> Result<()> {
        // windows-rs 0.58: SendInput(&[INPUT], cbsize) -> u32
        let sent = unsafe { SendInput(inputs, size_of::<INPUT>() as i32) };
        if sent != inputs.len() as u32 {
            return Err(anyhow!(
                "SendInput failed/sent partial input: {sent}/{}",
                inputs.len()
            ));
        }
        Ok(())
    }

    pub fn send_key_down(vk: VIRTUAL_KEY) -> Result<()> {
        send_inputs(&[key_input(vk, KEYBD_EVENT_FLAGS(0))])
    }

    pub fn send_key_up(vk: VIRTUAL_KEY) -> Result<()> {
        send_inputs(&[key_input(vk, KEYEVENTF_KEYUP)])
    }

    /// Espera o usuário soltar modificadores físicos (Shift/Ctrl/Alt), com timeout.
    /// Isso reduz casos onde Ctrl+V vira Ctrl+Shift+V durante o hotkey.
    pub fn wait_for_modifiers_released(timeout_ms: u64) -> Result<()> {
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        while Instant::now() < deadline {
            let shift = is_key_down(VK_SHIFT);
            let ctrl = is_key_down(VK_CONTROL);
            let alt = is_key_down(VK_MENU);

            if !shift && !ctrl && !alt {
                return Ok(());
            }
            sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    /// Tenta soltar Shift/Ctrl/Alt "stuck" (por hotkey global / estado preso).
    pub fn release_stuck_modifiers() -> Result<()> {
        if is_key_down(VK_SHIFT) {
            let _ = send_key_up(VK_SHIFT);
            sleep(Duration::from_millis(5));
        }
        if is_key_down(VK_CONTROL) {
            let _ = send_key_up(VK_CONTROL);
            sleep(Duration::from_millis(5));
        }
        if is_key_down(VK_MENU) {
            let _ = send_key_up(VK_MENU);
            sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    pub fn send_key_combo(modifiers: &[VIRTUAL_KEY], key: VIRTUAL_KEY) -> Result<()> {
        // tenta evitar que Shift do hotkey contamine o combo
        let _ = wait_for_modifiers_released(200);
        let _ = release_stuck_modifiers();

        let mut seq: Vec<INPUT> = Vec::with_capacity(modifiers.len() * 2 + 2);

        for &m in modifiers {
            seq.push(key_input(m, KEYBD_EVENT_FLAGS(0)));
        }

        seq.push(key_input(key, KEYBD_EVENT_FLAGS(0)));
        seq.push(key_input(key, KEYEVENTF_KEYUP));

        for &m in modifiers.iter().rev() {
            seq.push(key_input(m, KEYEVENTF_KEYUP));
        }

        send_inputs(&seq)?;
        Ok(())
    }

    pub fn simulate_select_all() -> Result<()> {
        send_key_combo(&[VK_CONTROL], VK_A)
    }

    pub fn simulate_copy() -> Result<()> {
        send_key_combo(&[VK_CONTROL], VK_C)?;
        sleep(Duration::from_millis(70)); // clipboard update
        Ok(())
    }

    pub fn simulate_paste() -> Result<()> {
        send_key_combo(&[VK_CONTROL], VK_V)
    }
}

#[cfg(not(windows))]
mod imp {
    use anyhow::{anyhow, Result};

    #[allow(non_camel_case_types)]
    pub type VIRTUAL_KEY = u16;

    pub fn send_key_down(_vk: VIRTUAL_KEY) -> Result<()> {
        Err(anyhow!("Input simulation not supported on non-Windows targets."))
    }

    pub fn send_key_up(_vk: VIRTUAL_KEY) -> Result<()> {
        Err(anyhow!("Input simulation not supported on non-Windows targets."))
    }

    pub fn wait_for_modifiers_released(_timeout_ms: u64) -> Result<()> {
        Ok(())
    }

    pub fn release_stuck_modifiers() -> Result<()> {
        Ok(())
    }

    pub fn send_key_combo(_modifiers: &[VIRTUAL_KEY], _key: VIRTUAL_KEY) -> Result<()> {
        Err(anyhow!("Input simulation not supported on non-Windows targets."))
    }

    pub fn simulate_select_all() -> Result<()> {
        Err(anyhow!("Input simulation not supported on non-Windows targets."))
    }

    pub fn simulate_copy() -> Result<()> {
        Err(anyhow!("Input simulation not supported on non-Windows targets."))
    }

    pub fn simulate_paste() -> Result<()> {
        Err(anyhow!("Input simulation not supported on non-Windows targets."))
    }
}

pub use imp::{
    release_stuck_modifiers, send_key_combo, send_key_down, send_key_up, simulate_copy,
    simulate_paste, simulate_select_all, wait_for_modifiers_released,
};
