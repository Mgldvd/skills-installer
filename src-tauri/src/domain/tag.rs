use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillTag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub order: i32,
    pub enabled: bool,
}
