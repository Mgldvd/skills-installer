use serde::{Deserialize, Serialize};

/// A named, saved snapshot of which Skills were installed for a particular
/// local project — lets the user re-select the same set after a fresh
/// clone (or on another machine) without checking the installed Skill
/// files themselves into git. Stored globally, independent of any one
/// project directory — see `PresetsService`.
///
/// Named `Preset`, not `Project`: an unrelated, unfortunately-named
/// `InstallScope::Project` (the active install destination/scope selected
/// in the header) already uses that word for something entirely different
/// — see `REFACTOR_PROJECT_GLOBAL_SCOPE.md` for why this type was renamed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub git_url: Option<String>,
    /// `skill_name` (not `id`) for every Skill that was installed at save
    /// time — the portable identifier that survives local `id` regeneration
    /// and matches across machines/catalogs.
    pub skill_names: Vec<String>,
}
