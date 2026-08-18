TongYi Translator (通译) — Design Notes (V1)

1. Goals
- Provide a small, discreet Windows tray utility that can translate text in the currently focused input field.
- Triggered by a global hotkey (default Shift+Enter), it replaces the text in-place with its translation (default target zh-cn).
- V1 prioritizes reliability and user value over perfect elegance.

2. High-Level Architecture
The application is a single Windows process with:
- A hidden Win32 window to receive:
  - WM_HOTKEY
  - WM_TRAYICON (notify icon callbacks)
- A system tray icon and context menu for configuration.
- A translation “hot path” that orchestrates clipboard + input simulation + engine invocation.

Key modules
- main.rs: Win32 message loop, tray integration, hotkey integration, calls translator
- config.rs: persistent TOML configuration, defaults, load/save
- ui/tray.rs: tray icon + menu; NOTIFYICON_VERSION_4 event parsing
- clipboard.rs: Unicode clipboard read/write; full backup/restore via OLE IDataObject
- input.rs: SendInput key simulation (Ctrl+A, Ctrl+C, Ctrl+V), release_shift
- hotkey.rs: user32 RegisterHotKey/UnregisterHotKey via FFI
- translator.rs: orchestrates the full flow; uses engine router; includes safety measures
- engine/*: TranslationEngine trait + router + per-engine implementations

3. Critical Tray Design (NOTIFYICON_VERSION_4)
We use NOTIFYICON_VERSION_4 to avoid regressions and ensure correct tray message parsing.

Confirmed format with VERSION_4:
- notify_code = LOWORD(LPARAM)
- icon_id    = HIWORD(LPARAM)
- WPARAM contains anchor coordinates and should not be used for event type.

The tray module exposes helper methods to extract these fields and classify left/right clicks.

4. Configuration Design
Config file: config.toml at the project/app working directory.

Config fields used by V1:
- general.active: master on/off (hotkey still registered, but translation ignored)
- general.source_language: "pt" | "en" | "es"
- general.target_language: "zh-cn"
- general.active_engine: "windows_lp" | "marian" | "deepl" | "google"
- general.hotkey_modifier: "shift" | "ctrl+shift" | "alt"
- general.hotkey_key: currently "enter"
- engines.deepl.api_key: optional
- engines.google.api_key: optional
- engines.marian.model_path: path to local model folder
- ui.show_notifications: boolean

5. Translation Hot Path (End-to-End)
Purpose: replace the active field contents with the translation without corrupting user clipboard.

Flow in translator.rs:
- Guard against reentrancy using a global atomic TRANSLATING.
- Backup clipboard:
  - Use OleGetClipboard to get IDataObject (full-format clipboard backup).
- Neutralize Shift:
  - release_shift best-effort to prevent hotkey modifier from leaking into later key sequences.
- Capture target window:
  - Record GetForegroundWindow at hotkey time.
- Select + copy:
  - Ctrl+A then Ctrl+C, then read from clipboard (retry loop).
- Translate:
  - phrasebook-first for zh-cn for short greetings (deterministic)
  - otherwise: use selected engine via engine router
- Safety before paste:
  - If foreground window changed since hotkey, abort paste to avoid writing into wrong app.
- Write translation to clipboard.
- Refocus and replace:
  - SetForegroundWindow(target_hwnd)
  - Ctrl+A again
  - Ctrl+V (twice best-effort)
- Restore clipboard backup via OleSetClipboard in Drop guard.

Clipboard restore must run even if translation fails:
- Implemented using RAII (Drop guard).

6. Translation Engines (Abstraction)
Engine abstraction:
- trait TranslationEngine
  - name(): engine identifier string
  - translate(text, source, target) -> Result<String, TranslationError>
  - is_available() and requires_api_key() exist for future UX, but are not critical in V1 path yet.

Engine routing:
- engine/router.rs returns a boxed engine based on config.general.active_engine.

Engines in V1:
- Marian (offline):
  - implemented via a persistent Python worker (subprocess)
  - pivot path:
    - PT/ES -> EN: opus-mt-ROMANCE-en
    - EN -> ZH: opus-mt-en-zh
  - scripts:
    - scripts/marian_translate.py: worker protocol stdin/stdout JSON lines
      - uses ensure_ascii=True when dumping JSON to avoid Windows stdout encoding ('charmap') errors
    - scripts/marian_download_models.py: downloads full model snapshots locally (minimal allow_patterns)
- DeepL (API):
  - implemented; requires API key in config
- Google (API):
  - implemented; requires API key in config
- Windows Language Pack:
  - stubbed for V1 (EngineUnavailable) until a feasible integration path is confirmed

7. Python Worker Protocol (Marian)
Rationale:
- Keep Rust side lightweight and reliable for V1.
- Avoid heavy ML dependencies and slow startup in Rust.

Protocol:
- Each request is one JSON object per line on stdin:
  - { "text": "...", "source": "pt|en|es", "target": "zh-cn", "model_root": "..." }
- Each response is one JSON object per line on stdout:
  - { "ok": true, "translated": "..." }
  - { "ok": false, "error": "...", "trace": "..." }

Worker lifecycle:
- Rust engine spawns the worker lazily on first translation.
- Worker persists between translations to cache models.

Encoding:
- Worker outputs ASCII-safe JSON via json.dumps(..., ensure_ascii=True) to prevent UnicodeEncodeError on Windows consoles/pipes.

8. Reliability and UX Considerations
- Do not run as Administrator.
- Focus safety:
  - capture foreground window at hotkey, abort paste if focus changes during long translation.
- Timing:
  - small sleeps and clipboard read retry loops; later can be improved with more robust polling.
- Re-selection before paste:
  - required after slow engines; helps ensure replacement rather than insertion.
- Notifications:
  - should be short and professional; show errors like missing API key or engine unavailable.

9. Known Limitations (Accepted for V1)
- Marian pivot translation quality is imperfect for short greetings; phrasebook-first addresses common cases.
- First Marian translation can take several seconds due to model load.
- Translation currently runs on the message loop thread; recommended improvement is to offload to background thread.
- Windows Language Pack engine not implemented yet.

10. Next Engineering Steps
- Move translation execution off UI thread (std::thread::spawn) while keeping reentrancy guard.
- Implement engine availability checks (models present, python installed).
- Add better error surfaces:
  - "Python not found"
  - "Models missing"
  - "API key missing"
- Packaging:
  - release build
  - ship scripts/ folder
  - document Python deps + model download
  - include config.toml.example
