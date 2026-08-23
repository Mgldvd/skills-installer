use serde::{Deserialize, Serialize};

/// Where a skill's files come from. Kept as an explicit enum (rather than a
/// boolean flag) so new source kinds can be added without touching every call
/// site that currently checks `skill.local`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SkillSource {
    /// Installed/installable via the Skills CLI from a GitHub-backed repository.
    Remote,
    /// Discovered on disk under `.agents/skills/<dir>/SKILL.md`.
    Local { path: String },
}

/// The single domain representation of a skill, used for both remote
/// (configured) and local (discovered) skills — the GUI must not need to know
/// which one it's looking at beyond the `local`/`source` fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub source: SkillSource,
    pub repository: String,
    pub repository_url: String,
    pub skill_name: String,
    pub skills_url: String,
    pub group_id: String,
    pub tags: Vec<String>,
    pub preselected: bool,
    pub local: bool,
    pub installed: bool,
    /// Ids of the agents whose install destination (scoped to whichever
    /// project root `installed` was computed against) actually contains this
    /// skill. Several agent ids can share one destination directory, so they
    /// always appear or disappear together — see `discover_installed_agents`.
    #[serde(default)]
    pub installed_agents: Vec<String>,
    pub enabled: bool,
}

/// A skill actually found installed on disk (as opposed to `Skill.installed`,
/// which is a flag on the *configured* skill). Kept separate because installed
/// skills can exist without a matching configuration entry (installed out of
/// band) — `get_installed_skills` returns these directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub path: String,
    pub agent: Option<String>,
}

/// A named set of skill ids — used to carry "which skills should this
/// operation apply to" across the CLI/GUI boundary without passing bare
/// `Vec<String>` around (keeps intent explicit at call sites).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSelection {
    pub skill_ids: Vec<String>,
}

impl SkillSelection {
    pub fn new(skill_ids: Vec<String>) -> Self {
        Self { skill_ids }
    }

    pub fn is_empty(&self) -> bool {
        self.skill_ids.is_empty()
    }
}
