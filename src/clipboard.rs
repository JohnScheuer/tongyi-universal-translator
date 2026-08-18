// src/clipboard.rs
use anyhow::{anyhow, Context, Result};
use std::{
    mem::size_of,
    ptr,
    time::{Duration, Instant},
};
use windows::Win32::{
    Foundation::{GetLastError, HANDLE, HWND, HGLOBAL},
    System::{
        Com::IDataObject,
        DataExchange::{CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData},
        Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
        Ole::{OleGetClipboard, OleInitialize, OleSetClipboard},
    },
};

// CF_UNICODETEXT = 13 (Win32)
const CF_UNICODETEXT: u32 = 13;

#[link(name = "kernel32")]
extern "system" {
    fn GlobalFree(hmem: HGLOBAL) -> HGLOBAL;
}

pub struct ClipboardBackup {
    data_object: IDataObject,
}

fn win_last_error(msg: &str) -> anyhow::Error {
    let e = unsafe { GetLastError() };
    anyhow!("{msg} (GetLastError={})", e.0)
}

fn ensure_ole_initialized() -> Result<()> {
    unsafe { OleInitialize(None).ok().context("OleInitialize failed")? };
    Ok(())
}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open_retry(timeout: Duration) -> Result<Self> {
        let start = Instant::now();
        loop {
            let r = unsafe { OpenClipboard(HWND(ptr::null_mut())) };
            if r.is_ok() {
                return Ok(Self);
            }
            if start.elapsed() >= timeout {
                return Err(win_last_error("OpenClipboard failed (timeout)"));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

/// Backup do clipboard inteiro via OLE IDataObject.
pub fn backup() -> Result<ClipboardBackup> {
    ensure_ole_initialized()?;

    let start = Instant::now();
    loop {
        match unsafe { OleGetClipboard() } {
            Ok(data_object) => return Ok(ClipboardBackup { data_object }),
            Err(e) => {
                if start.elapsed() >= Duration::from_millis(250) {
                    return Err(anyhow!("OleGetClipboard failed: {e:?}"));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

pub fn restore(backup: ClipboardBackup) -> Result<()> {
    ensure_ole_initialized()?;
    unsafe { OleSetClipboard(&backup.data_object).ok().context("OleSetClipboard failed")? };
    Ok(())
}

pub fn clear() -> Result<()> {
    let _g = ClipboardGuard::open_retry(Duration::from_millis(250))?;
    unsafe { EmptyClipboard().ok().context("EmptyClipboard failed")? };
    Ok(())
}

pub fn read_text() -> Result<String> {
    let _g = ClipboardGuard::open_retry(Duration::from_millis(250))?;

    let handle: HANDLE = match unsafe { GetClipboardData(CF_UNICODETEXT) } {
        Ok(h) => h,
        Err(_) => return Ok(String::new()),
    };

    if handle.0.is_null() {
        return Ok(String::new());
    }

    let hglobal = HGLOBAL(handle.0);
    let p = unsafe { GlobalLock(hglobal) } as *const u16;
    if p.is_null() {
        return Err(win_last_error("GlobalLock failed"));
    }

    let mut len = 0usize;
    unsafe {
        while *p.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(p, len);
        let s = String::from_utf16_lossy(slice);
        let _ = GlobalUnlock(hglobal);
        Ok(s)
    }
}

pub fn write_text(text: &str) -> Result<()> {
    let _g = ClipboardGuard::open_retry(Duration::from_millis(250))?;
    unsafe { EmptyClipboard().ok().context("EmptyClipboard failed")? };

    unsafe {
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        wide.push(0);

        let bytes = wide.len() * size_of::<u16>();
        let hmem = GlobalAlloc(GMEM_MOVEABLE, bytes).context("GlobalAlloc failed")?;
        if hmem.0.is_null() {
            return Err(win_last_error("GlobalAlloc returned NULL"));
        }

        let p = GlobalLock(hmem) as *mut u16;
        if p.is_null() {
            let _ = GlobalFree(hmem);
            return Err(win_last_error("GlobalLock failed"));
        }

        ptr::copy_nonoverlapping(wide.as_ptr(), p, wide.len());
        let _ = GlobalUnlock(hmem);

        // Se OK, o sistema assume ownership do HGLOBAL.
        if SetClipboardData(CF_UNICODETEXT, HANDLE(hmem.0)).is_err() {
            let _ = GlobalFree(hmem);
            return Err(win_last_error("SetClipboardData(CF_UNICODETEXT) failed"));
        }

        Ok(())
    }
}

pub fn read_text_retry(max_wait: Duration) -> Result<String> {
    let start = Instant::now();
    loop {
        let s = read_text()?;
        if !s.is_empty() {
            return Ok(s);
        }
        if start.elapsed() >= max_wait {
            return Ok(String::new());
        }
        std::thread::sleep(Duration::from_millis(15));
    }
}
