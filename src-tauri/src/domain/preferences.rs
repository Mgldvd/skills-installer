use serde::{Deserialize, Serialize};

use super::install::InstallScope;

/// The five explicit presets from the product spec. `f64` values are what get
/// persisted/validated; the labels are a frontend concern.
pub const FONT_SCALE_MIN: f64 = 0.90;
pub const FONT_SCALE_MAX: f64 = 1.40;
pub const FONT_SCALE_PRESETS: [f64; 5] = [0.90, 1.00, 1.10, 1.25, 1.40];

/// Persisted separately from skill configuration (own JSON file) because it is
/// pure UI/installation-default state, not shareable/curated content — see
/// `preferences::service`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPreferences {
    pub font_scale: f64,
    #[serde(default)]
    pub default_agents: Vec<String>,
    #[serde(default, skip_serializing)]
    pub default_agent: Option<String>,
    pub copy_by_default: bool,
    pub confirm_before_install: bool,
    pub continue_after_failure: bool,
    #[serde(default = "default_accent")]
    pub accent: String,
    #[serde(default)]
    pub local_source_path: Option<String>,
    #[serde(default)]
    pub compact_cards: bool,
    /// User-customized display order for the agent list (Agents dialog and
    /// each Skill card's agent icons) — drag-and-drop reordering persists
    /// here. Empty means "no customization yet"; callers fall back to the
    /// built-in `SUPPORTED_AGENTS` order and append any agent id missing
    /// from a saved (possibly stale) order.
    #[serde(default)]
    pub agent_order: Vec<String>,
    /// The single active scope for the whole app, remembered across
    /// launches — see `InstallScope`. Replaced the old per-agent
    /// `agent_scopes` map (see `REFACTOR_PROJECT_GLOBAL_SCOPE.md`); an old
    /// file's `agentScopes` key is simply ignored by serde, landing here on
    /// the default `Project`.
    #[serde(default)]
    pub last_scope: InstallScope,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            font_scale: 1.0,
            default_agents: default_agents(),
            default_agent: None,
            copy_by_default: true,
            confirm_before_install: true,
            continue_after_failure: true,
            accent: default_accent(),
            local_source_path: None,
            compact_cards: false,
            agent_order: Vec::new(),
            last_scope: InstallScope::Project,
        }
    }
}

fn default_agents() -> Vec<String> {
    vec!["universal".to_string()]
}
fn default_accent() -> String {
    "#F43F75".to_string()
}
