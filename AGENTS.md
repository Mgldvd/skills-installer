# AGENTS.md

## Propósito del repositorio

Skills Installer es una aplicación de escritorio para descubrir, organizar e instalar Agent Skills remotos o locales. Una misma base Rust sirve a la interfaz Tauri/Vue y a la CLI.

El frontend está construido con Vue 3, TypeScript, Vite y SCSS. El backend utiliza Rust y Tauri v2. Las instalaciones reales se delegan al Skills CLI y transmiten comando, `stdout`, `stderr`, progreso y errores hacia la interfaz.

## Estructura principal

- `frontend/src/App.vue`: composición de la aplicación y conexión de los flujos principales.
- `frontend/src/components/`: componentes Vue organizados en carpetas PascalCase, normalmente con un `.vue` y un `.scss` del mismo nombre.
- `frontend/src/composables/`: estado reactivo y operaciones de dominio del frontend.
- `frontend/src/services/backend.ts`: única capa autorizada para invocar comandos Tauri.
- `frontend/src/styles/`: tokens, estilos base y mixins compartidos.
- `frontend/src/types/`: contratos TypeScript que reflejan los DTO de Rust.
- `frontend/tests/`: pruebas Vitest y Vue Test Utils.
- `src-tauri/src/commands/`: fachada delgada de comandos Tauri.
- `src-tauri/src/app/`: servicios de aplicación y reglas de negocio.
- `src-tauri/src/domain/`: DTO y modelos compartidos.
- `src-tauri/src/installer/`: integración con Skills CLI.
- `src-tauri/src/process/`: ejecución segura y streaming de procesos.
- `configs/skills.yaml`: catálogo predeterminado embebido.

## Estado actual de la experiencia

- `Add Skill` abre directamente el diálogo para agregar un Skill remoto.
- `Local Skill Source` se configura y refresca desde Preferences.
- PRESELECT contiene los Packs, el estado de selección y la acción `Clear`.
- `Install Selected` permanece en el footer y abre confirmación cuando corresponde.
- Las tarjetas tienen altura uniforme y separan selección, descripción y edición.
- La descripción abre un diálogo con el contenido completo y enlace a `skills.sh` cuando existe.
- La edición de un Skill permite actualizar sus datos y Packs individuales.
- `PackBadge` es la fuente visual compartida para Packs en PRESELECT, tarjetas y menú Packs.
- El panel de instalación muestra en vivo los comandos, `stdout`, `stderr` y errores.

## Reglas de implementación

1. Respeta estrictamente el alcance solicitado. Una tarea visual no autoriza cambios de lógica, persistencia, instalación o contratos Rust/Tauri.
2. Conserva los cambios existentes del usuario. No reviertas archivos no relacionados ni uses comandos destructivos.
3. Usa `<script setup lang="ts">` y mantén el orden `template`, `script`, `style` en componentes Vue.
4. Mantén los estilos específicos junto al componente. Los tokens compartidos pertenecen a `frontend/src/styles/tokens.scss`.
5. Reutiliza componentes y variantes antes de crear implementaciones visuales paralelas. Para Packs, usa `components/PackBadge/PackBadge.vue`.
6. No introduzcas Pinia ni otra librería de estado sin una necesidad explícita. El proyecto usa `useAppState` y composables enfocados.
7. Las llamadas Tauri deben pasar por `frontend/src/services/backend.ts`; los componentes no deben invocar Tauri directamente.
8. Los comandos Tauri deben ser fachadas delgadas. La lógica pertenece a los servicios de `src-tauri/src/app/`.
9. Nunca ejecutes comandos de instalación mediante strings de shell. Conserva `Command::new(program).args(args)` y el aislamiento de `ProcessRunner`.
10. No cambies npm por pnpm o yarn.

## UI, UX y accesibilidad

- Usa HTML semántico: botones para acciones, enlaces para navegación externa y `<dialog>` para modales.
- Todo control interactivo debe funcionar con teclado y tener un foco visible.
- Usa `aria-pressed` para toggles y un indicador adicional al color para comunicar selección.
- No uses emojis como iconos estructurales; usa SVG con `aria-hidden="true"` cuando sea decorativo.
- Mantén áreas táctiles de al menos 44 px en dispositivos de puntero grueso.
- Soporta temas claro y oscuro mediante tokens semánticos.
- Respeta `prefers-reduced-motion` en transiciones nuevas.
- Diseña para la ventana mínima de 760 × 560 y evita overflow horizontal en tamaños menores.
- Verifica la preferencia de escala tipográfica máxima; textos largos deben envolver sin romper el layout.
- No muestres fallbacks como Pack (`Other`, `Default`, `Uncategorized`) salvo que sean Packs configurados reales.

## Sistema visual de Packs

- Cada Pack tiene `name` y `color`; el mismo color debe aparecer en todos sus contextos.
- Usa `PackBadge` para representación estática o interactiva.
- El color del Pack se reserva principalmente para el punto y acentos sutiles.
- Las tarjetas muestran badges compactos y no interactivos; si no hay Packs, no renderizan un placeholder.
- PRESELECT y el menú Packs usan botones con estados activos, parciales, inactivos y deshabilitados.
- No reintroduzcas estilos duplicados como `pack-pill`, `tag-toggle__dot` o badges con geometrías independientes.

## Paleta de Packs

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

Mantén sincronizadas las fuentes de verdad cuando una tarea autorice cambiar la paleta: tokens SCSS, selectores Vue, utilidad TypeScript, configuración embebida y `src-tauri/src/config/color.rs`.

## Validación requerida

Para cambios frontend ejecuta desde `frontend/`:

```bash
npm run typecheck
npm run lint
npm test
npm run build
```

Si `frontend/dist` tiene permisos incompatibles, valida Vite con un destino temporal:

```bash
npx vite build --outDir /tmp/skills-installer-build
```

Para cambios Rust/Tauri, cuando el toolchain esté disponible, ejecuta desde la raíz:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
frontend/node_modules/.bin/tauri build --no-bundle
```

Si Rust no está instalado, informa la limitación con precisión; no presentes la compilación Tauri como aprobada.

## Entrega

Resume qué cambió, menciona los archivos principales y reporta las validaciones realmente ejecutadas. Señala cualquier comprobación bloqueada y su causa concreta.
