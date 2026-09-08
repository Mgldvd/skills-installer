---
type: Decision
title: Un solo scope de instalación activo (Project o Global), no por agente
description: El scope de instalación es un modo único para toda la app (InstallScope::{Project, Global}), no una selección independiente por agente.
status: stable
decision_status: accepted
date: 2026-08-25
---

# 0001. Un solo scope de instalación activo (Project o Global), no por agente

## Contexto

El scope de instalación vivía en `AgentScopeSelection { project: bool, global: bool }`
— no exclusivo, guardado por agente en `UiPreferences.agent_scopes: HashMap<agent_id,
AgentScopeSelection>`. Esto permitía, en teoría, que cada agente tuviera una
combinación distinta de Project/Global dentro de la misma instalación.

En la práctica esto generaba problemas concretos:

- La UI y el código de instalación/desinstalación tenían que armar hasta 2 grupos de
  invocación al CLI (`add_args_for_skill`) porque `--global` aplica a toda la
  invocación del CLI `skills`, no por agente.
- `remove()` nunca pasaba `--global` — era efectivamente siempre project-scoped en la
  práctica, sin importar lo que dijera `AgentScopeSelection`. Un bug real, no solo
  complejidad accidental: un uninstall no podía representar honestamente qué scope
  estaba removiendo.
- El discovery (`AGENT_PROJECT_DIRS` bajo `project_root` vs `AGENT_GLOBAL_DIRS` bajo
  `$HOME`) se mezclaba siempre en `SkillsService::load_state_for`, así que el estado
  "installed" mostrado en la UI no reflejaba con precisión un scope concreto.

## Decisión

Reemplazar el scope por-agente por un **modo único activo para toda la app**
(`InstallScope::{Project, Global}`), controlado desde un selector en el header:

- **Modo Project**: la carpeta seleccionada define destino y discovery (comportamiento
  histórico de la app).
- **Modo Global**: no hace falta carpeta; cwd de fallback es `$HOME` (siempre existe,
  a diferencia de "última carpeta usada"). El header cambia de aspecto (acento ámbar +
  badge "GLOBAL", independiente del accent color del usuario) para que sea imposible
  confundir en qué modo se está.
- El modo persiste entre lanzamientos (`UiPreferences.last_scope`), reemplazando el
  reseteo a carpeta vacía que tenía `state.projectRoot`.
- Un `preferences.json` viejo con la clave `agentScopes` sigue cargando sin error —
  serde la ignora y cae en el default `Project` — en vez de fallar toda la carga.

## Consecuencias

- **Se pierde** la capacidad de instalar el mismo agente a Project y Global
  simultáneamente en una sola acción. Aceptado explícitamente: para tener un agente en
  ambos lados, se cambia de modo y se instala dos veces.
- `domain::Project` (el preset de skills seleccionados, sin ruta, persistido en
  `projects.json`) se renombró a **`Preset`** (`ProjectsService` → `PresetsService`,
  `ProjectsDialog.vue` → `PresetsDialog.vue`, etc.) — quedaba un choque de nombres
  irresoluble entre ese concepto y el nuevo `InstallScope::Project`. `list()` migra un
  `projects.json` viejo a `presets.json` en el primer arranque.
- `AgentsDialog` perdió los toggles Project/Global por agente; ahora solo muestra
  ambas rutas destino como información de referencia para cada agente.
- Durante la verificación manual de este refactor se encontró y corrigió un bug real
  no relacionado con el diseño en sí: `AGENT_GLOBAL_DIRS`
  (`src-tauri/src/skills/discovery.rs`) y `SUPPORTED_AGENTS[].globalPath`
  (`frontend/src/types/preferences.ts`) tenían rutas globales desactualizadas para
  varios agentes (transcritas del README del CLI `skills`, que no coincidía con lo que
  el CLI realmente escribe en disco). Corregido y verificado contra el CLI real.
- Refactor grande y deliberado: dominio Rust, discovery, args de CLI, estado del
  frontend, `AppHeader`, `AgentsDialog` y varios tests existentes se reescribieron, no
  se parchó por encima.
