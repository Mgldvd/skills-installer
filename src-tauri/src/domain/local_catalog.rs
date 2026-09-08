use serde::{Deserialize, Serialize};

/// Result of `SkillsService::check_local_updates`: which installed Local
/// Skills are outdated, plus whether the Local Skill Source catalog itself
/// is under git version control — that's what lets the check cheaply tell
/// which catalog skills actually need re-signing (see
/// `skills::catalog_git`) instead of hashing the whole catalog every time.
/// The GUI uses `catalog_versioned`/`catalog_dirty_skill_names` to nudge the
/// user toward `git init`-ing an unversioned catalog, or committing after
/// this check has (re)signed whatever changed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalUpdatesReport {
    pub outdated_skill_ids: Vec<String>,
    /// `None` when there's no catalog directory to even check (no Local
    /// Skill Source configured, or the configured/default path doesn't
    /// exist on disk) — the GUI shows no versioning nudge in that case,
    /// since there's nothing there for the user to `git init`.
    pub catalog_versioned: Option<bool>,
    pub catalog_dirty_skill_names: Vec<String>,
}
