#[cfg(windows)]
mod imp {
    use anyhow::{anyhow, Context, Result};
    use clipboard_win::{formats, get_clipboard, set_clipboard};
    use std::ptr;
    use std::thread::sleep;
    use std::time::Duration;

    use windows::Win32::Foundation::{HANDLE, HGLOBAL, HWND};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData, OpenClipboard,
        SetClipboardData,
    };
    use windows::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
    };

    const OPEN_RETRIES: usize = 80;
    const OPEN_RETRY_DELAY_MS: u64 = 25;

    #[derive(Debug)]
    struct ClipboardGuard;

    impl ClipboardGuard {
        fn open_with_retry() -> Result<Self> {
            let mut last_err: Option<anyhow::Error> = None;

            for _ in 0..OPEN_RETRIES {
                let res = unsafe { OpenClipboard(HWND(ptr::null_mut())) };
                match res {
                    Ok(()) => return Ok(Self),
                    Err(e) => {
                        last_err = Some(anyhow!(e).context("OpenClipboard failed"));
                        sleep(Duration::from_millis(OPEN_RETRY_DELAY_MS));
                    }
                }
            }

            Err(last_err.unwrap_or_else(|| anyhow!("OpenClipboard failed (unknown error)")))
        }
    }

    impl Drop for ClipboardGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseClipboard();
            }
        }
    }

    #[derive(Debug, Clone)]
    struct FormatBlob {
        format: u32,
        data: Vec<u8>,
    }

    #[derive(Debug, Clone)]
    pub struct ClipboardBackup {
        blobs: Vec<FormatBlob>,
    }

    pub fn read_text() -> Result<String> {
        // clipboard-win retorna ErrorCode (nao implementa StdError), então sem .context()
        get_clipboard(formats::Unicode)
            .map_err(|e| anyhow!("Failed to read Unicode text from clipboard: {e:?}"))
    }

    pub fn write_text(text: &str) -> Result<()> {
        set_clipboard(formats::Unicode, text.to_string())
            .map_err(|e| anyhow!("Failed to write Unicode text to clipboard: {e:?}"))
    }

    pub fn backup() -> Result<ClipboardBackup> {
        let _guard = ClipboardGuard::open_with_retry()?;

        let mut blobs: Vec<FormatBlob> = Vec::new();

        unsafe {
            let mut fmt = EnumClipboardFormats(0);
            while fmt != 0 {
                // GetClipboardData pode falhar para alguns formatos; nesse caso, só ignoramos.
                let handle: HANDLE = match GetClipboardData(fmt) {
                    Ok(h) => h,
                    Err(_) => {
                        fmt = EnumClipboardFormats(fmt);
                        continue;
                    }
                };

                if handle.0.is_null() {
                    fmt = EnumClipboardFormats(fmt);
                    continue;
                }

                // Tentamos tratar como HGLOBAL. Se não for HGLOBAL (ex: CF_BITMAP),
                // GlobalSize/GlobalLock normalmente retorna 0/null e nós pulamos.
                let hglobal = HGLOBAL(handle.0);

                let size = GlobalSize(hglobal);
                if size == 0 {
                    fmt = EnumClipboardFormats(fmt);
                    continue;
                }

                let p = GlobalLock(hglobal) as *const u8;
                if p.is_null() {
                    fmt = EnumClipboardFormats(fmt);
                    continue;
                }

                let slice = std::slice::from_raw_parts(p, size);
                blobs.push(FormatBlob {
                    format: fmt,
                    data: slice.to_vec(),
                });

                let _ = GlobalUnlock(hglobal);

                fmt = EnumClipboardFormats(fmt);
            }
        }

        Ok(ClipboardBackup { blobs })
    }

    pub fn restore(backup: ClipboardBackup) -> Result<()> {
        let _guard = ClipboardGuard::open_with_retry()?;

        unsafe {
            EmptyClipboard().context("EmptyClipboard failed")?;

            // Recria apenas os formatos que conseguimos clonar como HGLOBAL.
            // Isso evita crashes/heap corruption com formatos baseados em GDI handles.
            for blob in backup.blobs {
                let len = blob.data.len();

                let hmem: HGLOBAL = GlobalAlloc(GMEM_MOVEABLE, len)
                    .context("GlobalAlloc failed while restoring clipboard")?;

                let dst = GlobalLock(hmem) as *mut u8;
                if dst.is_null() {
                    return Err(anyhow!("GlobalLock failed while restoring clipboard"));
                }

                ptr::copy_nonoverlapping(blob.data.as_ptr(), dst, len);

                let _ = GlobalUnlock(hmem);

                // Em sucesso, o clipboard assume ownership do HGLOBAL.
                SetClipboardData(blob.format, HANDLE(hmem.0)).with_context(|| {
                    format!("SetClipboardData failed for format {}", blob.format)
                })?;
            }
        }

        Ok(())
    }
}

#[cfg(not(windows))]
mod imp {
    use anyhow::{anyhow, Result};

    #[derive(Debug, Clone)]
    pub struct ClipboardBackup;

    pub fn read_text() -> Result<String> {
        Err(anyhow!("Clipboard not supported on non-Windows targets."))
    }

    pub fn write_text(_text: &str) -> Result<()> {
        Err(anyhow!("Clipboard not supported on non-Windows targets."))
    }

    pub fn backup() -> Result<ClipboardBackup> {
        Err(anyhow!("Clipboard not supported on non-Windows targets."))
    }

    pub fn restore(_backup: ClipboardBackup) -> Result<()> {
        Err(anyhow!("Clipboard not supported on non-Windows targets."))
    }
}

pub use imp::{backup, read_text, restore, write_text, ClipboardBackup};
