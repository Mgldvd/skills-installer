use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::config::migrate::dedupe_id;
use crate::domain::Project;
use crate::error::AppError;
use crate::platform::app_config_dir;

/// Persists the user's saved Projects (see `domain::Project`) — own JSON
/// file at `$XDG_CONFIG_HOME/skills-installer/projects.json`, entirely
/// separate from any one project's `skills.yaml`: the whole point is that a
/// Project survives independently of the directory it was saved from.
#[derive(Clone)]
pub struct ProjectsService {
    path_override: Option<PathBuf>,
}

impl ProjectsService {
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
        app_config_dir().map(|dir| dir.join("projects.json")).ok_or_else(|| {
            AppError::Validation(
                "could not determine a writable configuration directory (HOME and XDG_CONFIG_HOME are both unset)"
                    .to_string(),
            )
        })
    }

    /// Missing file is not an error — nothing has been saved yet.
    pub fn list(&self) -> Result<Vec<Project>, AppError> {
        let path = self.path()?;
        if !path.is_file() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| AppError::Io(format!("failed to read {}: {e}", path.display())))?;
        let mut projects: Vec<Project> = serde_json::from_str(&content).map_err(|e| {
            AppError::Validation(format!("failed to parse {}: {e}", path.display()))
        })?;
        projects.sort_by_key(|p| p.name.to_lowercase());
        Ok(projects)
    }

    fn write_all(&self, projects: &[Project]) -> Result<(), AppError> {
        let path = self.path()?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent)?;

        let json = serde_json::to_string_pretty(projects)
            .map_err(|e| AppError::Validation(format!("failed to serialize projects: {e}")))?;

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
    /// `dedupe_id`) — the visible `name` can repeat across Projects, only
    /// the internal `id` needs to stay unique.
    pub fn save(
        &self,
        name: String,
        git_url: Option<String>,
        skill_names: Vec<String>,
    ) -> Result<Project, AppError> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::Validation("Project name cannot be empty".into()));
        }
        if skill_names.is_empty() {
            return Err(AppError::Validation(
                "select at least one installed skill to save as a Project".into(),
            ));
        }
        let git_url = git_url
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty());

        let mut projects = self.list()?;
        let mut used: HashSet<String> = projects.iter().map(|p| p.id.clone()).collect();
        let id = dedupe_id(&name, &mut used);
        let project = Project {
            id,
            name,
            git_url,
            skill_names,
        };
        projects.push(project.clone());
        self.write_all(&projects)?;
        Ok(project)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut projects = self.list()?;
        let before = projects.len();
        projects.retain(|p| p.id != id);
        if projects.len() == before {
            return Err(AppError::NotFound(format!("project \"{id}\" not found")));
        }
        self.write_all(&projects)
    }
}

impl Default for ProjectsService {
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
        let service = ProjectsService::with_path(tmp.path().join("projects.json"));
        assert_eq!(service.list().unwrap(), Vec::new());
    }

    #[test]
    fn save_then_list_round_trips_and_sorts_by_name() {
        let tmp = tempfile::tempdir().unwrap();
        let service = ProjectsService::with_path(tmp.path().join("projects.json"));

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

        let projects = service.list().unwrap();
        assert_eq!(projects.len(), 2);
        assert_eq!(projects[0].name, "Atlas");
        assert_eq!(
            projects[0].git_url.as_deref(),
            Some("https://github.com/me/atlas")
        );
        assert_eq!(projects[0].skill_names, vec!["triage", "docs"]);
        assert_eq!(projects[1].name, "Zephyr");
        assert_eq!(projects[1].git_url, None);
    }

    #[test]
    fn save_dedupes_the_id_but_keeps_the_display_name_as_typed() {
        let tmp = tempfile::tempdir().unwrap();
        let service = ProjectsService::with_path(tmp.path().join("projects.json"));

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
        let service = ProjectsService::with_path(tmp.path().join("projects.json"));

        let project = service
            .save("  Spaced  ".into(), Some("   ".into()), vec!["a".into()])
            .unwrap();

        assert_eq!(project.name, "Spaced");
        assert_eq!(project.git_url, None);
    }

    #[test]
    fn save_rejects_an_empty_name() {
        let tmp = tempfile::tempdir().unwrap();
        let service = ProjectsService::with_path(tmp.path().join("projects.json"));
        assert!(service.save("   ".into(), None, vec!["a".into()]).is_err());
    }

    #[test]
    fn save_rejects_an_empty_skill_list() {
        let tmp = tempfile::tempdir().unwrap();
        let service = ProjectsService::with_path(tmp.path().join("projects.json"));
        assert!(service.save("Empty".into(), None, vec![]).is_err());
    }

    #[test]
    fn delete_removes_exactly_one_project() {
        let tmp = tempfile::tempdir().unwrap();
        let service = ProjectsService::with_path(tmp.path().join("projects.json"));
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
        let service = ProjectsService::with_path(tmp.path().join("projects.json"));
        assert!(service.delete("nope").is_err());
    }
}
