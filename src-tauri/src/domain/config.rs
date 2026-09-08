use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::group::SkillGroup;
use super::install::InstallScope;
use super::skill::{Skill, UnrecognizedSkill};
use super::tag::SkillTag;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDefaults {
    pub agent: Option<String>,
    pub copy: bool,
    pub scope: InstallScope,
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            agent: None,
            copy: true,
            scope: InstallScope::Project,
        }
    }
}

/// The in-memory, frontend-facing configuration shape — deliberately distinct
/// from the on-disk YAML shape in `config::disk`. Remote (configured) skills
/// only; `SkillsService` merges in locally discovered skills on top of this.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationConfig {
    pub version: u32,
    pub defaults: ConfigDefaults,
    pub groups: Vec<SkillGroup>,
    pub tags: Vec<SkillTag>,
    pub skills: Vec<Skill>,
    /// Persisted Tag assignments for filesystem-discovered skills. These
    /// are merged into `Skill.tags` before state reaches the frontend.
    #[serde(default)]
    pub local_skill_tags: BTreeMap<String, Vec<String>>,
    /// Skills found installed on disk (for whichever scope was last loaded)
    /// whose `skill_name` matches nothing in `skills` — i.e. installed by
    /// some other means (manually, another tool) and not tracked by this
    /// app's catalog. Never merged into `skills` itself; see
    /// `SkillsService::load_state_for`. Each entry's `path` is what the
    /// GUI's "copy into catalog" action sends back to
    /// `SkillsService::copy_unrecognized_skill_into_catalog`.
    #[serde(default)]
    pub unrecognized_skills: Vec<UnrecognizedSkill>,
    /// Absolute path this config was loaded from, or `None` when running on
    /// the embedded default (nothing on disk was found).
    pub source_path: Option<String>,
    /// True once a write has materialized the embedded default into a real
    /// user config file (see `config::service`).
    pub is_embedded_default: bool,
    /// The directory the app is operating in (local skill discovery root,
    /// and the working directory `Installer` invocations run in) — shown
    /// in the GUI header as "Project: <path>".
    pub project_root: String,
}
