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
///
/// `rename_all` only covers the `event` tag itself ("skill-error", ...) —
/// without `rename_all_fields` too, each variant's own fields (`skill_id`,
/// `display_name`, ...) serialize under their literal snake_case Rust names,
/// which the frontend's `event.data.skillId`/`.displayName` never match, so
/// every live per-skill row silently read as `undefined` (confirmed against
/// the real serialized JSON). `SkillInstallOutcome`/`InstallResult` never hit
/// this because they're separate structs with their own `rename_all`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "event",
    content = "data",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
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
    Command {
        skill_id: String,
        command: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    // Regression coverage for a real bug: `rename_all = "kebab-case"` on
    // this enum only ever covered the `event` tag itself. Every variant's
    // own fields kept serializing under their literal snake_case Rust
    // names, so the frontend's `event.data.skillId`/`.displayName` reads
    // were always `undefined` — the live per-skill row in the install
    // progress panel showed "undefined" for every install, every skill,
    // silently, because the final summary numbers (a separate struct with
    // its own `rename_all`) still happened to be correct and masked it.

    #[test]
    fn progress_event_fields_serialize_as_camel_case() {
        let event = InstallProgressEvent::Progress {
            current: 1,
            total: 2,
            skill_id: "triage".into(),
            display_name: "Issue Triage".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event"], "progress");
        assert_eq!(json["data"]["skillId"], "triage");
        assert_eq!(json["data"]["displayName"], "Issue Triage");
    }

    #[test]
    fn skill_error_event_fields_serialize_as_camel_case() {
        let event = InstallProgressEvent::SkillError {
            skill_id: "triage".into(),
            display_name: "Issue Triage".into(),
            message: "boom".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event"], "skill-error");
        assert_eq!(json["data"]["skillId"], "triage");
        assert_eq!(json["data"]["displayName"], "Issue Triage");
        assert_eq!(json["data"]["message"], "boom");
    }

    #[test]
    fn skill_success_event_fields_serialize_as_camel_case() {
        let event = InstallProgressEvent::SkillSuccess {
            skill_id: "triage".into(),
            display_name: "Issue Triage".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event"], "skill-success");
        assert_eq!(json["data"]["skillId"], "triage");
        assert_eq!(json["data"]["displayName"], "Issue Triage");
    }

    #[test]
    fn output_event_fields_serialize_as_camel_case() {
        let event = InstallProgressEvent::Output {
            skill_id: "triage".into(),
            line: "installing...".into(),
            stream: OutputStream::Stdout,
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event"], "output");
        assert_eq!(json["data"]["skillId"], "triage");
        assert_eq!(json["data"]["line"], "installing...");
    }

    #[test]
    fn command_event_fields_serialize_as_camel_case() {
        let event = InstallProgressEvent::Command {
            skill_id: "triage".into(),
            command: "skills add ...".into(),
        };
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event"], "command");
        assert_eq!(json["data"]["skillId"], "triage");
    }
}
