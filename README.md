# TongYi Translator (通译)

[![Platform](https://img.shields.io/badge/platform-Windows-0078D6)](https://www.microsoft.com/windows)
[![Language](https://img.shields.io/badge/language-Rust-black)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/badge/release-v0.1-informational)](#)
[![License](https://img.shields.io/badge/license-See%20LICENSE-green)](./LICENSE)

Windows tray utility that translates text in the currently focused input field via a global hotkey and replaces it in-place (target: Simplified Chinese, `zh-cn`).

Author: João Felipe De Souza  
GitHub: https://github.com/JohnScheuer

## Quick links

- Design notes: ./design.md
- Current state summary: ./summary.txt
- License: ./LICENSE

## What it does (V0.1)

- Runs in the Windows system tray (hidden window + Win32 message loop).
- Global hotkey (default): Shift+Enter
  - Select all (Ctrl+A)
  - Copy (Ctrl+C)
  - Translate (engine)
  - Paste back (Ctrl+V)
  - Restores the user clipboard (full clipboard backup/restore via OLE IDataObject)
- Tray UX:
  - Left-click: toggle Active/Inactive
  - Right-click: menu to select engine and source language, and Exit

## Translation engines (V0.1)

Implemented / working:
- MarianMT offline (Python worker), with pivot strategy:
  - PT/ES -> EN via opus-mt-ROMANCE-en
  - EN -> ZH via opus-mt-en-zh
- Phrasebook-first for short greetings (deterministic output)
  - Example: "bom dia" -> "早上好"

Implemented but requires API key:
- DeepL API (config.engines.deepl.api_key)
- Google Translate API (config.engines.google.api_key)

Stubbed (clean error for now):
- Windows Language Pack engine (not implemented in V0.1)

## Install (V0.1)

1) Download the release zip (or build from source).
2) Extract to a folder, for example:
   C:\Apps\tongyi-translator\
3) Run tongyi-translator.exe (non-admin).

Important:
- Do not run as Administrator.
- First offline translation can take several seconds (model load). After the first run it becomes faster.

## Offline Marian setup (recommended)

Prerequisites:
- Python 3.10+ installed (python in PATH)
- Python packages:
  - torch
  - transformers
  - sentencepiece
  - huggingface_hub

Install Python deps (PowerShell):
- python -m pip install --upgrade pip
- python -m pip install torch transformers sentencepiece huggingface_hub

Download models (online once; after that it works offline):
- cd C:\Apps\tongyi-translator\
- python .\scripts\marian_download_models.py --model-dir .\models --force

This will create:
- models\opus-mt-romance-en\
- models\opus-mt-en-zh\

Configure config.toml:
- [general]
  - active_engine = "marian"
  - source_language = "pt" (or "en"/"es")
  - target_language = "zh-cn"
- [engines.marian]
  - model_path = "./models"

Restart the app after config changes.

## Configuration

File: config.toml (created automatically on first run)

Common fields:
- [general]
  - active = true/false
  - source_language = "pt" | "en" | "es"
  - target_language = "zh-cn"
  - active_engine = "windows_lp" | "marian" | "deepl" | "google"
  - hotkey_modifier = "shift" | "ctrl+shift" | "alt"
  - hotkey_key = "enter"
- [engines.deepl]
  - api_key = ""
- [engines.google]
  - api_key = ""
- [engines.marian]
  - model_path = "./models"
- [ui]
  - show_notifications = true

## Usage

1) Open any text field (Notepad, browser input box, Discord, etc.)
2) Type text
3) Press Shift+Enter
4) The selected text is replaced by the translated text.

Notes:
- The app uses clipboard + Ctrl+A/C/V simulation, so some apps may behave differently.
- For safety, if focus changes during a long translation, paste is aborted to avoid replacing content in the wrong window.

## Troubleshooting

- Nothing happens on hotkey:
  - Ensure app is Active (tray tooltip/menu)
  - Another app may be using the same hotkey; change hotkey_modifier in config.toml and restart
- Marian errors:
  - Confirm scripts\marian_translate.py exists
  - Confirm models folder contains opus-mt-romance-en and opus-mt-en-zh
  - Confirm python dependencies installed:
    - python -c "import torch, transformers, sentencepiece, huggingface_hub; print('OK')"
- DeepL/Google:
  - Ensure API key is set in config.toml and engine is selected

## Privacy

- Offline Marian: translations happen locally on your machine after models are downloaded.
- API engines: text is sent to the selected provider (DeepL/Google) over the network.

## Development

Build (Windows):
- cargo run
- cargo build --release

See:
- ./design.md
- ./summary.txt

## License

See ./LICENSE
