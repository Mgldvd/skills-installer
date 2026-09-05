#!/usr/bin/env bash
set -Eeuo pipefail

# Args: $1 = DIST_DIR (relative to the project root).
#
# Copies the built AppImage to LOCAL_INSTALL (LOCAL_APP/LOCAL_INSTALL, read
# from .env at the project root — see taskfile.yml's `dotenv:`). If .env
# doesn't exist yet, offers to create it from a template and pauses so the
# user has a chance to fill in LOCAL_INSTALL before the copy runs.
dist_dir="$1"
env_file=".env"

if [ ! -f "$env_file" ]; then
  echo
  echo "No se encontró $env_file en la raíz del proyecto."
  echo "Sugerencia para instalar el AppImage automáticamente en cada build:"
  echo
  echo "  LOCAL_APP=\"skills-installer.AppImage\""
  echo "  LOCAL_INSTALL="
  echo
  read -r -p "¿Crear $env_file con esta plantilla ahora? [s/N] " reply
  if [[ "$reply" =~ ^[sSyY] ]]; then
    printf 'LOCAL_APP="skills-installer.AppImage"\nLOCAL_INSTALL=\n' >"$env_file"
    echo
    echo "Creado $env_file. Completa LOCAL_INSTALL con la carpeta destino y presiona Enter para continuar…"
    read -r
    # shellcheck disable=SC1090 # $env_file is always ".env" at the project root.
    source "$env_file"
  else
    echo "Build generado en $dist_dir/, sin instalar (no se creó $env_file)."
    exit 0
  fi
fi

if [ -n "${LOCAL_APP:-}" ] && [ -n "${LOCAL_INSTALL:-}" ]; then
  mkdir -p "$LOCAL_INSTALL"
  dest="${LOCAL_INSTALL%/}/$LOCAL_APP"
  cp "$dist_dir/$LOCAL_APP" "$dest"
  echo "Instalado: $dest"
else
  echo "LOCAL_APP/LOCAL_INSTALL no configurados en $env_file — build generado en $dist_dir/, sin instalar."
fi
