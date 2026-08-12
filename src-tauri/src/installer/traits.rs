use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use crate::domain::{
    DependencyStatus, InstallOptions, InstallProgressEvent, InstallResult, InstalledSkill, Skill,
};
use crate::error::AppError;

pub type ProgressSender = UnboundedSender<InstallProgressEvent>;

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub agents: Vec<String>,
    pub global_only: bool,
}

/// What the `Installer` trait needs to install a batch of skills: fully
/// resolved `Skill` objects, not bare ids. Deliberately distinct from
/// `domain::InstallRequest` (the IPC/CLI-facing "install these ids" shape) —
/// id resolution against configuration state happens once, in
/// `app::InstallationService`, so `Installer` implementations never need to
/// reach back into configuration/skills state themselves.
#[derive(Debug, Clone)]
pub struct InstallBatch {
    pub skills: Vec<Skill>,
    pub options: InstallOptions,
}

#[derive(Debug, Clone)]
pub struct RemoveRequest {
    pub skill: Skill,
    pub options: InstallOptions,
}

#[derive(Debug, Clone)]
pub struct UpdateRequest {
    pub options: InstallOptions,
}

/// Isolates all skill-installation-provider behavior behind one boundary.
/// The real implementation (`SkillsCliInstaller`) shells out to the Skills
/// CLI; tests use `FakeInstaller` so the full application service stack can
/// be exercised without a real network call or child process.
#[async_trait]
pub trait Installer: Send + Sync {
    async fn check_dependencies(&self) -> Result<DependencyStatus, AppError>;

    async fn list_installed(&self, options: ListOptions) -> Result<Vec<InstalledSkill>, AppError>;

    async fn install(
        &self,
        batch: InstallBatch,
        progress: ProgressSender,
        cancel: CancellationToken,
    ) -> Result<InstallResult, AppError>;

    async fn remove(&self, request: RemoveRequest) -> Result<(), AppError>;

    async fn update(&self, request: UpdateRequest) -> Result<(), AppError>;
}
