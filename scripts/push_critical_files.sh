#!/usr/bin/env bash
set -euo pipefail

WSL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WIN_ROOT="/mnt/c/dev/tongyi-translator"

echo "WSL_ROOT = ${WSL_ROOT}"
echo "WIN_ROOT = ${WIN_ROOT}"

mkdir -p "${WIN_ROOT}/src/ui"
mkdir -p "${WIN_ROOT}/scripts"

cp -v "${WSL_ROOT}/Cargo.toml" "${WIN_ROOT}/Cargo.toml"
cp -v "${WSL_ROOT}/build.rs"   "${WIN_ROOT}/build.rs"   2>/dev/null || true

cp -v "${WSL_ROOT}/src/main.rs"       "${WIN_ROOT}/src/main.rs"
cp -v "${WSL_ROOT}/src/config.rs"     "${WIN_ROOT}/src/config.rs"
cp -v "${WSL_ROOT}/src/clipboard.rs"  "${WIN_ROOT}/src/clipboard.rs"
cp -v "${WSL_ROOT}/src/input.rs"      "${WIN_ROOT}/src/input.rs"
cp -v "${WSL_ROOT}/src/translator.rs" "${WIN_ROOT}/src/translator.rs"
cp -rv "${WSL_ROOT}/src/engine"       "${WIN_ROOT}/src/" 2>/dev/null || true

cp -v "${WSL_ROOT}/src/ui/tray.rs"         "${WIN_ROOT}/src/ui/tray.rs"
cp -v "${WSL_ROOT}/src/ui/notification.rs" "${WIN_ROOT}/src/ui/notification.rs" 2>/dev/null || true

# scripts do Marian
cp -v "${WSL_ROOT}/scripts/marian_translate.py"       "${WIN_ROOT}/scripts/marian_translate.py"
cp -v "${WSL_ROOT}/scripts/marian_download_models.py" "${WIN_ROOT}/scripts/marian_download_models.py"

echo "Push done."
