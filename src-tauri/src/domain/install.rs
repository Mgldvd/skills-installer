use serde::{Deserialize, Serialize};

use super::skill::SkillSelection;

/// Project vs. global mirrors the real Skills CLI's `-g/--global` flag
/// (absence of the flag means project-scoped) — see `installer::skills_cli`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallScope {
    Project,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOptions {
    #[serde(default)]
    pub agents: Vec<String>,
    /// Explicit project working directory selected in the GUI. Global
    /// installs ignore this value.
    #[serde(default)]
    pub project_path: Option<String>,
    pub copy: bool,
    pub scope: InstallScope,
    pub dry_run: bool,
    pub confirm: bool,
    pub continue_on_error: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            agents: vec!["universal".to_string()],
            project_path: None,
            copy: true,
            scope: InstallScope::Project,
            dry_run: false,
            confirm: true,
            continue_on_error: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallRequest {
    pub selection: SkillSelection,
    pub options: InstallOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillInstallStatus {
    Installed,
    AlreadyInstalled,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInstallOutcome {
    pub skill_id: String,
    pub display_name: String,
    pub status: SkillInstallStatus,
    pub message: Option<String>,
    pub command_preview: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallResult {
    pub requested: usize,
    pub installed: usize,
    pub already_installed: usize,
    pub failed: usize,
    pub cancelled: bool,
    pub per_skill: Vec<SkillInstallOutcome>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// Streamed to the frontend through a `tauri::ipc::Channel<InstallProgressEvent>`
/// scoped to a single `install_skills` invocation (see `commands::installation`).
/// Variant names deliberately mirror the `install:start` / `install:progress` /
/// ... naming the product spec calls for; the discriminated-union shape is what
/// the current Tauri v2 IPC guidance recommends for high-frequency streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum InstallProgressEvent {
    Start {
        total: usize,
    },
    Progress {
        current: usize,
        total: usize,
        skill_id: String,
        display_name: String,
    },
    Output {
        skill_id: String,
        line: String,
        stream: OutputStream,
    },
    SkillSuccess {
        skill_id: String,
        display_name: String,
    },
    SkillError {
        skill_id: String,
        display_name: String,
        message: String,
    },
    Complete {
        result: InstallResult,
    },
}
