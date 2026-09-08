# AGENTS.md

## Propósito del repositorio

Skills Control Deck es una aplicación Linux para descubrir, organizar e instalar Agent Skills remotos o locales. El mismo binario ofrece una interfaz de escritorio Tauri/Vue y una CLI; ambos caminos reutilizan los mismos servicios de aplicación escritos en Rust.

El frontend usa Vue 3, TypeScript, Vite y SCSS. El backend usa Rust 2021 (MSRV 1.85) y Tauri v2. Las instalaciones se delegan al Skills CLI mediante procesos con argumentos estructurados y transmiten comando, `stdout`, `stderr`, progreso, resultados y errores hacia la interfaz.

## Memoria del proyecto

La memoria del proyecto vive en `.memory/` (formato OKF v0.2). Antes de un trabajo
significativo, lee `.memory/index.md` y seguí solo la documentación relevante a la
tarea; lee `.memory/STATUS.md` cuando importe el estado actual o en curso. Tratá
`.memory/decisions/` como el porqué durable de decisiones significativas — no las
reabras sin confirmación explícita del usuario. Si un cambio de código deja la memoria
del proyecto falsa o materialmente incompleta, actualizá el documento afectado en el
mismo cambio.

## Estructura vigente

- `frontend/src/App.vue`: composición de la interfaz y conexión de los flujos principales.
- `frontend/src/components/`: componentes Vue en carpetas PascalCase, normalmente con un `.vue` y un `.scss` homónimos.
- `frontend/src/composables/`: estado reactivo compartido y operaciones de Skills, Packs, preferencias, instalación y toasts.
- `frontend/src/services/backend.ts`: wrappers tipados de los comandos Tauri y del selector nativo de carpetas.
- `frontend/src/services/tauri/client.ts`: único lugar autorizado para llamar directamente a `invoke()`.
- `frontend/src/styles/`: tokens semánticos, estilos base, utilidades y mixins compartidos.
- `frontend/src/types/`: contratos TypeScript que reflejan los DTO de Rust.
- `frontend/src/utils/`: utilidades puras de color y navegación por teclado.
- `frontend/tests/`: pruebas Vitest, jsdom y Vue Test Utils.
- `src-tauri/src/commands/`: fachadas delgadas expuestas a Tauri.
- `src-tauri/src/app/`: composición de servicios y reglas de aplicación.
- `src-tauri/src/domain/`: DTO, validación y modelos compartidos.
- `src-tauri/src/config/`: carga, migración, validación y escritura atómica de YAML.
- `src-tauri/src/preferences/`: persistencia de preferencias de interfaz e instalación.
- `src-tauri/src/skills/`: descubrimiento local, análisis de URLs e importación de Packs.
- `src-tauri/src/installer/`: integración con Skills CLI y resolución de dependencias.
- `src-tauri/src/process/`: ejecución segura, streaming, vista previa y limpieza ANSI.
- `src-tauri/src/cli/`: argumentos y ejecución de la CLI sin iniciar el bucle gráfico.
- `src-tauri/configs/skills.yaml`: catálogo predeterminado incluido en el binario.
- `src-tauri/tauri.conf.json`: ventana mínima de 760 × 560 y bundles Linux AppImage/DEB.

## Estado actual del producto

- El encabezado muestra el destino, los Agents seleccionados y el estado del Skills CLI. En scope Project permite elegir una carpeta con el diálogo nativo; Global ignora ese destino.
- La barra PRESELECT muestra Packs reutilizables con estados completo, parcial, inactivo y deshabilitado. También permite limpiar la selección, filtrar Skills, ordenar por nombre o Pack y reordenar Packs con drag-and-drop o `Alt+Left`/`Alt+Right`.
- Los Skills seleccionados se ordenan primero. Las tarjetas pueden usar modo normal o compacto, muestran estado Installed/Local y los Packs asignados, y abren `EditSkillDialog` para consultar o modificar detalles.
- `Add Skill` acepta una URL individual de `skills.sh` o una URL de Pack con formato `https://skills.sh/p/...`. La importación de Pack crea el Pack y agrega al catálogo los Skills remotos que no estén duplicados; no instala nada.
- `Edit Skill` permite actualizar URL, nombre, descripción, estado, preselección y Packs, o eliminar una entrada remota del catálogo.
- El menú Packs permite crear, editar, eliminar y asignar Packs a Skills. El nombre visible es “Pack”, aunque la representación persistida y varios tipos internos conservan `tag`/`tags` por compatibilidad.
- Preferences controla escala tipográfica, acento, tarjetas compactas, scope, copia frente a enlace, confirmación, continuación tras errores y Local Skill Source. También importa/exporta configuración JSON e instala el lanzador Linux `~/.local/bin/skills`.
- Agents es una selección múltiple. La lista vigente vive en `frontend/src/types/preferences.ts`; no dupliques esos ids en componentes.
- La instalación usa un `Channel` por invocación, muestra eventos en vivo, permite cancelar y refresca el catálogo al terminar.
- La aplicación soporta temas claro y oscuro mediante tokens semánticos y `prefers-color-scheme`.

## Compatibilidad y fuentes de verdad

- La configuración activa se busca, en orden, mediante `--config`, `./skills.yaml`, `./skills.confg`, la ruta XDG del usuario y finalmente `src-tauri/configs/skills.yaml` embebido. Conserva esta precedencia y sus pruebas.
- El YAML moderno persiste Packs bajo `tags`. La clave `packs` solo se acepta como entrada heredada y no se vuelve a serializar.
- `groups` y los componentes/commands relacionados permanecen por compatibilidad histórica, pero no forman parte del flujo gráfico registrado actual. No construyas nuevas funciones sobre Groups ni los presentes en la UI salvo que la tarea autorice explícitamente reactivarlos.
- Las habilidades locales se descubren por `SKILL.md`. `localSkillTags` conserva sus asignaciones de Packs. La carpeta local es una fuente del catálogo, nunca el destino de instalación.
- Una entrada remota explícita prevalece frente a una local con el mismo nombre técnico.
- El catálogo embebido, los DTO TypeScript y los modelos Rust deben seguir siendo compatibles. Si cambia un contrato IPC, actualiza ambos lados y sus pruebas.

## Reglas de implementación

1. Respeta estrictamente el alcance solicitado. Una tarea visual no autoriza cambios de lógica, persistencia, instalación, CLI ni contratos Rust/Tauri.
2. Conserva los cambios existentes del usuario. No reviertas archivos no relacionados ni uses comandos destructivos.
3. En componentes Vue usa `<script setup lang="ts">`.
4. Mantén estilos específicos junto al componente. Los tokens compartidos pertenecen a `frontend/src/styles/tokens.scss`; los mixins de diálogos pertenecen a `dialog-base.scss`.
5. Reutiliza componentes y variantes antes de crear implementaciones paralelas. Para Packs usa `components/PackBadge/PackBadge.vue`; para cerrar diálogos usa `CloseButton`; para sincronizar `<dialog>` usa `useNativeDialog`.
6. No introduzcas Pinia ni otra librería de estado sin una necesidad explícita. El proyecto usa un store reactivo de módulo en `useAppState` y composables enfocados.
7. Añade wrappers tipados en `frontend/src/services/backend.ts`; nunca invoques comandos Tauri directamente desde componentes o composables. Solo `services/tauri/client.ts` llama a `invoke()`.
8. Los comandos Tauri deben ser fachadas delgadas. La lógica pertenece a `src-tauri/src/app/` o al módulo de dominio correspondiente.
9. Nunca construyas ejecuciones con strings de shell. Conserva `Command::new(program).args(args)`, `ProcessRunner`, las vistas previas seguras y el aislamiento del proceso.
10. Para progreso de instalación conserva un `tauri::ipc::Channel` ligado a cada invocación; no lo reemplaces por listeners globales que mezclen instalaciones.
11. Usa npm y `frontend/package-lock.json` como flujo oficial. No cambies el proyecto a pnpm, yarn o Bun.
12. El proyecto apunta únicamente a Linux desktop. No añadas rutas mobile ni supongas que AppImage elimina las dependencias de GTK/WebKitGTK.

## UI, UX y accesibilidad

- Usa HTML semántico: botones para acciones, enlaces para navegación externa y `<dialog>` para modales.
- Todo control interactivo debe funcionar con teclado y mostrar foco visible.
- Usa `aria-pressed` para toggles y un indicador adicional al color para comunicar selección o estado parcial.
- No uses emojis como iconos estructurales; usa SVG con `aria-hidden="true"` cuando sea decorativo.
- Mantén áreas táctiles de al menos 44 px en dispositivos de puntero grueso.
- Usa tokens semánticos para conservar temas claro/oscuro y acentos configurables.
- Respeta `prefers-reduced-motion` en transiciones nuevas.
- Diseña para la ventana mínima de 760 × 560, sin overflow horizontal y con textos largos capaces de envolver.
- Verifica la escala tipográfica máxima (`1.4`); no fijes alturas que corten contenido.
- No muestres fallbacks como Pack (`Other`, `Default`, `Uncategorized`) salvo que sean Packs configurados reales.
- Los controles de selección deben seguir siendo distinguibles sin depender únicamente del color.

## Sistema visual de Packs

- Cada Pack tiene `name`, `color`, `order` y `enabled`; el mismo color debe aparecer en todos sus contextos.
- Usa `PackBadge` para representación estática o interactiva.
- Reserva el color del Pack principalmente para el punto y acentos sutiles.
- Las tarjetas muestran badges compactos no interactivos; si no hay Packs, no renderizan un placeholder.
- PRESELECT y el menú Packs deben conservar estados activos, parciales, inactivos y deshabilitados.
- No reintroduzcas estilos duplicados como `pack-pill`, `tag-toggle__dot` o badges con geometrías independientes.

### Paleta de Packs

- Pink: `#F43F75`
- Red/Coral: `#F05252`
- Orange: `#F97316`
- Amber: `#F59E0B`
- Green: `#22C55E`
- Teal: `#14B8A6`
- Cyan: `#06B6D4`
- Blue: `#3B82F6`
- Indigo: `#6366F1`
- Violet: `#A855F7`

Cuando una tarea autorice cambiar la paleta, mantén sincronizados `frontend/src/utils/color.ts`, los selectores de `AddSkillDialog` y `TagsDialog`, cualquier selector heredado aún cubierto por pruebas, `src-tauri/configs/skills.yaml` y `src-tauri/src/config/color.rs`. La paleta de acentos de Preferences es un subconjunto independiente y solo debe cambiar si el alcance lo requiere.

## Validación requerida

Para cambios frontend ejecuta desde `frontend/`:

```bash
npm run typecheck
npm run lint
npm test
npm run build
```

Si `.generated/frontend` tiene permisos incompatibles, valida Vite con un destino temporal:

```bash
npx vite build --outDir /tmp/skills-control-deck-build
```

Para cambios Rust/Tauri, cuando el toolchain esté disponible, ejecuta desde la raíz:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
frontend/node_modules/.bin/tauri build --no-bundle
```

Para cambios que afecten ambos lados, ejecuta ambas baterías. `task typecheck`, `task lint` y `task test` son los equivalentes integrados. `task build` (y `artifacts`/`release`, que dependen de él) ya no corren en el host: solo dentro del contenedor Docker (`task docker:release`), para que todo artefacto distribuible salga del mismo entorno fijo y reproducible. No uses `task clean` sin autorización: elimina artefactos y directorios de salida.

Para cambios exclusivamente documentales basta con revisar rutas y comandos afectados y ejecutar `git diff --check`; no presentes las suites de aplicación como ejecutadas si no lo fueron.

Si Rust, Node o las librerías del sistema no están disponibles, informa la limitación con precisión; no presentes una compilación parcial como aprobada.

## Entrega

Resume qué cambió, menciona los archivos principales y reporta únicamente las validaciones realmente ejecutadas. Señala cualquier comprobación bloqueada y su causa concreta.
