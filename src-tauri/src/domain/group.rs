use serde::{Deserialize, Serialize};

/// A skill group. `color` is always a validated, normalized `#RRGGBB` string by
/// the time it reaches this type — validation happens once, in
/// `config::color`, at the config/service boundary, never in the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillGroup {
    pub id: String,
    pub name: String,
    pub color: String,
    pub order: i32,
    pub enabled: bool,
}

/// The reserved group id used for skills whose configured group no longer
/// exists (or was never set) — never presented to the user as a creatable
/// group, but always a valid `group_id` value.
pub const OTHER_GROUP_ID: &str = "other";
