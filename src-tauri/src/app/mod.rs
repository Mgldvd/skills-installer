pub mod installation_service;
pub mod skills_service;

pub use installation_service::InstallationService;
pub use skills_service::{
    DeleteGroupStrategy, GroupUpdateInput, NewGroupInput, NewSkillInput, PackImportResult,
    SkillUpdateInput, SkillsService,
};

use std::path::PathBuf;
use std::sync::Arc;

use crate::config::ConfigurationService;
use crate::installer::{Installer, SkillsCliInstaller};
use crate::preferences::PreferencesService;
use crate::presets::PresetsService;
use crate::process::{ProcessRunner, TokioProcessRunner};

/// The composition root: everything the Tauri command layer and the CLI
/// both depend on, wired exactly once, with exactly one real `Installer`
/// implementation. Neither the GUI nor the CLI constructs its own services —
/// they both just hold an `ApplicationServices`.
pub struct ApplicationServices {
    pub skills: SkillsService,
    pub preferences: PreferencesService,
    pub presets: PresetsService,
    pub installation: InstallationService,
}

impl ApplicationServices {
    pub fn new(config_override: Option<PathBuf>, project_root: PathBuf) -> Self {
        let config_service = ConfigurationService::new(config_override, project_root.clone());
        let preferences = PreferencesService::new();
        let skills = SkillsService::new(config_service, project_root.clone(), preferences.clone());
        let presets = PresetsService::new();

        let process_runner: Arc<dyn ProcessRunner> = Arc::new(TokioProcessRunner);
        let installer: Arc<dyn Installer> =
            Arc::new(SkillsCliInstaller::new(process_runner, project_root));
        let installation = InstallationService::new(installer);

        Self {
            skills,
            preferences,
            presets,
            installation,
        }
    }
}
