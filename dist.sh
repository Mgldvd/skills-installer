#!/usr/bin/env bash
set -Eeuo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if ! command -v docker >/dev/null 2>&1; then
  echo "Error: Docker no está instalado o no está disponible en PATH." >&2
  exit 1
fi

if ! docker compose version >/dev/null 2>&1; then
  echo "Error: se requiere Docker Compose v2 (comando: docker compose)." >&2
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "Error: el daemon de Docker no está activo o el usuario no tiene acceso." >&2
  exit 1
fi

export HOST_UID="$(id -u)"
export HOST_GID="$(id -g)"

docker compose build dist
docker compose run --rm --no-deps dist

echo
echo "Artifacts generados en: $SCRIPT_DIR/dist"
ls -lah "$SCRIPT_DIR/dist"
