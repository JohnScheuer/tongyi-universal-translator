#!/usr/bin/env bash
set -euo pipefail

SRC="${HOME}/dev/tongyi-translator"
DST="/mnt/c/dev/tongyi-translator"

mkdir -p "$DST"
mkdir -p "$DST/src/ui"

echo "=== Sync WSL -> Windows ==="
echo "SRC: $SRC"
echo "DST: $DST"

# Usar -rltD em vez de -a para evitar ruídos de owner/group/perms em NTFS
# Usar --checksum para forçar atualização mesmo se mtime/metadata estiver estranho
rsync -rltD --checksum --delete \
  --exclude 'target/' \
  --exclude '.git/' \
  "$SRC"/ "$DST"/

echo ""
echo "=== Verificacao pos-sync ==="
grep -n "DEBUG TongYi v2 fix" "$DST/src/main.rs" || true
grep -n "extract_mouse_msg" "$DST/src/main.rs" || true
grep -n "extract_mouse_msg" "$DST/src/ui/tray.rs" || true

echo ""
echo "OK: synced to $DST"
