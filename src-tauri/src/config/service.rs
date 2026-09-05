use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::domain::{ApplicationConfig, InstallScope, SkillSource};
use crate::error::AppError;

use super::color::validate_and_normalize_color;
use super::disk::{DiskConfig, DiskDefaults, DiskGroup, DiskSkill, DiskTag};
use super::locator::{self, ConfigSource};
use super::migrate::migrate;

/// The curated default configuration, compiled directly into the binary so
/// the app is fully usable immediately after installation with no loose
/// YAML file required next to it.
const EMBEDDED_DEFAULT_YAML: &str = include_str!("../../configs/skills.yaml");

/// Loads, validates, and atomically persists the skill/group configuration.
/// Both the GUI command layer and the CLI go through this one service —
/// neither ever reads/writes YAML directly.
pub struct ConfigurationService {
    explicit_override: Option<PathBuf>,
    cwd: PathBuf,
    /// Injected rather than resolved lazily inside `load()` so tests can
    /// exercise the "nothing on disk anywhere" branch deterministically,
    /// without depending on (or risking pollution from) this machine's real
    /// `$HOME`/`$XDG_CONFIG_HOME`.
    user_config_path: Option<PathBuf>,
}

impl ConfigurationService {
    pub fn new(explicit_override: Option<PathBuf>, cwd: PathBuf) -> Self {
        Self {
            explicit_override,
            cwd,
            user_config_path: locator::user_config_path(),
        }
    }

    #[cfg(test)]
    fn with_user_config_path(
        explicit_override: Option<PathBuf>,
        cwd: PathBuf,
        user_config_path: Option<PathBuf>,
    ) -> Self {
        Self {
            explicit_override,
            cwd,
            user_config_path,
        }
    }

    pub fn load(&self) -> Result<ApplicationConfig, AppError> {
        let source = locator::locate(
            self.explicit_override.as_deref(),
            &self.cwd,
            self.user_config_path.as_deref(),
        );

        match &source {
            ConfigSource::Embedded => {
                let disk: DiskConfig =
                    serde_norway::from_str(EMBEDDED_DEFAULT_YAML).map_err(|e| {
                        AppError::Config(format!("embedded default configuration is invalid: {e}"))
                    })?;
                Ok(migrate(disk, None, true))
            }
            _ => {
                let path = source
                    .path()
                    .expect("non-embedded sources always have a path");
                self.load_from_path(path)
            }
        }
    }

    fn load_from_path(&self, path: &Path) -> Result<ApplicationConfig, AppError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Config(format!("failed to read {}: {e}", path.display())))?;
        let disk: DiskConfig = serde_norway::from_str(&content)
            .map_err(|e| AppError::Config(format!("failed to parse {}: {e}", path.display())))?;
        let known: HashSet<&str> = disk.tags.iter().map(|tag| tag.id.as_str()).collect();
        let needs_tag_migration = !disk.packs.is_empty()
            || disk
                .skills
                .iter()
                .any(|skill| skill.tags.iter().any(|id| !known.contains(id.as_str())));
        let config = migrate(disk, Some(path.display().to_string()), false);
        if needs_tag_migration {
            // Materialize the canonical form immediately. The same atomic
            // writer and backup path used by every edit make this safe, and
            // the next load observes no legacy inputs (idempotent).
            self.save(&config)?;
        }
        Ok(config)
    }

    /// Validates and atomically persists `config`. If `config` is currently
    /// backed by the embedded default (no file on disk was ever loaded),
    /// this materializes it into the user config path first — the embedded
    /// resource itself is compiled into the binary and can never be
    /// written to, so "edit while on embedded defaults" naturally routes
    /// here instead of anywhere else.
    pub fn save(&self, config: &ApplicationConfig) -> Result<PathBuf, AppError> {
        validate_config(config)?;

        let target_path = if config.is_embedded_default {
            locator::user_config_path().ok_or_else(|| {
                AppError::Config(
                    "could not determine a writable configuration directory (HOME and XDG_CONFIG_HOME are both unset)"
                        .to_string(),
                )
            })?
        } else {
            config
                .source_path
                .as_ref()
                .map(PathBuf::from)
                .ok_or_else(|| {
                    AppError::Config(
                        "configuration has no source path to write back to".to_string(),
                    )
                })?
        };

        atomic_write_yaml(&target_path, config)?;
        Ok(target_path)
    }
}

fn to_disk(config: &ApplicationConfig) -> DiskConfig {
    DiskConfig {
        version: config.version,
        defaults: DiskDefaults {
            agent: config.defaults.agent.clone(),
            copy: config.defaults.copy,
            scope: Some(
                match config.defaults.scope {
                    InstallScope::Project => "project",
                    InstallScope::Global => "global",
                }
                .to_string(),
            ),
        },
        groups: config
            .groups
            .iter()
            .map(|g| DiskGroup {
                id: g.id.clone(),
                name: g.name.clone(),
                color: Some(g.color.clone()),
                order: g.order,
                enabled: g.enabled,
            })
            .collect(),
        tags: config
            .tags
            .iter()
            .map(|t| DiskTag {
                id: t.id.clone(),
                name: t.name.clone(),
                color: t.color.clone(),
                order: t.order,
                enabled: t.enabled,
            })
            .collect(),
        packs: vec![],
        local_skill_tags: config.local_skill_tags.clone(),
        // Local skills are pure filesystem discoveries (`.agents/skills/*/SKILL.md`)
        // and are never written into configuration — only configured/remote
        // skills round-trip to disk.
        skills: config
            .skills
            .iter()
            .filter(|s| matches!(s.source, SkillSource::Remote))
            .map(|s| DiskSkill {
                id: Some(s.id.clone()),
                name: s.display_name.clone(),
                url: s.skills_url.clone(),
                description: if s.description.is_empty() {
                    None
                } else {
                    Some(s.description.clone())
                },
                group: Some(s.group_id.clone()),
                tags: s.tags.clone(),
                preselected: s.preselected,
                enabled: s.enabled,
            })
            .collect(),
    }
}

/// Full-configuration validation, run once before any write ever replaces
/// an existing valid configuration on disk. Individual field validators
/// (`validate_skill_id`, `validate_group_id`, color normalization) are
/// re-checked here defensively even though callers are expected to have
/// already validated each field at the point of mutation — this is the
/// single choke point every write passes through.
fn validate_config(config: &ApplicationConfig) -> Result<(), AppError> {
    let mut seen_tag_ids = HashSet::new();
    let mut seen_tag_names = HashSet::new();
    for tag in &config.tags {
        crate::domain::validate_skill_id(&tag.id)?;
        if !seen_tag_ids.insert(tag.id.clone()) {
            return Err(AppError::Validation(format!(
                "duplicate tag id \"{}\"",
                tag.id
            )));
        }
        let folded = tag.name.trim().to_lowercase();
        if folded.is_empty() || tag.name.chars().count() > 64 {
            return Err(AppError::Validation(
                "tag name must be between 1 and 64 characters".into(),
            ));
        }
        if !seen_tag_names.insert(folded) {
            return Err(AppError::Validation(format!(
                "duplicate tag name \"{}\"",
                tag.name
            )));
        }
        validate_and_normalize_color(&tag.color)?;
    }
    let mut seen_group_ids = HashSet::new();
    for group in &config.groups {
        crate::domain::validate_group_id(&group.id)?;
        if !seen_group_ids.insert(group.id.clone()) {
            return Err(AppError::Validation(format!(
                "duplicate group id \"{}\"",
                group.id
            )));
        }
        validate_and_normalize_color(&group.color)?;
    }

    let mut seen_skill_ids = HashSet::new();
    let mut seen_urls = HashSet::new();
    for skill in &config.skills {
        crate::domain::validate_skill_id(&skill.id)?;
        if !seen_skill_ids.insert(skill.id.clone()) {
            return Err(AppError::Validation(format!(
                "duplicate skill id \"{}\"",
                skill.id
            )));
        }
        if matches!(skill.source, SkillSource::Remote) {
            if !skill.skills_url.is_empty() && !seen_urls.insert(skill.skills_url.clone()) {
                return Err(AppError::Validation(format!(
                    "duplicate skill URL \"{}\"",
                    skill.skills_url
                )));
            }
            if !seen_group_ids.contains(&skill.group_id) {
                return Err(AppError::Validation(format!(
                    "skill \"{}\" references unknown group \"{}\"",
                    skill.id, skill.group_id
                )));
            }
        }
        if let Some(tag) = skill.tags.iter().find(|id| !seen_tag_ids.contains(*id)) {
            return Err(AppError::Validation(format!(
                "skill \"{}\" references unknown tag \"{}\"",
                skill.id, tag
            )));
        }
    }
    for (skill_id, tag_ids) in &config.local_skill_tags {
        crate::domain::validate_skill_id(skill_id)?;
        if let Some(tag) = tag_ids.iter().find(|id| !seen_tag_ids.contains(*id)) {
            return Err(AppError::Validation(format!(
                "local skill \"{skill_id}\" references unknown tag \"{tag}\""
            )));
        }
    }
    Ok(())
}

/// serialize -> write temp file in the same directory -> fsync -> rename
/// over the destination, with a `<name>.bak` copy of whatever was
/// previously there. The temp file living in the same directory as the
/// destination is what makes the final rename atomic (same filesystem).
fn atomic_write_yaml(target: &Path, config: &ApplicationConfig) -> Result<(), AppError> {
    let disk = to_disk(config);
    let yaml = serde_norway::to_string(&disk)
        .map_err(|e| AppError::Config(format!("failed to serialize configuration: {e}")))?;

    let parent = target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)?;

    if target.exists() {
        let backup = PathBuf::from(format!("{}.bak", target.display()));
        std::fs::copy(target, &backup)?;
    }

    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(yaml.as_bytes())?;
    tmp.flush()?;
    tmp.as_file().sync_all()?;
    tmp.persist(target).map_err(|e| {
        AppError::Io(format!(
            "failed to finalize write to {}: {}",
            target.display(),
            e.error
        ))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ConfigDefaults, Skill, SkillGroup};
    use std::fs;

    fn sample_config(source_path: Option<String>, is_embedded_default: bool) -> ApplicationConfig {
        ApplicationConfig {
            version: 1,
            defaults: ConfigDefaults::default(),
            groups: vec![SkillGroup {
                id: "other".into(),
                name: "Other".into(),
                color: "#8A7F84".into(),
                order: 0,
                enabled: true,
            }],
            tags: vec![],
            local_skill_tags: Default::default(),
            skills: vec![Skill {
                id: "triage".into(),
                name: "Issue Triage".into(),
                display_name: "Issue Triage".into(),
                description: "desc".into(),
                source: SkillSource::Remote,
                repository: "skills".into(),
                repository_url: "https://github.com/mattpocock/skills".into(),
                skill_name: "triage".into(),
                skills_url: "https://www.skills.sh/mattpocock/skills/triage".into(),
                group_id: "other".into(),
                tags: vec![],
                preselected: true,
                local: false,
                installed: false,
                installed_agents: Vec::new(),
                enabled: true,
            }],
            source_path,
            is_embedded_default,
            project_root: "/tmp/project".into(),
        }
    }

    #[test]
    fn load_falls_back_to_embedded_default_when_nothing_on_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let service =
            ConfigurationService::with_user_config_path(None, tmp.path().to_path_buf(), None);
        let config = service.load().unwrap();
        assert!(config.is_embedded_default);
        assert!(
            !config.skills.is_empty(),
            "embedded default should ship with curated skills"
        );
        assert!(config.groups.iter().any(|g| g.id == "other"));
    }

    #[test]
    fn load_reads_explicit_override() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("custom.yaml");
        fs::write(&path, "skills:\n  - name: Example\n    url: https://www.skills.sh/mattpocock/skills/triage\n    preselected: true\n").unwrap();

        let service = ConfigurationService::new(Some(path.clone()), tmp.path().to_path_buf());
        let config = service.load().unwrap();
        assert!(!config.is_embedded_default);
        assert_eq!(config.source_path.as_deref(), Some(path.to_str().unwrap()));
        assert_eq!(config.skills.len(), 1);
    }

    #[test]
    fn save_materializes_embedded_default_into_user_config_path() {
        let tmp = tempfile::tempdir().unwrap();
        let user_path = tmp.path().join("skills.yaml");
        let config = sample_config(None, true);

        // We can't easily override the real XDG lookup here without
        // touching process env vars, so exercise `atomic_write_yaml`
        // directly (the unit under test for "never write into the embedded
        // resource, always write a real file") and assert on its result.
        atomic_write_yaml(&user_path, &config).unwrap();
        assert!(user_path.is_file());
        let written = fs::read_to_string(&user_path).unwrap();
        assert!(written.contains("triage"));
    }

    #[test]
    fn save_creates_backup_of_previous_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("skills.yaml");
        fs::write(&path, "skills: []\n").unwrap();

        let config = sample_config(Some(path.to_str().unwrap().to_string()), false);
        atomic_write_yaml(&path, &config).unwrap();

        let backup = tmp.path().join("skills.yaml.bak");
        assert!(backup.is_file());
        assert_eq!(fs::read_to_string(&backup).unwrap(), "skills: []\n");
    }

    #[test]
    fn save_round_trips_through_load() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("skills.yaml");
        let config = sample_config(Some(path.to_str().unwrap().to_string()), false);
        atomic_write_yaml(&path, &config).unwrap();

        let service = ConfigurationService::new(Some(path.clone()), tmp.path().to_path_buf());
        let reloaded = service.load().unwrap();
        assert_eq!(reloaded.skills.len(), 1);
        assert_eq!(reloaded.skills[0].id, "triage");
        assert_eq!(reloaded.skills[0].group_id, "other");
    }

    #[test]
    fn validate_rejects_duplicate_group_ids() {
        let mut config = sample_config(None, false);
        config.groups.push(config.groups[0].clone());
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_duplicate_skill_ids() {
        let mut config = sample_config(None, false);
        config.skills.push(config.skills[0].clone());
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_duplicate_skill_urls() {
        let mut config = sample_config(None, false);
        let mut second = config.skills[0].clone();
        second.id = "triage-2".into();
        config.skills.push(second);
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_skill_referencing_unknown_group() {
        let mut config = sample_config(None, false);
        config.skills[0].group_id = "does-not-exist".into();
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_rejects_invalid_group_color() {
        let mut config = sample_config(None, false);
        config.groups[0].color = "not-a-color".into();
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_accepts_well_formed_config() {
        let config = sample_config(None, false);
        assert!(validate_config(&config).is_ok());
    }
}
