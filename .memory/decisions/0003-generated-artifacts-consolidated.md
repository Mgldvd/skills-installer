---
type: Decision
title: "Todo lo generado y no versionado vive bajo `.generated/`, excepto `node_modules/` y `target/`"
description: Los artefactos generados por builds/tests se consolidan bajo una sola carpeta .generated/ en la raíz, en vez de estar dispersos con nombres inconsistentes.
status: stable
decision_status: accepted
date: 2026-09-05
---

# 0003. Todo lo generado y no versionado vive bajo `.generated/`, excepto `node_modules/` y `target/`

## Contexto

Antes de esta decisión, los artefactos generados por builds/tests estaban dispersos
por la raíz del repo con nombres inconsistentes: `test-skills/` (fixture del test de
integración `skill_lifecycle.rs`), `test/` (scratch de `task smoke`), `dist/`
(instaladores empaquetados AppImage/.deb) y `frontend/dist/` (build del webview vía
Vite). Todos gitignored individualmente, pero visibles como ruido suelto en la raíz
del repo (`ls .` mostraba 5 carpetas de código+generado mezcladas).

## Decisión

Consolidar todo lo anterior bajo una sola carpeta `.generated/` en la raíz (gitignored
con una sola línea, en vez de 4 entradas separadas):

- `.generated/test-skills/` (antes `test-skills/`)
- `.generated/smoke/` (antes `test/`)
- `.generated/dist/` (antes `dist/`)
- `.generated/frontend/` (antes `frontend/dist/`)

**Explícitamente excluidos de la consolidación**, y dejados donde ya estaban:

- `frontend/node_modules/`: Node exige que viva exactamente junto a `package.json`;
  no existe una forma soportada de relocalizarlo sin symlinks que rompen el
  resolutor de módulos.
- `src-tauri/target/`: caché de compilación de Cargo. Ya vivía ordenado dentro de
  `src-tauri/`, es el artefacto más grande y con más escritura de todo el repo, y
  relocalizarlo (vía `CARGO_TARGET_DIR`) no ganaba limpieza real mientras sí
  arriesgaba romper supuestos de rust-analyzer/IDEs sobre dónde vive `target/`.
- `src-tauri/gen/`: solo se genera para targets mobile de Tauri; este proyecto es
  Linux desktop exclusivamente (ver `AGENTS.md`), así que ni siquiera existe en
  disco — no había nada que mover.

## Consecuencias

- Un solo punto de gitignore (`/.generated/`) reemplaza 4 entradas dispersas.
- `task clean` pasa de listar 3 rutas a un `rm -rf {{.TAURI_DIR}}/target
  {{.GENERATED_DIR}}` — borra todo lo generado de una sola vez.
- Mover `frontend/dist/` fuera de la raíz del proyecto Vite (`frontend/`) hizo que
  Vite dejara de vaciar el directorio automáticamente antes de cada build (advertencia
  real observada: "outDir is not inside project root and will not be emptied").
  Corregido agregando `emptyOutDir: true` explícito en `vite.config.ts` — sin esto,
  builds sucesivos habrían acumulado artefactos viejos silenciosamente.
- `.tasks/scripts/smoke.sh` resolvía el binario del CLI con una ruta relativa
  (`../"$cli_bin"`) que asumía que `test_dir` estaba exactamente un nivel bajo la
  raíz del repo. Al anidar un nivel más (`.generated/smoke/`), esa ruta se habría
  roto; se corrigió resolviendo rutas absolutas antes de hacer `cd`, lo cual de paso
  deja el script correcto sin importar la profundidad de anidamiento futura.
- `src-tauri/tests/skill_lifecycle.rs` (`fixture_root()`) y `taskfile.yml`
  (`DIST_DIR`, `TEST_DIR`, nueva `GENERATED_DIR`) actualizados para apuntar a las
  nuevas rutas. Verificado de punta a punta: `cargo test`, `vite build` (confirmando
  que `emptyOutDir` limpia artefactos viejos), y `task smoke` corrido contra el
  binario real.
