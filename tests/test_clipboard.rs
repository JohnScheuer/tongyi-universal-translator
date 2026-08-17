#![cfg(windows)]

use std::sync::{Mutex, OnceLock};
use tongyi_translator::clipboard;

fn clipboard_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    // Não falhar com PoisonError se algum teste anterior panicar
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[test]
fn test_write_read_roundtrip() {
    let _lock = clipboard_lock();

    let backup = clipboard::backup().unwrap();

    clipboard::write_text("hello").unwrap();
    let got = clipboard::read_text().unwrap();
    assert_eq!(got, "hello");

    clipboard::restore(backup).unwrap();
}

#[test]
fn test_backup_restore() {
    let _lock = clipboard_lock();

    let backup1 = clipboard::backup().unwrap();

    clipboard::write_text("original").unwrap();
    let backup2 = clipboard::backup().unwrap();

    clipboard::write_text("modified").unwrap();
    assert_eq!(clipboard::read_text().unwrap(), "modified");

    clipboard::restore(backup2).unwrap();
    assert_eq!(clipboard::read_text().unwrap(), "original");

    clipboard::restore(backup1).unwrap();
}

#[test]
fn test_unicode_chinese_characters() {
    let _lock = clipboard_lock();

    let backup = clipboard::backup().unwrap();

    clipboard::write_text("你好").unwrap();
    let got = clipboard::read_text().unwrap();
    assert_eq!(got, "你好");

    clipboard::restore(backup).unwrap();
}
