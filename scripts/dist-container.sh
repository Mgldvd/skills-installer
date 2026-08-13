#!/usr/bin/env bash
set -Eeuo pipefail

readonly LOCK_FILE="frontend/package-lock.json"
readonly MODULES_DIR="frontend/node_modules"
readonly LOCK_STAMP="$MODULES_DIR/.skills-installer-lock-hash"
readonly LOCK_HASH="$(sha256sum "$LOCK_FILE" | cut -d' ' -f1)"

installed_hash=""
if [[ -f "$LOCK_STAMP" ]]; then
  installed_hash="$(<"$LOCK_STAMP")"
fi

if [[ "$installed_hash" != "$LOCK_HASH" || ! -x "$MODULES_DIR/.bin/tauri" ]]; then
  echo "Installing frontend dependencies..."
  npm ci \
    --prefix frontend \
    --cache /opt/npm-cache \
    --prefer-offline \
    --no-audit \
    --no-fund
  printf '%s\n' "$LOCK_HASH" > "$LOCK_STAMP"
else
  echo "Frontend dependencies match package-lock.json; reusing the Docker volume."
fi

rm -rf dist
make release
chown -R "${HOST_UID:-0}:${HOST_GID:-0}" dist
