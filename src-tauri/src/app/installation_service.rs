use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use crate::domain::{DependencyStatus, InstallRequest, InstallResult, Skill};
use crate::error::AppError;
use crate::installer::{InstallBatch, Installer, ProgressSender};

/// Resolves an IPC/CLI-facing `InstallRequest` (skill ids) against an
/// already-loaded skill list, drives the shared `Installer`, and owns the
/// single in-flight cancellation token so a separate `cancel_installation`
/// call can reach the same installation. GUI and CLI both go through this —
/// neither talks to `Installer` directly.
pub struct InstallationService {
    installer: Arc<dyn Installer>,
    active_cancel: Mutex<Option<CancellationToken>>,
}

impl InstallationService {
    pub fn new(installer: Arc<dyn Installer>) -> Self {
        Self {
            installer,
            active_cancel: Mutex::new(None),
        }
    }

    pub async fn check_dependencies(&self) -> Result<DependencyStatus, AppError> {
        self.installer.check_dependencies().await
    }

    /// Resolves `request.selection` against `available_skills`, rejecting
    /// unknown/disabled ids before anything is spawned — this is the
    /// "validate dependencies and selection before confirming" step the
    /// GUI's confirmation dialog relies on.
    pub fn validate_installation(
        &self,
        request: &InstallRequest,
        available_skills: &[Skill],
    ) -> Result<(), AppError> {
        if request.selection.is_empty() {
            return Err(AppError::Validation(
                "no skills selected for installation".to_string(),
            ));
        }
        if request.options.agents.is_empty() {
            return Err(AppError::Validation(
                "select at least one installation agent".into(),
            ));
        }
        for agent in &request.options.agents {
            crate::domain::validate_agent_id(agent)?;
            if !crate::domain::is_supported_agent(agent) {
                return Err(AppError::Validation(format!(
                    "unsupported Skills CLI agent \"{agent}\""
                )));
            }
        }
        if request.options.scope == crate::domain::InstallScope::Project {
            if let Some(path) = request.options.project_path.as_deref() {
                if !std::path::Path::new(path).is_dir() {
                    return Err(AppError::Validation(format!(
                        "installation destination is not an available directory: {path}"
                    )));
                }
            }
        }
        for skill_id in &request.selection.skill_ids {
            let skill = available_skills
                .iter()
                .find(|s| &s.id == skill_id)
                .ok_or_else(|| AppError::Validation(format!("unknown skill id \"{skill_id}\"")))?;
            if !skill.enabled {
                return Err(AppError::Validation(format!(
                    "skill \"{}\" is disabled and cannot be installed",
                    skill.display_name
                )));
            }
        }
        Ok(())
    }

    pub async fn install(
        &self,
        request: InstallRequest,
        available_skills: &[Skill],
        progress: ProgressSender,
    ) -> Result<InstallResult, AppError> {
        self.validate_installation(&request, available_skills)?;

        let skills: Vec<Skill> = request
            .selection
            .skill_ids
            .iter()
            .filter_map(|id| available_skills.iter().find(|s| &s.id == id).cloned())
            .collect();

        let cancel = CancellationToken::new();
        *self.active_cancel.lock().unwrap() = Some(cancel.clone());

        let batch = InstallBatch {
            skills,
            options: request.options,
        };
        let result = self.installer.install(batch, progress, cancel).await;

        *self.active_cancel.lock().unwrap() = None;
        result
    }

    /// Idempotent: cancelling with nothing in flight is a no-op, not an
    /// error, since the GUI may race a "Cancel" click against completion.
    pub fn cancel(&self) {
        if let Some(token) = self.active_cancel.lock().unwrap().as_ref() {
            token.cancel();
        }
    }

    pub fn is_installing(&self) -> bool {
        self.active_cancel.lock().unwrap().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        DependencySource, InstallOptions, InstallScope, SkillInstallStatus, SkillSelection,
        SkillSource, OTHER_GROUP_ID,
    };
    use crate::installer::FakeInstaller;

    fn sample_skill(id: &str, enabled: bool) -> Skill {
        Skill {
            id: id.to_string(),
            name: id.to_string(),
            display_name: id.to_string(),
            description: String::new(),
            source: SkillSource::Remote,
            repository: "skills".into(),
            repository_url: "https://github.com/mattpocock/skills".into(),
            skill_name: id.to_string(),
            skills_url: format!("https://www.skills.sh/mattpocock/skills/{id}"),
            group_id: OTHER_GROUP_ID.to_string(),
            tags: vec![],
            preselected: false,
            local: false,
            installed: false,
            installed_agents: Vec::new(),
            enabled,
        }
    }

    fn fake_dependency_status() -> DependencyStatus {
        DependencyStatus {
            available: true,
            source: DependencySource::InstalledExecutable,
            executable_path: Some("/usr/bin/skills".into()),
            version: Some("1.0.0".into()),
            detail: None,
        }
    }

    #[test]
    fn validate_rejects_empty_selection() {
        let service =
            InstallationService::new(Arc::new(FakeInstaller::new(fake_dependency_status())));
        let request = InstallRequest {
            selection: SkillSelection::default(),
            options: InstallOptions::default(),
        };
        assert!(service.validate_installation(&request, &[]).is_err());
    }

    #[test]
    fn validate_rejects_unknown_skill_id() {
        let service =
            InstallationService::new(Arc::new(FakeInstaller::new(fake_dependency_status())));
        let request = InstallRequest {
            selection: SkillSelection::new(vec!["ghost".to_string()]),
            options: InstallOptions::default(),
        };
        assert!(service
            .validate_installation(&request, &[sample_skill("triage", true)])
            .is_err());
    }

    #[test]
    fn validate_rejects_disabled_skill() {
        let service =
            InstallationService::new(Arc::new(FakeInstaller::new(fake_dependency_status())));
        let request = InstallRequest {
            selection: SkillSelection::new(vec!["triage".to_string()]),
            options: InstallOptions::default(),
        };
        assert!(service
            .validate_installation(&request, &[sample_skill("triage", false)])
            .is_err());
    }

    #[tokio::test]
    async fn install_resolves_ids_and_delegates_to_installer() {
        let fake = Arc::new(FakeInstaller::new(fake_dependency_status()));
        let service = InstallationService::new(fake.clone());
        let skills = vec![sample_skill("triage", true), sample_skill("tdd", true)];
        let request = InstallRequest {
            selection: SkillSelection::new(vec!["triage".to_string()]),
            options: InstallOptions {
                scope: InstallScope::Project,
                ..InstallOptions::default()
            },
        };

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let result = service.install(request, &skills, tx).await.unwrap();
        drop(rx);

        assert_eq!(result.requested, 1);
        let calls = fake.install_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].skills.len(), 1);
        assert_eq!(calls[0].skills[0].id, "triage");
    }

    #[tokio::test]
    async fn is_installing_is_false_after_completion() {
        let fake = Arc::new(FakeInstaller::new(fake_dependency_status()));
        let service = InstallationService::new(fake);
        let skills = vec![sample_skill("triage", true)];
        let request = InstallRequest {
            selection: SkillSelection::new(vec!["triage".to_string()]),
            options: InstallOptions::default(),
        };

        assert!(!service.is_installing());
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        service.install(request, &skills, tx).await.unwrap();
        drop(rx);
        assert!(!service.is_installing());
    }

    #[test]
    fn cancel_without_active_installation_is_a_no_op() {
        let service =
            InstallationService::new(Arc::new(FakeInstaller::new(fake_dependency_status())));
        service.cancel();
    }

    #[tokio::test]
    async fn install_result_reflects_fake_installer_status_field() {
        let fake = Arc::new(
            FakeInstaller::new(fake_dependency_status()).with_install_result(Ok(
                crate::domain::InstallResult {
                    requested: 1,
                    installed: 1,
                    per_skill: vec![crate::domain::SkillInstallOutcome {
                        skill_id: "triage".into(),
                        display_name: "Issue Triage".into(),
                        status: SkillInstallStatus::Installed,
                        message: None,
                        command_preview: "skills add mattpocock/skills --skill triage".into(),
                    }],
                    ..Default::default()
                },
            )),
        );
        let service = InstallationService::new(fake);
        let skills = vec![sample_skill("triage", true)];
        let request = InstallRequest {
            selection: SkillSelection::new(vec!["triage".to_string()]),
            options: InstallOptions::default(),
        };

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let result = service.install(request, &skills, tx).await.unwrap();
        drop(rx);
        assert_eq!(result.installed, 1);
        assert_eq!(result.per_skill[0].status, SkillInstallStatus::Installed);
    }
}
