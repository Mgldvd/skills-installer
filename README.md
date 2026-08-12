# Skills Installer

A desktop application for discovering, configuring, and installing [agent
skills](https://www.skills.sh/) — curated or local — built with **Tauri v2**,
**Rust**, and **Vue 3**. One executable serves as both a native Linux GUI
application and a scriptable command-line tool.

## Table of Contents

- [Project overview](#project-overview)
- [What Skills Installer Does](#what-skills-installer-does)
- [Architecture](#architecture)
- [Directory structure](#directory-structure)
- [Requirements](#requirements)
- [Development setup](#development-setup)
- [Development workflow](#development-workflow)
- [Frontend organization](#frontend-organization)
- [Configuration](#configuration)
- [Using the GUI](#using-the-gui)
- [CLI reference](#cli-reference)
- [Security model](#security-model)
- [Building](#building)
- [Application icons](#application-icons)
- [Linux desktop identity](#linux-desktop-identity)
- [Testing](#testing)
- [Release process](#release-process)
- [Troubleshooting](#troubleshooting)

## Project overview

Skills Installer lets you:

- Maintain a deliberate catalog of remote Skills added with their canonical
  `skills.sh/<owner>/<repository>/<skill>` URL.
- Include installable Skills discovered from a configurable **Local Skill
  Source** (default `~/.control/skill`).
- Organize reusable preselection sets with global Packs, without adding Pack
  clutter or category badges to normal Skill cards.
- **Select, preview, and install** skills through the real
  [`skills`](https://www.npmjs.com/package/skills) CLI
  (`npx skills add <owner>/<repo> --skill <name>`), with streamed progress
  and cancellation.
- Do all of the above from either a native GUI window or the same binary's
  command-line interface — both call the same Rust application services, so
behavior never diverges between the two.

## What Skills Installer Does

Skills Installer is a catalog and installation front end. It is not an agent,
a Skill runtime, or a replacement for the Skills CLI. The normal workflow is:

1. Add the remote `skills.sh` Skills you want to keep in your curated catalog.
2. Optionally configure a folder of local Skill definitions. Each child Skill
   is identified by its `SKILL.md` file and available front matter.
3. Select Skills, choose Project or Global scope, and select one or more agent
   targets (including Universal `.agents`).
4. Review the visible destination and agents, then install through the Rust
   installation service.

The **Local Skill Source is input**, not an installation destination. Changing
it never changes the current project, global scope, or agent target paths.
Project installations default to the directory from which the GUI was
launched. Agent preferences, appearance, failure behavior, local-source path,
Packs, and catalog configuration persist in XDG-compatible user configuration.

The footer separates the main responsibilities:

- **Skills** manages remote catalog entries and the local source folder.
- **Preferences** controls proportional interface size, accent, installation
  scope, copy/link behavior, continue-on-failure, and portable JSON
  import/export.
- **Agents** is a direct multi-select list of supported Skills CLI targets.
- **Packs** manages global Packs and one-click Skill assignments.
- **Clear** only clears the current selection; it never changes configuration.

## Architecture

```
Vue 3 GUI  ──┐
             │  Tauri invoke / IPC channels
             ▼
     Tauri command façade (src-tauri/src/commands/)  — thin, no business logic
             │
             ▼
     Application services (src-tauri/src/app/)
       ├── ConfigurationService  (src-tauri/src/config/)
       ├── SkillsService         (src-tauri/src/app/skills_service.rs)
       ├── PreferencesService    (src-tauri/src/preferences/)
       └── InstallationService   (src-tauri/src/app/installation_service.rs)
                     │
                     ▼
              Installer trait (src-tauri/src/installer/)
                     │
                     ▼
              ProcessRunner (src-tauri/src/process/) — tokio::process::Command

Rust CLI (src-tauri/src/cli/) ──┘  (same ApplicationServices, no window)
```

Both the GUI command layer and the CLI depend on the same
`app::ApplicationServices` composition root (`src-tauri/src/app/mod.rs`) —
installer behavior is never duplicated between the two entry points.

## Directory structure

```
skills-installer/
├── frontend/                  Vite + Vue 3 + TypeScript app
│   └── src/
│       ├── components/        One folder per component (.vue + .scss)
│       ├── composables/       useAppState, useSkills, useFilters, ...
│       ├── services/          services/backend.ts — the only invoke() callers
│       ├── styles/            tokens.scss, base.scss, dialog-base.scss, ...
│       └── types/             TS mirrors of the Rust domain DTOs
├── src-tauri/
│   ├── src/
│   │   ├── app/                ApplicationServices, SkillsService, InstallationService
│   │   ├── cli/                clap argument parsing + CLI command handlers
│   │   ├── commands/            Thin #[tauri::command] façade
│   │   ├── config/               Disk shape, migration, locator, atomic writes, color
│   │   ├── domain/                Shared DTOs (Skill, SkillGroup, InstallResult, ...)
│   │   ├── error/                  AppError
│   │   ├── installer/               Installer trait, SkillsCliInstaller, dependency resolution
│   │   ├── platform/                XDG paths, PATH augmentation
│   │   ├── preferences/              UiPreferences persistence
│   │   ├── process/                   ProcessRunner, ANSI stripping, command preview
│   │   └── skills/                     URL parser, local discovery, source resolver
│   ├── capabilities/           Tauri v2 capability files (minimal permissions)
│   ├── icons/                  Generated platform icon set
│   ├── app-icon.svg / app-icon.png   Canonical source icon
│   └── tauri.conf.json
├── configs/skills.yaml         Curated default configuration (embedded via include_str!)
├── dist/                       Copied final release artifacts
└── Makefile
```

## Requirements

### Tauri v2 / Linux prerequisites

Per the [official Tauri v2 Linux prerequisites](https://v2.tauri.app/start/prerequisites/),
building on Debian/Ubuntu-family systems needs:

```bash
sudo apt-get update
sudo apt-get install -y build-essential curl file pkg-config \
  libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libssl-dev \
  libayatana-appindicator3-dev patchelf
```

The raw executable itself **does** have runtime system-library requirements
on Linux — it links against WebKitGTK 4.1 and GTK 3, which is why the
generated `.deb` declares `Depends: libwebkit2gtk-4.1-0, libgtk-3-0`. It is
not a fully static binary.

### Rust

Install via [rustup](https://rustup.rs/); this project targets current
stable Rust (edition 2021).

### Node / frontend

Node 20+ and npm (the project keeps npm as its package manager — do not
switch to pnpm/yarn). The frontend's `@tauri-apps/cli` devDependency drives
`tauri dev`/`tauri build`.

### Packaging/inspection tools (optional, used by the verification pipeline)

```bash
sudo apt-get install -y desktop-file-utils librsvg2-bin imagemagick xvfb x11-utils
```

## Development setup

```bash
git clone <repo>
cd skills-installer
make frontend-deps      # npm install inside frontend/
```

## Development workflow

```bash
make dev          # starts the Tauri dev app (hot-reloading Vue + Rust)
make typecheck     # vue-tsc + cargo check
make lint          # eslint + cargo fmt --check + cargo clippy -D warnings
make test          # vitest run + cargo test (no real network/installs)
make build         # release executable + AppImage + .deb
make appimage      # copy the built AppImage into dist/
make deb           # copy the built .deb into dist/
make release        # full pipeline: typecheck, lint, test, then every artifact
```

The Tauri CLI's project-root auto-detection only searches the current
directory **downward**, not upward — so these targets always invoke it from
the repository root (`frontend/node_modules/.bin/tauri ...`), never from
inside `frontend/`.

## Frontend organization

Every substantial component lives in its own PascalCase folder:

```
components/SkillCard/
├── SkillCard.vue     template + <script setup lang="ts"> (never split apart)
└── SkillCard.scss     component-specific styles only
```

Every `.vue` file follows the mandatory order `template` → `script` →
`style`. Global/shared styles live only under `frontend/src/styles/` — see
`tokens.scss` for the color/typography design tokens and
`dialog-base.scss` for the shared native-`<dialog>` reset mixins every
dialog component uses.

State management is a single reactive module-level store
(`composables/useAppState.ts`) plus focused composables
(`useSkills`, `useFilters`, `useInstallation`, `usePreferences`) — Pinia
was deliberately not introduced; the state shape didn't justify the extra
dependency.

## Configuration

### Format

```yaml
version: 1

defaults:
  agent: universal
  copy: true
  scope: project

tags:
  - id: frontend
    name: Frontend
    color: "#F43F75"
    order: 10
    enabled: true

skills:
  - id: triage
    name: Issue Triage
    url: https://www.skills.sh/mattpocock/skills/triage
    description: Helps triage development issues.
    tags: [frontend]
    preselected: true
    enabled: true
```

A minimal legacy shape also works, with no manual migration required —
missing fields default safely; the internal `tags` key is retained for configuration compatibility:

```yaml
skills:
  - name: Example
    url: https://www.skills.sh/mattpocock/skills/triage
    preselected: true
```

### Precedence (highest to lowest)

1. `--config <path>` (CLI flag / GUI startup override)
2. `./skills.yaml`
3. `./skills.confg` (legacy filename)
4. `$XDG_CONFIG_HOME/skills-installer/skills.yaml` (falls back to `~/.config/skills-installer/skills.yaml`)
5. The embedded curated default (`configs/skills.yaml`, compiled into the
   binary via `include_str!`) — used only when nothing above exists.

Editing configuration while running on the embedded default materializes a
real file at the XDG path first; the embedded resource itself can never be
written to. Every write is atomic: serialize → write a temp file in the
same directory → `fsync` → rename over the destination, with a
`<name>.bak` copy of whatever was there before.

### Adding a skill

Open **Skills** in the footer and choose **+ Add Skill**. Paste a
`https://www.skills.sh/<owner>/<repository>/<skill-name>` URL; the dialog
shows the derived owner/repository/skill name once the URL resolves via
the Rust-side `SkillUrlParser`. Display name and description are optional.

### Editing a skill

Open **Skills** and choose **Edit** beside a remote catalog entry. Changing
the URL re-validates and re-parses it in Rust. Removing a catalog entry never
uninstalls copies that already exist in a project or agent directory.

### Packs

Packs are global reusable preselection sets, not categories or main-grid
filters. The Packs dialog shows every enabled Pack on every Skill row; one
click assigns or unassigns it. Pack names and strict `#RRGGBB` colors are
validated in Rust. Existing `tags` configuration remains supported as the
stable persisted representation, so this terminology change is non-destructive.

### Local skills

The folder configured under **Skills → Local Skill Source** is scanned for
`<dir>/SKILL.md` entries and `name`/`description` front matter. It defaults
to `~/.control/skill` and is a source catalog, never an installation target.
An explicit remote catalog entry wins when it has the same technical Skill
name, preventing confusing duplicate cards. Installed detection separately
checks the active project's `.agents/skills` directory.

## Using the GUI

The main window keeps destination and agent targets visible above the clean
Skill grid. The persistent footer opens lightweight Skills, Preferences,
Agents, and Packs dialogs and provides Clear and Install Selected. Font scale
applies proportionally to every semantic typography token, not just body
text — see `--font-scale` in `styles/tokens.scss`.

## CLI reference

```bash
skills-installer                    # launch the GUI (default)
skills-installer gui                # launch the GUI explicitly

skills-installer list                       # list configured + local skills
skills-installer list --installed           # only what's actually installed
skills-installer list --json                # machine-readable, no decorative text

skills-installer install                    # install the currently preselected skills
skills-installer install skill-a skill-b    # install specific configured skill ids
skills-installer install --all              # install every enabled configured skill
skills-installer install --dry-run          # preview the commands, execute nothing
skills-installer install --yes              # skip the interactive confirmation prompt
skills-installer install --agent <a> --global --copy --stop-on-error --json

skills-installer doctor             # Skills CLI availability + configuration health
skills-installer doctor --json
skills-installer version
skills-installer --help
```

Global options: `--config <path>`, `--debug` (verbose structured logging).

Exit codes: `0` success · `1` operation/install error · `2` invalid
arguments/configuration · `3` missing dependency · `130` interrupted
(Ctrl+C, or an install result that reports `cancelled: true`).

`install`'s interactive "Continue? [y/N]" prompt is skipped automatically
for `--yes`, `--dry-run` (nothing destructive happens), and `--json`
(machine consumers must never block on stdin).

## Security model

- **No shell interpolation anywhere.** Every process spawn goes through
  `process::ProcessRunner`, which always calls
  `Command::new(program).args(args)` with a `Vec<String>` — never
  `sh -c`/`bash -c`/`eval`, never a concatenated string.
- **The frontend has no filesystem or shell access.** Installing, listing,
  and removing skills all happen through dedicated `#[tauri::command]`s
  backed by Rust process-spawning code — the Tauri shell plugin is never
  used, and it isn't even a dependency of this project.
- **Tauri capabilities are minimal** (`src-tauri/capabilities/default.json`):
  only `core:default`, `core:window:default`, and `core:event:default`. No
  `fs:*`, no `shell:*`, no `dialog:*` — no plugin is registered unless the
  frontend actually calls into it.
- **All user input is validated in Rust**, not just the frontend: skill/group
  ids and agent ids (slug charset), URLs (via the `url` crate — rejects
  `javascript:`/`file:`/`data:`/host-confusion/embedded-credentials), group
  colors (strict hex), font scale (0.90–1.40 range).
- **The `skills` CLI process is always invoked non-interactively** (`--yes`
  is always passed) since a GUI-spawned child process has no TTY to answer
  prompts — independent of the app's own "confirm before installation"
  preference, which only gates whether *our* confirmation dialog appears
  before anything is spawned at all.

## Building

```bash
make build       # release executable + configured bundles (AppImage, .deb)
```

This runs the Tauri v2 bundler
(`frontend/node_modules/.bin/tauri build`, executed from the repo root
since the Tauri CLI only searches downward for `src-tauri`). Bundle targets
are set in `src-tauri/tauri.conf.json`'s `bundle.targets`.

### Raw executable

```bash
make appimage    # implies build; also copies to dist/
ls dist/
# dist/skills-installer                                   (release binary — build via `make release`)
# dist/Skills-Installer-<version>-x86_64.AppImage
# dist/skills-installer_<version>_amd64.deb
```

The raw executable at `src-tauri/target/release/skills-installer` supports
both the GUI and every CLI subcommand directly, with no packaging step
required.

### AppImage

AppImage portability depends on the Linux distribution used to build it —
build on an appropriately old baseline (Tauri's docs specifically mention
Ubuntu 22.04 / Debian 12) for the widest compatibility, since WebKitGTK's
ABI is the limiting factor. This project was built and verified on Ubuntu
26.04 ("resolute") with WebKitGTK 4.1.

### Debian package

```bash
dpkg-deb -c dist/skills-installer_<version>_amd64.deb   # verify contents
dpkg-deb -I dist/skills-installer_<version>_amd64.deb   # verify metadata/deps
```

Verified contents: `usr/bin/skills-installer`, a desktop entry under
`usr/share/applications/`, and icons at 32/128/256/512px under
`usr/share/icons/hicolor/*/apps/skills-installer.png`. `Depends:
libwebkit2gtk-4.1-0, libgtk-3-0` is set automatically by the bundler.

**Known Tauri quirk:** the generated `.desktop` file's *name on disk* is
`Skills Installer.desktop` — Tauri derives the filename directly from
`productName` without sanitizing spaces out of it. The file's actual
content is fully correct and passes `desktop-file-validate` with zero
warnings (`Exec=skills-installer`, `Icon=skills-installer`,
`StartupWMClass=skills-installer`, `Name=Skills Installer`); this is
purely a filesystem-naming cosmetic quirk of the current Tauri v2 bundler,
not a functional defect, and there is no config knob to change it — see
`DebConfig` in the [Tauri v2 config schema](https://schema.tauri.app/config/2).

## Application icons

The canonical source is the root-level `icon.png`. The Linux icon set used
by the application window, desktop integration, AppImage, and DEB bundles
is generated from that single source with the official tool:

```bash
frontend/node_modules/.bin/tauri icon icon.png   # run from repo root
```

Only the Linux-relevant sizes are kept (`32x32.png`, `64x64.png`,
`128x128.png`, `128x128@2x.png`, `icon.png`) — the icon command also
generates Windows/iOS/Android assets by default, which are deleted since
this project targets Linux only. All filenames are lowercase (verified
with `find`/`identify`) and every PNG is confirmed RGBA (`identify -format
"%[channels]"` reports `srgba`) — a non-RGBA source icon is rejected
outright by `tauri::generate_context!()` at compile time.

## Linux desktop identity

| Concept | Value |
|---|---|
| Display name | Skills Installer |
| Binary / CLI command | `skills-installer` |
| Tauri `productName` | Skills Installer |
| Tauri bundle identifier | `com.example.skills-installer` (placeholder — rename in `tauri.conf.json` if a real domain is available) |
| Icon theme name | `skills-installer` |
| Runtime `WM_CLASS` | `"skills-installer", "Skills-installer"` (verified via `xprop`, see below) |

### WM_CLASS troubleshooting

Verified empirically, not assumed — launched under Xvfb and inspected with
`xprop`:

```bash
xvfb-run -a skills-installer &
xdotool search --name "Skills Installer"
xprop -id <window-id> WM_CLASS
# WM_CLASS(STRING) = "skills-installer", "Skills-installer"
```

The `.desktop` file's `StartupWMClass=skills-installer` matches the first
(instance) `WM_CLASS` string, which is what desktop environments match
against for launcher-to-running-window association.

### Wayland troubleshooting

X11 `WM_CLASS` alone does not guarantee GNOME/KDE launcher association
under native Wayland — some compositors use the desktop file's
`Exec`/id matching instead. This project's sandbox has no real Wayland
compositor available to verify against, so **the launcher-icon-to-running-window
association on a real GNOME/KDE Wayland session should be manually
verified** after installing the `.deb` on a real desktop, in addition to
the automated X11/Xvfb check above.

### PATH differences between terminal and GUI

GUI-launched Linux processes (desktop entry, AppImage) commonly see a
narrower `$PATH` than an interactive shell — they don't source
`.bashrc`/`.profile`/nvm/pnpm init scripts. `installer::dependency`
therefore never trusts `std::env::var("PATH")` alone: it also probes
`~/.local/share/pnpm`, `~/.npm-global/bin`, `~/.local/bin`, `~/.cargo/bin`,
and every `~/.nvm/versions/node/*/bin`, merging whichever exist onto the
search path (see `platform::path_augment`).

## Testing

```bash
make test
```

- **Rust** (`cargo test`): config parsing/migration/precedence,
  atomic writes, the URL parser (including the real
  `https://www.skills.sh/mattpocock/skills/triage` and
  `https://www.skills.sh/pbakaus/impeccable/impeccable` examples), local
  local-source discovery, Pack validation and CRUD, the installer's
  command construction and result aggregation, cancellation, dependency
  resolution, and CLI argument parsing/dispatch — all using
  `installer::FakeInstaller`/`FakeProcessRunner` test doubles, never a real
  network call or child process.
- **Vue** (`vitest run`): selection, search, clean Skill cards, catalog
  dialogs, direct Agent and Pack toggles, Preferences, and proportional font
  scaling. Backend calls are mocked; tests never perform real installation.

## Release process

```bash
make release
```

Runs `typecheck` → `lint` → `test` → builds and copies every artifact into
`dist/`. Before tagging a release, additionally:

1. Inspect `dpkg-deb -c`/`-I` output on the actual `.deb` (not just a
   successful exit code).
2. Run `desktop-file-validate` on the extracted `.desktop` file.
3. Audit icon filenames/dimensions with `find`/`identify`.
4. Launch the raw executable and the AppImage on a real desktop session
   (X11 and, separately, Wayland) and confirm launcher/taskbar/Alt+Tab
   icon association manually — this sandboxed environment can verify X11
   `WM_CLASS` via Xvfb, but not real compositor-level Wayland integration.
5. Install the `.deb` in a disposable environment and confirm the menu
   entry, CLI command, and dependency detection all work post-install.

## Troubleshooting

**"Skills CLI: NOT available"** — `doctor` reports the exact reason. The
app looks for an installed `skills` executable first (including common
Node install locations beyond a narrow GUI `$PATH`), then falls back to
`npx --yes skills`. If neither resolves, install with `npm install -g
skills` or ensure `npx` is reachable.

**Config changes aren't taking effect** — check the precedence order
above; a `./skills.yaml` or `./skills.confg` in the current working
directory always wins over the XDG user config and the embedded default.
Use `--config <path>` to force a specific file.

**AppImage won't run in a minimal container** — some containers lack FUSE;
try `--appimage-extract-and-run`.
