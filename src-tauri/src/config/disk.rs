use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Raw on-disk YAML shape. Every field except `skills` is `#[serde(default)]`
/// so a bare legacy file containing only a `skills:` array deserializes
/// cleanly through the exact same struct as a full curated config —
/// requirement is "don't require the user to manually migrate", and the
/// simplest way to guarantee that is to make the modern shape a strict
/// superset of the legacy one at the type level, not maintain two parsers.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiskConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub defaults: DiskDefaults,
    #[serde(default)]
    pub groups: Vec<DiskGroup>,
    #[serde(default)]
    pub tags: Vec<DiskTag>,
    /// Accepted only for backwards compatibility. Saves never emit packs.
    #[serde(default, skip_serializing)]
    pub packs: Vec<DiskPack>,
    #[serde(default)]
    pub skills: Vec<DiskSkill>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub local_skill_tags: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiskTag {
    pub id: String,
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub order: i32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiskPack {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiskDefaults {
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default = "default_true")]
    pub copy: bool,
    #[serde(default)]
    pub scope: Option<String>,
}

impl Default for DiskDefaults {
    fn default() -> Self {
        Self {
            agent: None,
            copy: true,
            scope: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiskGroup {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub order: i32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiskSkill {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub preselected: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

fn default_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_curated_shape() {
        let yaml = r##"
version: 1
defaults:
  agent: universal
  copy: true
  scope: project
groups:
  - id: frontend
    name: Frontend
    color: "#E75480"
    order: 10
    enabled: true
skills:
  - id: triage
    name: Issue Triage
    url: https://www.skills.sh/mattpocock/skills/triage
    description: Helps triage development issues.
    group: testing
    tags: [issues, workflow]
    preselected: true
    enabled: true
"##;
        let config: DiskConfig = serde_norway::from_str(yaml).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.groups.len(), 1);
        assert_eq!(config.skills.len(), 1);
        assert_eq!(config.skills[0].id.as_deref(), Some("triage"));
    }

    #[test]
    fn parses_bare_legacy_shape() {
        let yaml = r#"
skills:
  - name: Example
    url: https://www.skills.sh/mattpocock/skills/triage
    preselected: true
"#;
        let config: DiskConfig = serde_norway::from_str(yaml).unwrap();
        assert_eq!(config.version, 1);
        assert!(config.groups.is_empty());
        assert_eq!(config.skills.len(), 1);
        assert_eq!(config.skills[0].id, None);
        assert_eq!(config.skills[0].name, "Example");
        assert!(config.skills[0].preselected);
        assert!(config.skills[0].enabled);
        assert!(config.skills[0].description.is_none());
        assert!(config.skills[0].group.is_none());
    }

    #[test]
    fn rejects_completely_invalid_yaml() {
        let yaml = "not: [valid, yaml structure for our schema because skills is a string";
        let result: Result<DiskConfig, _> = serde_norway::from_str(yaml);
        assert!(result.is_err());
    }
}
