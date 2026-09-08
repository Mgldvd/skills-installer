---
type: Decision
title: Sin librería de estado global (Pinia); store reactivo de módulo
description: El estado compartido del frontend vive en un store reactivo de módulo (useAppState), no en Pinia ni otra librería de manejo de estado.
status: stable
decision_status: accepted
date: "fundacional; presente desde el primer commit del proyecto"
---

# 0002. Sin librería de estado global (Pinia); store reactivo de módulo

## Contexto

Skills Installer es una app de escritorio de un solo módulo de UI (no hay rutas ni
vistas independientes que necesiten compartir estado entre sí de forma desacoplada):
un header, una grilla de Skills y un puñado de diálogos, todos hijos directos de
`App.vue`. El estado que se comparte (preferencias, Skills cargados, progreso de
instalación, etc.) es relativamente chico y sus mutaciones están concentradas en
composables enfocados (`useSkills`, `useInstallation`, `usePreferences`, `usePresets`,
`useTags`, `useFilters`), no dispersas por toda la base de código.

## Decisión

No usar Pinia (ni Vuex ni ninguna otra librería de manejo de estado). El estado
compartido vive en un store reactivo de módulo, `useAppState`
(`frontend/src/composables/useAppState.ts`): un objeto `reactive()` a nivel de
módulo, exportado e importado donde se necesita, más los composables enfocados que
operan sobre él.

## Consecuencias

- Una dependencia menos, sin el boilerplate de definir stores/acciones/getters para
  un árbol de estado que no lo necesita.
- El patrón está protegido explícitamente en `AGENTS.md` (regla 6): no se debe
  introducir Pinia u otra librería de estado "sin una necesidad explícita" — si la
  app crece a múltiples vistas/rutas con estado genuinamente desacoplado entre sí,
  ese sería el momento de reabrir esta decisión, no antes.
- Riesgo aceptado: un store de módulo reactivo no da de forma gratuita herramientas
  de Pinia como devtools time-travel, plugins de persistencia, o namespacing entre
  módulos — irrelevante mientras el estado siga siendo tan chico y centralizado como
  hoy.
