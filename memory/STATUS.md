# Estado activo

Este archivo es **el único** de estado mutable — se reescribe a medida que cambia la
realidad del proyecto. A diferencia de `decisions/`, no tiene historial: refleja el
presente. Si vuelves a este proyecto después de un tiempo, este es el primer archivo
a leer después de `AGENTS.md`.

_Última actualización: 2026-09-05._

## En curso / sin commitear

Nada en curso ni sin commitear. La sesión de limpieza y reorganización de repo se
commiteó completa en `c6a58b5` ("chore: clean up repo layout and add ADR-based
project memory") y ya está pusheada a `origin/master`. `git status` limpio. Resumen
de lo que entró en ese commit:

- Eliminados archivos muertos de raíz: `.oxfmtrc.json`, `PROMT.md`,
  `REFACTOR_PROJECT_GLOBAL_SCOPE.md` (su contenido vive ahora en
  [`decisions/0001-single-active-install-scope.md`](decisions/0001-single-active-install-scope.md)),
  `frontend/README.md` (boilerplate de Vite sin editar), `frontend/bun.lock`
  (lockfile de un package manager no usado).
- `configs/skills.yaml` movido a `src-tauri/configs/skills.yaml`.
- Todo lo generado por builds/tests consolidado bajo `.generated/` (ver
  [0003](decisions/0003-generated-artifacts-consolidated.md)).
- `task docker:release-install` ahora ofrece crear `.env` desde una plantilla si no
  existe, y espera confirmación antes de instalar (`.tasks/scripts/install-local.sh`).
- Este mismo sistema de memoria (`memory/`) se creó en esta sesión, con 3 ADRs
  retroactivos: [0001](decisions/0001-single-active-install-scope.md),
  [0002](decisions/0002-no-global-state-library.md) y
  [0003](decisions/0003-generated-artifacts-consolidated.md).
- El patrón `memory/` se empaquetó como skill reutilizable (`memory-adr`) en
  `/home/uu/Skills/memory-adr/`, fuera de este repo — no requiere acción aquí.

**Si retomas este trabajo**: corre `git status --short` primero — si ya no coincide
con "limpio", alguien siguió trabajando; confía en git, no en este archivo, y corrige
esta sección.

## Verificado antes de commitear

`cargo clippy --all-targets` sin warnings, 201 tests Rust (`cargo test`), `eslint` y
`vue-tsc` sin hallazgos, 233 tests frontend (`vitest run`), `vite build` con el nuevo
`outDir`, y `task smoke` corrido de punta a punta contra el binario real. No se
volvió a correr el pipeline después del commit — si retomas trabajo, correlo de
nuevo antes de asumir que sigue verde.

## Pendiente / ideas abiertas

- Ninguna abierta por ahora. Nada bloqueando: el pipeline completo está verde.
