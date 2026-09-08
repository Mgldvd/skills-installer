---
type: Project Status
title: Current project status
description: Current mutable state, active work, and immediately relevant pending items.
status: stable
---

# Current project status

_Last updated: 2026-09-08._

## In progress / uncommitted

Branch `master`, `origin/master` at `4328d4f`. Everything below is uncommitted (some of
it staged, some not — see `git status --short` before assuming what's included in a
future commit).

- **App renamed**: "Skills Installer" → **"Skills Control Deck"**, end to end —
  `productName`/window title/bundle metadata (`tauri.conf.json`), Cargo package/lib/bin
  names (`skills-control-deck`, `skills_control_deck_lib`), the clap CLI name (drives
  `--version`, which the app's own dependency resolver also uses to detect and reject a
  `skills` PATH entry that's secretly itself — see `installer::dependency`), the
  `~/.local/bin` CLI-install symlink name, build/release scripts and Docker env var
  (`SKILLS_CONTROL_DECK_RELEASE_CONTAINER`), README/AGENTS.md prose. The XDG config
  directory also moved, `~/.config/skills-installer` → `~/.config/skills-control-deck`
  (`platform::xdg::app_config_dir`), with a one-time, best-effort `std::fs::rename`
  migration on first read so existing `preferences.json`/`presets.json`/`skills.yaml`
  keep working with no user action — **already confirmed to have run successfully
  against this machine's real config directory** (content and original file timestamps
  intact at the new path; not just unit-tested).
  Deliberately left alone: the real git remote (still `github.com/Mgldvd/skills-installer`
  — a hosting-platform rename is a separate, unrequested action) and its test fixtures in
  `gitUrl.spec.ts`/`PresetsDialog.spec.ts`, and the gitignored local `.env`
  (`LOCAL_APP="skills-installer.AppImage"`), which self-corrects the next time
  `.tasks/scripts/install-local.sh` runs against a rebuilt artifact.
- **SkillCard**: the "Update" badge moved from the footer into the header's top-right
  corner (next to the title), and a Skill with an update available now gets a
  full-strength warning-colored card border instead of the earlier subtle one.
- **Update workflow unified with Install Selected**: a fully-installed Skill with an
  update available can now be selected via its checkbox (previously blocked — "nothing
  left to install" no longer holds once an update exists), and stays selected across a
  `refresh()`. A new **"Update all (N)"** toolbar button (next to "Select missing")
  bulk-selects every outdated Skill, so the existing "Install Selected" button updates
  them — re-installing a Local Skill already *is* how an update happens, no separate
  bulk-update code path needed.
- **Removed the manual "Check for Skills updates" footer button** (`handleCheckForUpdates`,
  `isCheckingForUpdates`, its spinner CSS/`@keyframes app-shell-spin`) — redundant and
  confusing now that this check already runs automatically on app open and gets Skills
  moved live during install/on window focus (see the two items below, from the previous
  update to this file).
- `useInstallation.install()` takes an optional `onSkillSuccess(skillId)` callback, fired
  per `skill-success` progress event (not only after the whole batch finishes) —
  `App.vue`'s `runInstall` uses it via a single-flight `refreshSoon()` helper to move
  each Skill into "Installed" as soon as it's done, not at the end of the batch.
- `App.vue` also calls `refreshSoon()` on the window's `focus` DOM event, so Skills
  deleted by hand from `.agents/skills/` or the Local Skill Source catalog while the app
  was backgrounded stop showing as still-installed once the window is refocused.
  Focus-based, not a real filesystem watcher — a change made while the app stays focused
  and visible the whole time isn't caught.
- This `.memory/` bundle itself (migrated from the old `memory/` directory, OKF v0.2
  format) is also part of the current uncommitted change.

## Recently verified

All run and green in this session, most recently after the rename (backend rebuilt
under the new package/binary names, frontend rebuilt):
`cargo build`, `cargo test` (213 unit + 14 integration, 1 ignored cross-check),
`cargo clippy --all-targets`, `cargo fmt -- --check`, `vue-tsc --noEmit`, `vitest run`
(240 tests), `eslint .`, `vite build`. Also confirmed visually (WebKitGTK, not just
Chrome) for the SkillCard/toolbar changes and the renamed header. Not re-run since this
was written; re-verify before assuming still green if picking this back up later.

The commit at `4328d4f` (local-skill signing + git-aware catalog checks) was verified
the same way before being committed, plus a manual byte-for-byte cross-check against the
original Python `sign-folder` tool (`cargo test -- --ignored
cross_checks_against_the_original_python_tool`, `SIGN_FOLDER_BIN` pointing at it).

## Open / pending

- Nothing blocking. All of the above is functionally done, just not yet committed —
  confirm with whoever is picking this up before assuming it's fine to discard, and note
  the working tree currently has a mix of staged (the `.memory/` migration) and unstaged
  changes, not one clean diff.
- If the actual GitHub repository is ever renamed to match (`skills-installer` →
  something reflecting "Skills Control Deck"), update `gitUrl.spec.ts`/
  `PresetsDialog.spec.ts`'s example fixture URLs and `git remote` to match — deliberately
  not done as part of this change since that's an external, separate action.
- Known limitation, not a bug: "move to Installed live" and "refresh on focus" both go
  through the same full `refresh()` (re-scans disk); acceptable at today's catalog size,
  revisit only if that scan becomes measurably slow.
