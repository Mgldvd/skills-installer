use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::config::migrate::dedupe_id;
use crate::domain::Preset;
use crate::error::AppError;
use crate::platform::app_config_dir;

/// Persists the user's saved Presets (see `domain::Preset`) — own JSON file
/// at `$XDG_CONFIG_HOME/skills-installer/presets.json`, entirely separate
/// from any one project's `skills.yaml`: the whole point is that a Preset
/// survives independently of the directory it was saved from.
///
/// This service, its file, and `Preset` itself were all renamed from
/// "Project(s)" — `list()` migrates an older install's `projects.json` in
/// place the first time it's read.
#[derive(Clone)]
pub struct PresetsService {
    path_override: Option<PathBuf>,
}

impl PresetsService {
    pub fn new() -> Self {
        Self {
            path_override: None,
        }
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self {
            path_override: Some(path),
        }
    }

    fn path(&self) -> Result<PathBuf, AppError> {
        if let Some(path) = &self.path_override {
            return Ok(path.clone());
        }
        app_config_dir().map(|dir| dir.join("presets.json")).ok_or_else(|| {
            AppError::Validation(
                "could not determine a writable configuration directory (HOME and XDG_CONFIG_HOME are both unset)"
                    .to_string(),
            )
        })
    }

    fn read_presets_file(path: &Path) -> Result<Vec<Preset>, AppError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| AppError::Io(format!("failed to read {}: {e}", path.display())))?;
        let mut presets: Vec<Preset> = serde_json::from_str(&content).map_err(|e| {
            AppError::Validation(format!("failed to parse {}: {e}", path.display()))
        })?;
        presets.sort_by_key(|p| p.name.to_lowercase());
        Ok(presets)
    }

    /// Missing file is not an error — nothing has been saved yet, unless an
    /// older install's `projects.json` sits next to where `presets.json`
    /// would go, in which case it's read once and immediately persisted
    /// under the new name so every subsequent call uses `presets.json`.
    pub fn list(&self) -> Result<Vec<Preset>, AppError> {
        let path = self.path()?;
        if path.is_file() {
            return Self::read_presets_file(&path);
        }
        let legacy_path = path.with_file_name("projects.json");
        if legacy_path.is_file() {
            let presets = Self::read_presets_file(&legacy_path)?;
            self.write_all(&presets)?;
            return Ok(presets);
        }
        Ok(Vec::new())
    }

    fn write_all(&self, presets: &[Preset]) -> Result<(), AppError> {
        let path = self.path()?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;

        let json = serde_json::to_string_pretty(presets)
            .map_err(|e| AppError::Validation(format!("failed to serialize presets: {e}")))?;

        let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
        tmp.write_all(json.as_bytes())?;
        tmp.flush()?;
        tmp.as_file().sync_all()?;
        tmp.persist(&path).map_err(|e| {
            AppError::Io(format!(
                "failed to finalize write to {}: {}",
                path.display(),
                e.error
            ))
        })?;

        Ok(())
    }

    /// Names are deduplicated the same way Packs/Skills are (see
    /// `dedupe_id`) — the visible `name` can repeat across Presets, only
    /// the internal `id` needs to stay unique.
    pub fn save(
        &self,
        name: String,
        git_url: Option<String>,
        skill_names: Vec<String>,
    ) -> Result<Preset, AppError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation("Preset name cannot be empty".into()));
        }
        if skill_names.is_empty() {
            return Err(AppError::Validation(
                "select at least one installed skill to save as a Preset".into(),
            ));
        }
        let git_url = git_url
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty());

        let mut presets = self.list()?;
        let mut used: HashSet<String> = presets.iter().map(|p| p.id.clone()).collect();
        let id = dedupe_id(&name, &mut used);
        let preset = Preset {
            id,
            name,
            git_url,
            skill_names,
        };
        presets.push(preset.clone());
        self.write_all(&presets)?;
        Ok(preset)
    }

    /// Replaces a Preset's Skill list in place — keeps its `id`/`name`/
    /// `git_url`, only `skill_names` changes — so re-syncing after
    /// installing/removing Skills doesn't require deleting and recreating it.
    pub fn update(&self, id: &str, skill_names: Vec<String>) -> Result<Preset, AppError> {
        if skill_names.is_empty() {
            return Err(AppError::Validation(
                "select at least one installed skill to update the Preset with".into(),
            ));
        }
        let mut presets = self.list()?;
        let preset = presets
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| AppError::NotFound(format!("preset \"{id}\" not found")))?;
        preset.skill_names = skill_names;
        let updated = preset.clone();
        self.write_all(&presets)?;
        Ok(updated)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut presets = self.list()?;
        let before = presets.len();
        presets.retain(|p| p.id != id);
        if presets.len() == before {
            return Err(AppError::NotFound(format!("preset \"{id}\" not found")));
        }
        self.write_all(&presets)
    }
}

impl Default for PresetsService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_returns_empty_when_no_file_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));
        assert_eq!(service.list().unwrap(), Vec::new());
    }

    #[test]
    fn save_then_list_round_trips_and_sorts_by_name() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));

        service
            .save("Zephyr".into(), None, vec!["triage".into()])
            .unwrap();
        service
            .save(
                "Atlas".into(),
                Some("https://github.com/me/atlas".into()),
                vec!["triage".into(), "docs".into()],
            )
            .unwrap();

        let presets = service.list().unwrap();
        assert_eq!(presets.len(), 2);
        assert_eq!(presets[0].name, "Atlas");
        assert_eq!(
            presets[0].git_url.as_deref(),
            Some("https://github.com/me/atlas")
        );
        assert_eq!(presets[0].skill_names, vec!["triage", "docs"]);
        assert_eq!(presets[1].name, "Zephyr");
        assert_eq!(presets[1].git_url, None);
    }

    #[test]
    fn save_dedupes_the_id_but_keeps_the_display_name_as_typed() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));

        let first = service
            .save("My App".into(), None, vec!["a".into()])
            .unwrap();
        let second = service
            .save("My App".into(), None, vec!["b".into()])
            .unwrap();

        assert_eq!(first.id, "my-app");
        assert_eq!(second.id, "my-app-2");
        assert_eq!(second.name, "My App");
    }

    #[test]
    fn save_trims_whitespace_and_drops_an_empty_git_url() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));

        let preset = service
            .save("  Spaced  ".into(), Some("   ".into()), vec!["a".into()])
            .unwrap();

        assert_eq!(preset.name, "Spaced");
        assert_eq!(preset.git_url, None);
    }

    #[test]
    fn save_rejects_an_empty_name() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));
        assert!(service.save("   ".into(), None, vec!["a".into()]).is_err());
    }

    #[test]
    fn save_rejects_an_empty_skill_list() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));
        assert!(service.save("Empty".into(), None, vec![]).is_err());
    }

    #[test]
    fn update_replaces_skill_names_but_keeps_id_and_name() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));
        let saved = service
            .save(
                "Atlas".into(),
                Some("https://github.com/me/atlas".into()),
                vec!["triage".into()],
            )
            .unwrap();

        let updated = service
            .update(&saved.id, vec!["triage".into(), "docs".into()])
            .unwrap();

        assert_eq!(updated.id, saved.id);
        assert_eq!(updated.name, "Atlas");
        assert_eq!(
            updated.git_url.as_deref(),
            Some("https://github.com/me/atlas")
        );
        assert_eq!(updated.skill_names, vec!["triage", "docs"]);

        let reloaded = service.list().unwrap();
        assert_eq!(reloaded[0].skill_names, vec!["triage", "docs"]);
    }

    #[test]
    fn update_rejects_an_empty_skill_list() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));
        let saved = service.save("A".into(), None, vec!["x".into()]).unwrap();
        assert!(service.update(&saved.id, vec![]).is_err());
    }

    #[test]
    fn update_errors_on_an_unknown_id() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));
        assert!(service.update("nope", vec!["x".into()]).is_err());
    }

    #[test]
    fn delete_removes_exactly_one_preset() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));
        let a = service.save("A".into(), None, vec!["x".into()]).unwrap();
        service.save("B".into(), None, vec!["y".into()]).unwrap();

        service.delete(&a.id).unwrap();

        let remaining = service.list().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].name, "B");
    }

    #[test]
    fn delete_errors_on_an_unknown_id() {
        let tmp = tempfile::tempdir().unwrap();
        let service = PresetsService::with_path(tmp.path().join("presets.json"));
        assert!(service.delete("nope").is_err());
    }

    // Migration from the old "Project" naming: an existing projects.json
    // next to where presets.json would live must still be readable, and
    // gets copied over to presets.json in place so later writes land in
    // the new file.
    #[test]
    fn list_migrates_a_legacy_projects_json_file_in_place() {
        let tmp = tempfile::tempdir().unwrap();
        let presets_path = tmp.path().join("presets.json");
        let legacy_path = tmp.path().join("projects.json");
        std::fs::write(
            &legacy_path,
            r#"[{"id":"atlas","name":"Atlas","gitUrl":null,"skillNames":["triage"]}]"#,
        )
        .unwrap();
        let service = PresetsService::with_path(presets_path.clone());

        let presets = service.list().unwrap();

        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].name, "Atlas");
        assert!(
            presets_path.is_file(),
            "migrated content must be persisted to presets.json"
        );

        // A later save operates on the new file, not the legacy one.
        service
            .save("Zephyr".into(), None, vec!["docs".into()])
            .unwrap();
        let legacy_content = std::fs::read_to_string(&legacy_path).unwrap();
        assert!(
            !legacy_content.contains("Zephyr"),
            "the legacy file must not be rewritten after migration"
        );
        let current = service.list().unwrap();
        assert_eq!(current.len(), 2);
    }
}
