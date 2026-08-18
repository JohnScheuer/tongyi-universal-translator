# TongYi Translator (通译)

[![Platform](https://img.shields.io/badge/platform-Windows-0078D6)](https://www.microsoft.com/windows)
[![Language](https://img.shields.io/badge/language-Rust-black)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/badge/release-v0.1-informational)](#)
[![License](https://img.shields.io/badge/license-See%20LICENSE-green)](./LICENSE)

Universal Windows tray translator: press a global hotkey to translate the currently focused input field and replace it in-place.

Target language: Simplified Chinese (zh-cn)

Author: João Felipe De Souza  
GitHub: https://github.com/JohnScheuer

## Quick links
- Design notes: ./design.md
- Current state summary: ./summary.txt
- License: ./LICENSE

## Features (V0.1)
- Runs as a lightweight Windows tray application
- Global hotkey translation (default: Shift+Enter)
- In-place replacement in the currently focused input field
- Full clipboard backup/restore using OLE IDataObject
- Active/Inactive toggle from tray
- Engine and source-language selection from tray menu

### Runtime behavior
- Starts silently and stays in the system tray
- Uses a global hotkey to trigger translation
- Does not require Administrator privileges (do not run as Admin)

### Tray behavior
- Left-click toggles Active/Inactive
- Right-click opens a menu to select:
  - Engine
  - Source language
  - Exit

### Translation flow
1. Select all (Ctrl+A)
2. Copy (Ctrl+C)
3. Translate with selected engine
4. Paste translated text back (Ctrl+V)
5. Restore original clipboard contents

## Translation engines (V0.1)

### Available
- MarianMT offline (Python worker)  
  Local translation after one-time model download.  
  Depends on a local Python runtime and a persistent worker script.  
  Uses a pivot pipeline:
  - PT/ES -> EN via opus-mt-ROMANCE-en
  - EN -> ZH via opus-mt-en-zh
  - Phrasebook-first for short greetings (deterministic results)

### Available with API key
- DeepL API
- Google Translate API

### Not yet implemented
- Windows Language Pack engine

## Install (V0.1)

### Option A — Run prebuilt release (recommended)
1) Download the latest release zip
2) Extract to a folder, for example:
   C:\Apps\tongyi-translator\
3) Run tongyi-translator.exe (do not run as Administrator)

### Option B — Build from source (Windows)
Requirements:
- Rust toolchain (stable)
- Windows MSVC toolchain

Commands:
- cargo run
- cargo build --release

### Option C — Enable MarianMT offline
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

Configure config.toml:
- Set engine to marian
- Set model_path to ./models

Restart the app after config changes.

## Configuration

File: config.toml (created automatically on first run)

Example:

    [general]
    active = true
    active_engine = "marian"
    source_language = "pt"
    # valid values: "pt", "en", "es"
    target_language = "zh-cn"
    hotkey_modifier = "shift"
    # valid values: "shift", "ctrl+shift", "alt"
    hotkey_key = "enter"

    [engines.deepl]
    api_key = ""

    [engines.google]
    api_key = ""

    [engines.marian]
    model_path = "./models"

    [ui]
    show_notifications = true

## Usage
1) Focus any input field (Notepad, browser input box, Discord, etc.)
2) Type text
3) Press Shift+Enter
4) The selected text is replaced by the translated text

## Current limitations
- Windows only
- Translation is driven by simulated Ctrl+A/C/V, so behavior depends on the target application
- Some applications may block or modify clipboard interactions
- First Marian translation can be slow due to model load
- Windows Language Pack engine is not implemented in V0.1

## Troubleshooting
- Nothing happens on hotkey:
  - Ensure app is Active (tray tooltip/menu)
  - Another app may be using the same hotkey; change it in config.toml and restart
- Marian offline:
  - Confirm scripts/marian_translate.py exists in the app folder
  - Confirm models folder contains:
    - models/opus-mt-romance-en/
    - models/opus-mt-en-zh/
  - Confirm python deps:
    - python -c "import torch, transformers, sentencepiece, huggingface_hub; print('OK')"
- DeepL/Google:
  - Ensure API key is set in config.toml and engine is selected in tray

## Privacy
- Marian offline: translations are performed locally after model download.
- API engines: text is sent to the selected provider (DeepL/Google) over the network.

## License
See ./LICENSE
