# Estado activo

Este archivo es **el único** de estado mutable — se reescribe a medida que cambia la
realidad del proyecto. A diferencia de `decisions/`, no tiene historial: refleja el
presente. Si vuelves a este proyecto después de un tiempo, este es el primer archivo
a leer después de `AGENTS.md`.

_Última actualización: 2026-09-05._

## En curso / sin commitear

Hay una sesión de limpieza y reorganización de repo completa, **sin commitear
todavía** (el usuario no lo ha pedido explícitamente). `git status --short` en la raíz
muestra el detalle exacto; resumen de lo hecho:

- Eliminados archivos muertos de raíz: `.oxfmtrc.json`, `PROMT.md`,
  `REFACTOR_PROJECT_GLOBAL_SCOPE.md` (su contenido vive ahora en
  [`decisions/0001-single-active-install-scope.md`](decisions/0001-single-active-install-scope.md)),
  `frontend/README.md` (boilerplate de Vite sin editar), `frontend/bun.lock`
  (lockfile de un package manager no usado).
- `configs/skills.yaml` movido a `src-tauri/configs/skills.yaml` (era el único
  archivo de esa carpeta y solo lo consume el backend Rust vía `include_str!`).
- Todo lo generado por builds/tests (antes disperso en `test-skills/`, `test/`,
  `dist/`, `frontend/dist/`) consolidado bajo `.generated/` (gitignored en una sola
  línea). Ver `taskfile.yml` (`GENERATED_DIR`/`DIST_DIR`/`TEST_DIR`),
  `frontend/vite.config.ts` (`outDir`+`emptyOutDir`), `tauri.conf.json`
  (`frontendDist`), `.tasks/scripts/smoke.sh`, `src-tauri/tests/skill_lifecycle.rs`.
- `task docker:release-install` ahora ofrece crear `.env` desde una plantilla si no
  existe, y espera confirmación antes de instalar (`.tasks/scripts/install-local.sh`).
- Este mismo sistema de memoria (`memory/`) se creó en esta sesión, con 3 ADRs
  retroactivos: [0001](decisions/0001-single-active-install-scope.md),
  [0002](decisions/0002-no-global-state-library.md) y
  [0003](decisions/0003-generated-artifacts-consolidated.md) (esta última documenta
  la consolidación de `.generated/` de arriba).
- El patrón `memory/` se empaquetó como skill reutilizable (`memory-adr`) en
  `/home/uu/Skills/memory-adr/`, fuera de este repo — no requiere acción aquí.

**Si retomas este trabajo**: corre `git status --short` primero — si ya no coincide
con la lista de arriba, alguien commiteó o siguió trabajando; confía en git, no en
este archivo, y corrige esta sección.

## Verificado en esta sesión

`cargo clippy --all-targets` sin warnings, 201 tests Rust (`cargo test`), `eslint` y
`vue-tsc` sin hallazgos, 233 tests frontend (`vitest run`), `vite build` con el nuevo
`outDir`, y `task smoke` corrido de punta a punta contra el binario real.

## Pendiente / ideas abiertas

- Ninguna abierta por ahora. Nada bloqueando: el pipeline completo está verde.
