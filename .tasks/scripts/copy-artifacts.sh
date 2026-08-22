#!/usr/bin/env bash
set -Eeuo pipefail

# Args: $1 = TAURI_DIR, $2 = DIST_DIR, $3 = VERSION (all relative/values from the project root).
tauri_dir="$1"
dist_dir="$2"
version="$3"

appimage_src=$(find "$tauri_dir/target/release/bundle/appimage" -maxdepth 1 -name '*.AppImage' | head -n1)
if [ -z "$appimage_src" ]; then
  echo "No AppImage found under $tauri_dir/target/release/bundle/appimage" >&2
  exit 1
fi
cp "$appimage_src" "$dist_dir/Skills-Installer-$version-x86_64.AppImage"
echo "Copied $appimage_src -> $dist_dir/Skills-Installer-$version-x86_64.AppImage"
cp "$appimage_src" "$dist_dir/skills-installer.AppImage"
echo "Copied $appimage_src -> $dist_dir/skills-installer.AppImage"

deb_src=$(find "$tauri_dir/target/release/bundle/deb" -maxdepth 1 -name '*.deb' | head -n1)
if [ -z "$deb_src" ]; then
  echo "No .deb found under $tauri_dir/target/release/bundle/deb" >&2
  exit 1
fi
cp "$deb_src" "$dist_dir/skills-installer_${version}_amd64.deb"
echo "Copied $deb_src -> $dist_dir/skills-installer_${version}_amd64.deb"
