---
type: Project Status
title: Current project status
description: Current mutable state, active work, and immediately relevant pending items.
status: stable
---

# Current project status

_Last updated: 2026-09-08._

## In progress / uncommitted

Nothing. `git status --short` is clean; `master` and `origin/master` both point at `d03c3c1`,
pushed. Everything described in this file's previous revision (the Skills Control Deck rename,
the SkillCard "Update" badge/border/selection changes, the live-refresh-on-install/focus work,
and the `memory/` → `.memory/` migration) has since landed.

Commit history was also rewritten (`git filter-repo`, local rewrite + `push --force-with-lease`,
each pass backed by a bundle and verified: identical trees, preserved author/dates, no merges,
no AI attribution) to follow
[commit-convention](https://github.com/Mgldvd/commit-convention) — every one of the 44 commits
now reads `type(scope): <shape> - description`, column-aligned with dot leaders, and the root
commit is that convention's fixed `init...............: ○ - 🌱.`. Any SHA referenced in an
older revision of this file is stale as a result — don't rely on this file's own history for
exact hashes, only `git log`.

## Recently verified

Last full verification pass was the rename (see prior revision of this file for the exact
command list: `cargo build/test/clippy/fmt`, `vue-tsc`, `vitest`, `eslint`, `vite build`, plus
visual checks in the real WebKitGTK window). The history rewrite itself only touches commit
messages — trees are byte-identical to what was already verified, so nothing needs re-running
on that account alone. Still, re-verify before assuming green if resuming work after a gap.

## Open / pending

- Nothing blocking.
- If the actual GitHub repository is ever renamed to match (`skills-installer` → something
  reflecting "Skills Control Deck"), update `gitUrl.spec.ts`/`PresetsDialog.spec.ts`'s example
  fixture URLs and `git remote` to match — deliberately not done, that's an external action.
- Known limitation, not a bug: "move to Installed live" and "refresh on focus" both go through
  the same full `refresh()` (re-scans disk); acceptable at today's catalog size, revisit only if
  that scan becomes measurably slow.
- Anyone with a local clone predating this history rewrite needs to reset to the new
  `origin/master` (SHAs all changed) rather than merge/pull normally.
