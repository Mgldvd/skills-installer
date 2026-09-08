#!/usr/bin/env bash
set -Eeuo pipefail

# This container runs as root by default (see Dockerfile — HOST_UID/GID
# are only known at `docker compose run` time, not at image-build time).
# A root process writing into `/workspace`, the bind-mounted repo, leaves
# every file it touches (src-tauri/target, .generated, ...) owned by root on the
# host — which then blocks a normal host-side `npm install`/`cargo build`
# with EACCES. Fix: remap the base image's existing non-root user
# ("master") to whatever uid/gid actually owns the repo on the host
# (exported by the `task docker:release` Task that starts this container),
# then hand off the rest of this script to that
# user via `gosu` before touching anything under /workspace. If
# HOST_UID/HOST_GID were never set (e.g. this script run some other way),
# this falls back to root:root — unchanged from before this fix, not worse.
if [[ "$(id -u)" -eq 0 ]]; then
  target_uid="${HOST_UID:-0}"
  target_gid="${HOST_GID:-0}"
  if [[ "$target_uid" != "$(id -u master)" || "$target_gid" != "$(id -g master)" ]]; then
    groupmod -o -g "$target_gid" master
    usermod -o -u "$target_uid" -g "$target_gid" master
    chown -R master:master /home/master /opt/cargo /opt/rustup
  fi
  # Docker mounts a brand-new named volume (compose.yml: frontend_node_modules,
  # npm_cache, cargo_registry, cargo_git, rust_target) owned by root, no
  # matter which user the container itself runs as — unrelated to the
  # uid/gid check above, so this always runs, every container start.
  # Already-populated volumes from a prior run are already master-owned;
  # chown -R on those is just a fast no-op walk, not a real rewrite.
  chown -R master:master \
    /workspace/frontend/node_modules \
    /opt/npm-cache \
    /opt/cargo/registry \
    /opt/cargo/git \
    /workspace/src-tauri/target
  exec gosu master bash "$0" "$@"
fi

readonly LOCK_FILE="frontend/package-lock.json"
readonly MODULES_DIR="frontend/node_modules"
readonly LOCK_STAMP="$MODULES_DIR/.skills-control-deck-lock-hash"
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
task release
