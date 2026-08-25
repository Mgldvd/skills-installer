# Refactor: modo único Project/Global

Documento de trabajo para el refactor arquitectónico acordado el 2026-08-25: reemplazar el
scope por-agente (`AgentScopeSelection{project, global}`, no exclusivo) por un **modo único
activo para toda la app** (`Project` | `Global`), controlado desde el header. En modo Project,
la carpeta seleccionada define destino/discovery (look actual). En modo Global no hace falta
carpeta y el header cambia de aspecto para que sea imposible confundir en qué modo se está.

Este archivo es el checklist vivo del refactor. Si una sesión se corta (cuota, error, lo que
sea), la siguiente sesión debe leer este archivo completo antes de tocar código.

---

## Prompt de reanudación

Si esta conversación se interrumpió y necesitas retomar el refactor en una sesión nueva de
Claude Code, copia y pega esto como primer mensaje:

```
Retomamos el refactor de scope Project/Global documentado en
REFACTOR_PROJECT_GLOBAL_SCOPE.md en la raíz del repo. Antes de escribir código:

1. Lee REFACTOR_PROJECT_GLOBAL_SCOPE.md completo (contexto, decisiones ya tomadas,
   trade-offs aceptados, y el checklist con su estado real).
2. Corre `git log --oneline -20` y busca commits con prefijo "refactor(scope):" para ver
   qué fases quedaron realmente commiteadas — el checklist de este archivo puede estar
   desactualizado si una sesión anterior no lo actualizó antes de cortarse. Si hay
   discrepancia entre lo marcado [x] y lo que el código realmente tiene, confía en el
   código/git log, no en el checkbox, y corrige el archivo antes de seguir.
3. Corre `git status --short` y `git diff --stat` — si hay cambios sin commitear de una
   sesión anterior, revísalos antes de continuar (no los descartes a menos que estén
   claramente rotos o a medio terminar de forma inconsistente).
4. Corre el pipeline completo (`task typecheck`, `task lint`, `task test` desde la raíz)
   para confirmar que el estado actual está verde antes de seguir agregando cambios.
5. Continúa desde el primer checkbox sin marcar, en orden de fase (no saltees fases).
6. Al completar cada checkbox, márcalo [x] en este mismo archivo y comitéalo junto con el
   código de esa fase (commits pequeños, uno por checkbox o por grupo chico de checkboxes
   relacionados, con mensaje "refactor(scope): fase N - <qué se hizo>").
7. Si te quedas sin cuota o tienes que cortar, deja el archivo con el estado real reflejado
   (checkboxes al día) y el repo en un commit limpio (no a medio editar) antes de terminar,
   aunque eso signifique parar en medio de una fase en vez de completarla.
```

---

## Contexto arquitectónico (para no tener que re-investigar)

- Hay **dos conceptos distintos llamados "Project"** en el código actual:
  - `domain::Project` / `ProjectsService` / `ProjectsDialog.vue` / `useProjects.ts`: un preset
    con nombre de skills seleccionados, **sin ruta**, persistido en
    `$XDG_CONFIG_HOME/skills-installer/projects.json`.
  - `state.projectRoot` (`frontend/src/composables/useAppState.ts`): la carpeta activa real,
    **efímera** (se resetea a `""` en cada arranque, sin persistencia), fijada solo desde
    `AppHeader.vue`'s folder picker vía `App.vue:handleProjectPathUpdate`.
- El catálogo (`skills.yaml`) es **único por máquina** (resuelto en
  `src-tauri/src/config/locator.rs`), nunca duplicado por proyecto. Lo único que varía por
  carpeta es el estado `installed`/`installed_agents` de cada Skill.
- El scope hoy vive en `src-tauri/src/domain/install.rs`:
  `AgentScopeSelection { project: bool, global: bool }` — **no exclusivo, por agente**,
  guardado en `UiPreferences.agent_scopes: HashMap<agent_id, AgentScopeSelection>`. El enum
  `InstallScope::{Project, Global}` ya existe en el mismo archivo pero **no se usa como valor
  único** — es justo lo que este refactor promueve a canónico.
- Discovery: `src-tauri/src/skills/discovery.rs` tiene tablas separadas
  (`AGENT_PROJECT_DIRS` bajo `project_root`, `AGENT_GLOBAL_DIRS` bajo `$HOME`) que hoy se
  **mezclan siempre** en `SkillsService::load_state_for` (`src-tauri/src/app/skills_service.rs`).
- Install (`src-tauri/src/installer/skills_cli.rs`, `add_args_for_skill`): hoy arma **hasta 2
  grupos** de invocación CLI (agentes-project, agentes-global) porque `--global` aplica a toda
  la invocación, no por agente.
- Uninstall (`remove()` en el mismo archivo): **nunca pasa `--global`** — es efectivamente
  siempre project-scoped en la práctica, aunque `AgentScopeSelection` diga otra cosa. Este es
  el bug ya detectado que el refactor arregla gratis (Fase 7).
- `AppHeader.vue` hoy es minimalista: logo + `<h1>` + un botón de carpeta. No tiene selector
  de modo ni tratamiento visual distinto.

## Decisiones ya tomadas (confirmadas 2026-08-25)

- ✅ Un modo activo a la vez para toda la app. **Se acepta perder** la capacidad actual de
  instalar el mismo agente a project+global simultáneamente en una sola acción (construida
  esta misma sesión a pedido explícito del usuario, antes de este refactor). Para tener un
  agente en ambos lados: cambiar de modo e instalar dos veces.
- ✅ Renombrar el concepto actual de "Project" (preset sin ruta) → **`Preset`**, liberando
  "Project" para que signifique solo la carpeta/modo activo. Ver Fase 8.

## Decisiones ya tomadas (Fase 0, 2026-08-25)

- ✅ cwd de fallback en modo Global puro: **`$HOME`**. Semánticamente correcto (los dirs
  globales viven bajo `$HOME`) y siempre existe, a diferencia de "última carpeta usada" que
  puede no existir más.
- ✅ Persistencia: **sí**, se agrega `last_scope: InstallScope` y `last_project_path:
  Option<String>` a `UiPreferences`, restaurados al abrir la app (reemplaza el reseteo a
  vacío actual de `state.projectRoot`).
- ✅ Look de modo Global en el header: acento ámbar/naranja (`--danger`-adjacent pero no rojo,
  para no leerse como error) en el borde inferior del header + badge de texto "GLOBAL" junto
  al `<h1>`, independiente del accent color configurable del usuario — así se distingue
  siempre, sea cual sea la paleta elegida.

---

## Checklist

### Fase 0 — Decisiones de diseño

- [x] Resolver cwd de fallback en modo Global puro (ver arriba: `$HOME`).
- [x] Resolver persistencia de último modo/carpeta (ver arriba: sí, en `UiPreferences`).
- [x] Mockup/decisión concreta del look distinto del header en modo Global (ver arriba:
  acento ámbar + badge "GLOBAL").

### Fase 1 — Backend: modelo de dominio ✅ (completada junto con Fase 3 — ver nota)

- [x] `InstallOptions.agent_scopes: HashMap<...>` → `InstallOptions.scope: InstallScope`.
- [x] Eliminar `AgentScopeSelection` por completo (no solo "de uso activo": no hacía falta
  mantenerla ni para migración — serde ya ignora una clave `agentScopes` desconocida en un
  `preferences.json` viejo sin `deny_unknown_fields`, así que no hay nada que parsear).
- [x] `UiPreferences`: quitar `agent_scopes`; agregar `last_scope: InstallScope` (con
  `#[derive(Default)]`/`#[default]` en `InstallScope`, no un `impl Default` manual — así lo
  pidió Clippy) y `last_project_path: Option<String>`.
- [x] `UninstallRequest` (no estaba en el plan original, pero era necesario): se le agregó
  `scope: InstallScope` — sin esto no había forma de que el frontend le dijera al backend qué
  scope desinstalar.
- [x] Migración: un `preferences.json` viejo con `agentScopes` carga sin error y cae en
  `last_scope: Project` — test reescrito
  (`load_ignores_a_legacy_agent_scopes_map_and_defaults_last_scope_to_project`).
- [x] Tests Rust de dominio actualizados/reescritos.

**Nota de ejecución**: Fase 1 y Fase 3 no se pudieron separar en commits independientes — el
compilador de Rust exige que todo sitio que use `AgentScopeSelection`/`scopes_for`/
`any_global` se actualice a la vez para que el crate compile, y eso es exactamente el trabajo
de Fase 3 (`add_args_for_skill`, `remove()`, `update()`). Quedaron en un solo commit.

### Fase 2 — Backend: discovery ✅

- [x] `SkillsService::load_state_for(project_root, scope)` — nuevo segundo parámetro
  obligatorio. `Project` escanea solo `project_root`; `Global` escanea solo `$HOME`
  (`home_override` en tests). Ya no mezcla ambos nunca.
- [x] `SkillsService` gana un campo `scope: InstallScope` (default `Project`) + builder
  `with_scope`, espejando el patrón ya existente de `project_root`/`home_override`.
  `load_state()` (sin args) usa `self.scope`; nuevo `load_state_with_scope(scope)` para
  cuando el caller tiene scope explícito pero no un `project_root` distinto.
- [x] `Skill.installed`/`installed_agents` ahora son relativos al scope activo — antes era
  la unión de ambos scopes, ahora es exactamente uno.
- [x] Único call site de producción que necesitó tocarse: `commands::installation::refresh`
  (el resto de los comandos usan `load_state()` sin args, sin cambios). Ganó un parámetro
  IPC nuevo `scope: Option<InstallScope>`, default `Project` si el frontend viejo no lo manda
  — mismo patrón "checkpoint seguro" que Fase 1+3.
- [x] Tests: reescrito `load_state_also_reports_agents_the_skill_is_installed_for_globally`
  (asumía el merge viejo) →
  `load_state_for_never_mixes_project_and_global_scope_discovery`, que prueba explícitamente
  que Project no ve lo instalado en Global y viceversa. 5 call sites en
  `tests/skill_lifecycle.rs` actualizados con el nuevo parámetro (todos Project, sin cambio
  de comportamiento).

### Fase 3 — Backend: install / uninstall / update ✅ (ver nota en Fase 1)

- [x] `add_args_for_skill`: de "hasta 2 grupos" → siempre 1 grupo con el scope activo.
  Retorna `Vec<String>` en vez de `Vec<Vec<String>>`.
- [x] `remove()`: pasa `--global` cuando `request.options.scope == InstallScope::Global`
  (cierra el bug ya detectado antes de este refactor).
- [x] `update`: usa `request.options.scope` directo en vez de `any_global()`.
- [x] cwd de fallback en Global puro implementado en `resolve_cwd`: ignora `project_path` por
  completo y usa `$HOME` (con `self.project_root` como último recurso si `$HOME` no está
  seteado) — ver decisión de Fase 0.
- [x] Tests de `skills_cli.rs` reescritos: se borraron los 2 tests de "grupos duales"
  (`_splits_into_two_groups_for_mixed_scope`, `_puts_one_agent_in_both_groups...`, ya no
  aplican), se agregaron `remove_passes_the_global_flag_when_the_active_scope_is_global` y
  `global_scope_ignores_project_path_and_uses_home_as_cwd`.

`task typecheck`/`task lint`/`task test` verdes desde la raíz tras Fase 1+3 (184 tests Rust +
9 de integración + 219 frontend, sin tocar frontend todavía — la app real seguirá operando en
Project scope hasta que el frontend empiece a mandar `scope`/`UninstallRequest.scope`, ya que
ambos default a `Project` vía `#[serde(default)]`. Checkpoint seguro: nada se rompe estando a
medio camino.

### Fase 4 — Frontend: estado

- [ ] `state.projectRoot: string` + scope implícito por-agente → `state.scope: "project" |
  "global"` + `state.projectPath: string` como única fuente de verdad.
- [ ] Revisar cada lectura actual de `state.projectRoot` contra el nuevo modelo.

### Fase 5 — Frontend: AppHeader como selector de modo

- [ ] Control tipo toggle/segmented "Project" / "Global" en el header.
- [ ] Modo Project: look actual (botón de carpeta).
- [ ] Modo Global: tratamiento visual distinto decidido en Fase 0.
- [ ] Cambiar de modo dispara refresh de skills/discovery.
- [ ] Tests de `AppHeader.spec.ts` (crear si no existe, o actualizar).

### Fase 6 — Frontend: simplificar AgentsDialog

- [ ] Quitar los botones Project/Global por agente y el toggle masivo.
- [ ] AgentsDialog queda solo para habilitar agentes + reordenar.
- [ ] Quitar plumbing muerto de `agentScopes` en `App.vue`/`AgentsDialog.vue`.
- [ ] `AgentsDialog.spec.ts` actualizado (se van los tests de toggle de scope).

### Fase 7 — Bulk uninstall (verificación, no trabajo nuevo si Fases 2+3 están bien)

- [ ] Confirmar que bulk-uninstall ya respeta el scope activo automáticamente.
- [ ] Cerrar el gap conocido de "bulk uninstall no cubre Global" documentado antes de este
  refactor.

### Fase 8 — Rename Project → Preset

- [ ] `domain::Project` → `domain::Preset`.
- [ ] `ProjectsService` → `PresetsService`; `projects.json` → `presets.json` (con migración
  de nombre de archivo si ya existe uno viejo).
- [ ] `ProjectsDialog.vue` → `PresetsDialog.vue`; `useProjects.ts` → `usePresets.ts`.
- [ ] Textos de UI en español/inglés actualizados donde digan "Project" refiriéndose al preset.
- [ ] `PROMT.md` línea 13 ("alcance Project o Global") revisada/actualizada si el lenguaje
  cambió con este refactor.

### Fase 9 — Tests y validación final

- [ ] Test de migración explícito: cargar un `preferences.json` viejo con `agent_scopes` y
  confirmar que no rompe y cae en un `last_scope` sano.
- [ ] `task typecheck`, `task lint`, `task test` en verde desde la raíz.
- [ ] Verificación manual en la app real (no solo tests) de: cambiar de modo en el header,
  instalar en cada modo, desinstalar en cada modo, bulk-uninstall en modo Global.

---

## Trade-offs aceptados (no reabrir sin confirmación explícita del usuario)

- Se pierde instalar el mismo agente a project+global en una sola acción.
- Es un refactor grande: dominio Rust, discovery, args de CLI, estado del front, Header,
  AgentsDialog, y bastantes tests existentes se reescriben, no solo se parchan.
