use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::domain::SkillSource;
use crate::domain::{
    DependencyStatus, InstallProgressEvent, InstallResult, InstalledSkill, SkillInstallOutcome,
    SkillInstallStatus, UninstallResult,
};
use crate::error::AppError;
use crate::process::{preview_command, ProcessOutcome, ProcessRunner, ProcessSpec};

use super::dependency::DependencyResolver;
use super::traits::{
    InstallBatch, Installer, ListOptions, ProgressSender, RemoveRequest, UpdateRequest,
};

/// Outcome of running one `skills add` subprocess for one (skill, scope
/// group) pair — see `SkillsCliInstaller::run_one_group`.
enum GroupOutcome {
    Success,
    Failed(String),
    Cancelled,
}

/// The one real `Installer`. Talks to the actual `skills` CLI (see
/// `installer::dependency` for how it's located) using the flag mapping
/// confirmed against the real vercel-labs/skills tool:
/// `skills add <owner>/<repository> --skill <name> [--agent <a>] [--global] [--copy] --yes`.
pub struct SkillsCliInstaller {
    process_runner: Arc<dyn ProcessRunner>,
    resolver: DependencyResolver,
    project_root: PathBuf,
}

impl SkillsCliInstaller {
    pub fn new(process_runner: Arc<dyn ProcessRunner>, project_root: PathBuf) -> Self {
        Self {
            process_runner,
            resolver: DependencyResolver::new(),
            project_root,
        }
    }

    #[cfg(test)]
    pub fn with_resolver(
        process_runner: Arc<dyn ProcessRunner>,
        project_root: PathBuf,
        resolver: DependencyResolver,
    ) -> Self {
        Self {
            process_runner,
            resolver,
            project_root,
        }
    }

    /// Builds one complete `skills add` argument list per distinct scope
    /// present among `options.agents` — almost always just one, but two
    /// when the caller mixes Project- and Global-scoped agents in the same
    /// install, since the real CLI's `--global` flag applies to its whole
    /// invocation and can't be split per `--agent`. Deterministic order:
    /// Project group (if any) first, then Global.
    fn add_args_for_skill(
        skill: &crate::domain::Skill,
        options: &crate::domain::InstallOptions,
    ) -> Result<Vec<Vec<String>>, AppError> {
        let source = match &skill.source {
            SkillSource::Remote if skill.repository_url.starts_with("https://skills.sh/p/") => {
                skill.repository_url.clone()
            }
            SkillSource::Remote => format!(
                "{}/{}",
                owner_from_repository_url(&skill.repository_url)?,
                skill.repository
            ),
            SkillSource::Local { path } => std::path::Path::new(path)
                .parent()
                .ok_or_else(|| {
                    AppError::Installation(format!("local Skill source has no parent: {path}"))
                })?
                .display()
                .to_string(),
        };
        let base_args = vec![
            "add".to_string(),
            source,
            "--skill".to_string(),
            skill.skill_name.clone(),
        ];

        // Translate our own ids (e.g. `vscode`) into whatever the real CLI
        // recognizes (see `cli_agent_id`), deduping per group in case that
        // collapses two selected rows onto the same underlying agent. An
        // agent selected for *both* Project and Global lands in both lists
        // — that's exactly what makes it install to both.
        let mut project_agents: Vec<String> = Vec::new();
        let mut global_agents: Vec<String> = Vec::new();
        for agent in &options.agents {
            let cli_id = crate::domain::cli_agent_id(agent).to_string();
            let selection = options.scopes_for(agent);
            if selection.project && !project_agents.contains(&cli_id) {
                project_agents.push(cli_id.clone());
            }
            if selection.global && !global_agents.contains(&cli_id) {
                global_agents.push(cli_id);
            }
        }

        let mut groups: Vec<(Vec<String>, bool)> = Vec::new();
        if options.agents.is_empty() {
            groups.push((Vec::new(), false));
        } else {
            if !project_agents.is_empty() {
                groups.push((project_agents, false));
            }
            if !global_agents.is_empty() {
                groups.push((global_agents, true));
            }
        }

        Ok(groups
            .into_iter()
            .map(|(agents, is_global)| {
                let mut args = base_args.clone();
                if !agents.is_empty() {
                    args.push("--agent".into());
                    args.extend(agents);
                }
                if is_global {
                    args.push("--global".into());
                }
                if options.copy {
                    args.push("--copy".into());
                }
                args.push("--yes".into());
                args
            })
            .collect())
    }

    /// Runs one already-built argument list as a `skills` subprocess,
    /// streaming its output through `progress` tagged with `skill.id`
    /// exactly like a single-group install always did — a second group for
    /// the same skill just means this gets called again, and its output
    /// interleaves under the same skill id in the progress panel.
    async fn run_one_group(
        &self,
        skill: &crate::domain::Skill,
        args: Vec<String>,
        resolved: &super::dependency::ResolvedSkillsCli,
        cwd: PathBuf,
        cancel: &CancellationToken,
        progress: &ProgressSender,
    ) -> GroupOutcome {
        let full_args: Vec<String> = resolved.leading_args.iter().cloned().chain(args).collect();
        let program = resolved.program.clone();
        let executed_command = preview_command(&program, &full_args);

        let _ = progress.send(InstallProgressEvent::Command {
            skill_id: skill.id.clone(),
            command: executed_command,
        });

        let (tx, mut rx) = mpsc::unbounded_channel::<crate::process::ProcessOutputLine>();
        let progress_for_output = progress.clone();
        let skill_id_for_output = skill.id.clone();
        let output_task = tokio::spawn(async move {
            while let Some(line) = rx.recv().await {
                let _ = progress_for_output.send(InstallProgressEvent::Output {
                    skill_id: skill_id_for_output.clone(),
                    line: line.line,
                    stream: line.stream,
                });
            }
        });

        let spec = ProcessSpec {
            program,
            args: full_args,
            cwd: Some(cwd),
            env: Vec::new(),
        };
        let outcome = self.process_runner.run(spec, tx, cancel.clone()).await;
        let _ = output_task.await;

        match outcome {
            Ok(ProcessOutcome::Completed { exit_code: 0 }) => GroupOutcome::Success,
            Ok(ProcessOutcome::Completed { exit_code }) => {
                GroupOutcome::Failed(format!("exited with code {exit_code}"))
            }
            Ok(ProcessOutcome::Cancelled) => GroupOutcome::Cancelled,
            Err(err) => GroupOutcome::Failed(err.to_string()),
        }
    }

    /// The directory a batch's `skills` invocations run in: the GUI's
    /// selected destination folder when set (trimmed; empty/whitespace
    /// treated the same as unset, see `empty_project_path_falls_back_to_project_root`
    /// test), otherwise this installer's own project root.
    fn resolve_cwd(&self, project_path: Option<&str>) -> PathBuf {
        project_path
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| self.project_root.clone())
    }

    async fn run_and_collect(
        &self,
        program: &str,
        args: Vec<String>,
        cwd: PathBuf,
        cancel: CancellationToken,
    ) -> Result<ProcessOutcome, AppError> {
        let (tx, mut rx) = mpsc::unbounded_channel::<crate::process::ProcessOutputLine>();
        let drain = tokio::spawn(async move { while rx.recv().await.is_some() {} });
        let spec = ProcessSpec {
            program: program.to_string(),
            args,
            cwd: Some(cwd),
            env: Vec::new(),
        };
        let outcome = self.process_runner.run(spec, tx, cancel).await;
        let _ = drain.await;
        outcome
    }
}

fn owner_from_repository_url(repository_url: &str) -> Result<String, AppError> {
    let parsed = url::Url::parse(repository_url).map_err(|_| {
        AppError::Installation(format!("invalid repository URL \"{repository_url}\""))
    })?;
    parsed
        .path_segments()
        .and_then(|mut segments| segments.next())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            AppError::Installation(format!(
                "could not determine owner from \"{repository_url}\""
            ))
        })
}

#[async_trait]
impl Installer for SkillsCliInstaller {
    async fn check_dependencies(&self) -> Result<DependencyStatus, AppError> {
        Ok(self.resolver.status().await)
    }

    async fn list_installed(&self, _options: ListOptions) -> Result<Vec<InstalledSkill>, AppError> {
        // Deliberately not implemented here: "what's installed" is derived
        // from `.agents/skills/*/SKILL.md` on disk (see
        // `skills::discovery`) at the `SkillsService` layer, using the same
        // domain representation as configured skills — not by re-parsing
        // `skills list` output, which has no documented `--json` mode to
        // parse reliably.
        Ok(Vec::new())
    }

    async fn install(
        &self,
        batch: InstallBatch,
        progress: ProgressSender,
        cancel: CancellationToken,
    ) -> Result<InstallResult, AppError> {
        let total = batch.skills.len();
        let _ = progress.send(InstallProgressEvent::Start { total });

        let mut result = InstallResult {
            requested: total,
            ..Default::default()
        };

        let mut resolved_cli = None;

        for (index, skill) in batch.skills.iter().enumerate() {
            if cancel.is_cancelled() {
                result.cancelled = true;
                break;
            }

            let _ = progress.send(InstallProgressEvent::Progress {
                current: index + 1,
                total,
                skill_id: skill.id.clone(),
                display_name: skill.display_name.clone(),
            });

            let groups = match Self::add_args_for_skill(skill, &batch.options) {
                Ok(groups) => groups,
                Err(err) => {
                    result.failed += 1;
                    result.per_skill.push(SkillInstallOutcome {
                        skill_id: skill.id.clone(),
                        display_name: skill.display_name.clone(),
                        status: SkillInstallStatus::Failed,
                        message: Some(err.to_string()),
                        command_preview: String::new(),
                    });
                    let _ = progress.send(InstallProgressEvent::SkillError {
                        skill_id: skill.id.clone(),
                        display_name: skill.display_name.clone(),
                        message: err.to_string(),
                    });
                    if !batch.options.continue_on_error {
                        break;
                    }
                    continue;
                }
            };

            let combined_preview = groups
                .iter()
                .map(|args| preview_command("skills", args))
                .collect::<Vec<_>>()
                .join("\n");

            if batch.options.dry_run {
                for args in &groups {
                    let _ = progress.send(InstallProgressEvent::Command {
                        skill_id: skill.id.clone(),
                        command: preview_command("skills", args),
                    });
                }
                result.per_skill.push(SkillInstallOutcome {
                    skill_id: skill.id.clone(),
                    display_name: skill.display_name.clone(),
                    status: SkillInstallStatus::Skipped,
                    message: Some("dry run — command not executed".to_string()),
                    command_preview: combined_preview,
                });
                let _ = progress.send(InstallProgressEvent::SkillSuccess {
                    skill_id: skill.id.clone(),
                    display_name: skill.display_name.clone(),
                });
                continue;
            }

            if resolved_cli.is_none() {
                resolved_cli = Some(self.resolver.resolve().await);
            }
            let resolved = match resolved_cli.as_ref().unwrap() {
                Ok(resolved) => resolved,
                Err(err) => {
                    let _ = progress.send(InstallProgressEvent::Command {
                        skill_id: skill.id.clone(),
                        command: combined_preview.clone(),
                    });
                    result.failed += 1;
                    result.per_skill.push(SkillInstallOutcome {
                        skill_id: skill.id.clone(),
                        display_name: skill.display_name.clone(),
                        status: SkillInstallStatus::Failed,
                        message: Some(err.to_string()),
                        command_preview: combined_preview,
                    });
                    let _ = progress.send(InstallProgressEvent::SkillError {
                        skill_id: skill.id.clone(),
                        display_name: skill.display_name.clone(),
                        message: err.to_string(),
                    });
                    if !batch.options.continue_on_error {
                        break;
                    }
                    continue;
                }
            };

            let cwd = self.resolve_cwd(batch.options.project_path.as_deref());
            let mut failures: Vec<String> = Vec::new();
            let mut cancelled_here = false;
            for args in groups {
                match self
                    .run_one_group(skill, args, resolved, cwd.clone(), &cancel, &progress)
                    .await
                {
                    GroupOutcome::Success => {}
                    GroupOutcome::Failed(message) => {
                        failures.push(message);
                        if !batch.options.continue_on_error {
                            break;
                        }
                    }
                    GroupOutcome::Cancelled => {
                        cancelled_here = true;
                        break;
                    }
                }
            }

            if cancelled_here {
                result.cancelled = true;
                break;
            }

            if failures.is_empty() {
                result.installed += 1;
                result.per_skill.push(SkillInstallOutcome {
                    skill_id: skill.id.clone(),
                    display_name: skill.display_name.clone(),
                    status: SkillInstallStatus::Installed,
                    message: None,
                    command_preview: combined_preview,
                });
                let _ = progress.send(InstallProgressEvent::SkillSuccess {
                    skill_id: skill.id.clone(),
                    display_name: skill.display_name.clone(),
                });
            } else {
                result.failed += 1;
                let message = failures.join("; ");
                result.per_skill.push(SkillInstallOutcome {
                    skill_id: skill.id.clone(),
                    display_name: skill.display_name.clone(),
                    status: SkillInstallStatus::Failed,
                    message: Some(message.clone()),
                    command_preview: combined_preview,
                });
                let _ = progress.send(InstallProgressEvent::SkillError {
                    skill_id: skill.id.clone(),
                    display_name: skill.display_name.clone(),
                    message,
                });
                if !batch.options.continue_on_error {
                    break;
                }
            }
        }

        // The real `skills` CLI writes its own skills-lock.json (a
        // package-lock-style manifest of what it has seen) into whatever
        // directory it ran in — a file this app's users never asked for and
        // don't want left behind. Best-effort: ignore the error when there's
        // nothing to remove (dry runs, or every skill failing before the CLI
        // ever wrote one).
        let _ = std::fs::remove_file(
            self.resolve_cwd(batch.options.project_path.as_deref())
                .join("skills-lock.json"),
        );

        let _ = progress.send(InstallProgressEvent::Complete {
            result: result.clone(),
        });
        Ok(result)
    }

    /// `skills remove` takes skill names directly as positional args — no
    /// `owner/repo` source, no `--skill` flag, unlike `add` — and accepts
    /// more than one name in a single invocation, so a bulk uninstall from
    /// the GUI is exactly one process call: `skills remove name1 name2 ...
    /// --yes`. No `--agent`/`--global` filtering: this app currently only
    /// ever detects project-scoped installs (see `skills::discovery`), so
    /// removing without those flags — which the real CLI resolves against
    /// every agent it finds the skill installed for, at project scope — is
    /// the one behavior this app can honestly represent as "uninstalled".
    async fn remove(&self, request: RemoveRequest) -> Result<UninstallResult, AppError> {
        let requested = request.skills.len();
        if requested == 0 {
            return Ok(UninstallResult::default());
        }
        let resolved = self.resolver.resolve().await?;
        let mut args = vec!["remove".to_string()];
        args.extend(request.skills.iter().map(|skill| skill.skill_name.clone()));
        args.push("--yes".to_string());
        let full_args: Vec<String> = resolved.leading_args.iter().cloned().chain(args).collect();
        let cwd = self.resolve_cwd(request.options.project_path.as_deref());

        match self
            .run_and_collect(&resolved.program, full_args, cwd, CancellationToken::new())
            .await?
        {
            ProcessOutcome::Completed { exit_code: 0 } => Ok(UninstallResult {
                requested,
                removed: requested,
                failed: 0,
                message: None,
            }),
            ProcessOutcome::Completed { exit_code } => Ok(UninstallResult {
                requested,
                removed: 0,
                failed: requested,
                message: Some(format!("exited with code {exit_code}")),
            }),
            ProcessOutcome::Cancelled => Err(AppError::Cancelled),
        }
    }

    async fn update(&self, request: UpdateRequest) -> Result<(), AppError> {
        let resolved = self.resolver.resolve().await?;
        let mut args = vec!["update".to_string(), "--yes".to_string()];
        args.push(
            if request.options.any_global() {
                "--global"
            } else {
                "--project"
            }
            .to_string(),
        );
        let full_args: Vec<String> = resolved.leading_args.iter().cloned().chain(args).collect();
        let cwd = self.resolve_cwd(request.options.project_path.as_deref());

        match self
            .run_and_collect(&resolved.program, full_args, cwd, CancellationToken::new())
            .await?
        {
            ProcessOutcome::Completed { exit_code: 0 } => Ok(()),
            ProcessOutcome::Completed { exit_code } => Err(AppError::Installation(format!(
                "update exited with code {exit_code}"
            ))),
            ProcessOutcome::Cancelled => Err(AppError::Cancelled),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::InstallOptions;

    #[test]
    fn add_args_for_skill_builds_one_group_when_every_agent_shares_a_scope() {
        let global_only = crate::domain::AgentScopeSelection {
            project: false,
            global: true,
        };
        let mut agent_scopes = std::collections::HashMap::new();
        agent_scopes.insert("claude-code".to_string(), global_only);
        agent_scopes.insert("codex".to_string(), global_only);
        let options = InstallOptions {
            agents: vec!["claude-code".into(), "codex".into()],
            agent_scopes,
            copy: true,
            ..InstallOptions::default()
        };
        let groups =
            SkillsCliInstaller::add_args_for_skill(&remote_skill("triage"), &options).unwrap();
        assert_eq!(
            groups,
            vec![vec![
                "add".to_string(),
                "mattpocock/skills".to_string(),
                "--skill".to_string(),
                "triage".to_string(),
                "--agent".to_string(),
                "claude-code".to_string(),
                "codex".to_string(),
                "--global".to_string(),
                "--copy".to_string(),
                "--yes".to_string(),
            ]]
        );
    }

    #[test]
    fn add_args_for_skill_omits_optional_flags_when_unset() {
        let options = InstallOptions {
            agents: vec![],
            copy: false,
            ..InstallOptions::default()
        };
        let groups =
            SkillsCliInstaller::add_args_for_skill(&remote_skill("impeccable"), &options).unwrap();
        assert_eq!(
            groups,
            vec![vec![
                "add".to_string(),
                "mattpocock/skills".to_string(),
                "--skill".to_string(),
                "impeccable".to_string(),
                "--yes".to_string(),
            ]]
        );
    }

    #[test]
    fn add_args_for_skill_always_includes_yes() {
        let groups =
            SkillsCliInstaller::add_args_for_skill(&remote_skill("s"), &InstallOptions::default())
                .unwrap();
        assert!(groups
            .iter()
            .all(|args| args.contains(&"--yes".to_string())));
    }

    // The real CLI's `--global` flag applies to its whole invocation, so
    // mixing Project- and Global-scoped agents in one install has to become
    // two separate `skills add` calls — this is the behavior that makes the
    // Agents dialog's per-agent scope control actually take effect.
    #[test]
    fn add_args_for_skill_splits_into_two_groups_for_mixed_scope() {
        let mut agent_scopes = std::collections::HashMap::new();
        agent_scopes.insert(
            "claude-code".to_string(),
            crate::domain::AgentScopeSelection {
                project: false,
                global: true,
            },
        );
        // "cursor" is deliberately left out of `agent_scopes` — it must
        // default to Project-only, same as `InstallOptions::scopes_for`.
        let options = InstallOptions {
            agents: vec!["claude-code".into(), "cursor".into()],
            agent_scopes,
            copy: false,
            ..InstallOptions::default()
        };
        let groups =
            SkillsCliInstaller::add_args_for_skill(&remote_skill("triage"), &options).unwrap();
        assert_eq!(groups.len(), 2, "one group per distinct scope");
        assert_eq!(
            groups[0],
            vec![
                "add".to_string(),
                "mattpocock/skills".to_string(),
                "--skill".to_string(),
                "triage".to_string(),
                "--agent".to_string(),
                "cursor".to_string(),
                "--yes".to_string(),
            ],
            "Project group comes first and carries no --global flag"
        );
        assert_eq!(
            groups[1],
            vec![
                "add".to_string(),
                "mattpocock/skills".to_string(),
                "--skill".to_string(),
                "triage".to_string(),
                "--agent".to_string(),
                "claude-code".to_string(),
                "--global".to_string(),
                "--yes".to_string(),
            ],
            "Global group comes second"
        );
    }

    // Project and Global aren't exclusive — the same agent, selected for
    // both, must appear in *both* groups so it actually installs to both
    // destinations from one action.
    #[test]
    fn add_args_for_skill_puts_one_agent_in_both_groups_when_selected_for_both_scopes() {
        let mut agent_scopes = std::collections::HashMap::new();
        agent_scopes.insert(
            "claude-code".to_string(),
            crate::domain::AgentScopeSelection {
                project: true,
                global: true,
            },
        );
        let options = InstallOptions {
            agents: vec!["claude-code".into()],
            agent_scopes,
            ..InstallOptions::default()
        };
        let groups =
            SkillsCliInstaller::add_args_for_skill(&remote_skill("triage"), &options).unwrap();
        assert_eq!(
            groups.len(),
            2,
            "one group per scope, both containing claude-code"
        );
        assert!(
            groups[0].contains(&"claude-code".to_string())
                && !groups[0].contains(&"--global".to_string())
        );
        assert!(
            groups[1].contains(&"claude-code".to_string())
                && groups[1].contains(&"--global".to_string())
        );
    }

    // `vscode` has no CLI id of its own (see `domain::cli_agent_id`) — if it
    // and `github-copilot` both land in the same scope group, the group must
    // still send `--agent github-copilot` exactly once.
    #[test]
    fn add_args_for_skill_dedupes_translated_ids_within_a_group() {
        let options = InstallOptions {
            agents: vec!["vscode".into(), "github-copilot".into()],
            ..InstallOptions::default()
        };
        let groups =
            SkillsCliInstaller::add_args_for_skill(&remote_skill("triage"), &options).unwrap();
        assert_eq!(groups.len(), 1);
        let agent_count = groups[0].iter().filter(|a| *a == "github-copilot").count();
        assert_eq!(agent_count, 1);
    }

    #[test]
    fn owner_from_repository_url_extracts_first_segment() {
        assert_eq!(
            owner_from_repository_url("https://github.com/mattpocock/skills").unwrap(),
            "mattpocock"
        );
        assert_eq!(
            owner_from_repository_url("https://github.com/pbakaus/impeccable").unwrap(),
            "pbakaus"
        );
    }

    #[test]
    fn owner_from_repository_url_rejects_malformed_urls() {
        assert!(owner_from_repository_url("not a url").is_err());
        assert!(owner_from_repository_url("https://github.com/").is_err());
    }

    // --- install() loop/aggregation tests -----------------------------------

    use crate::domain::{OutputStream, Skill, SkillInstallStatus, SkillSource, OTHER_GROUP_ID};
    use crate::installer::{FakeProcessResult, FakeProcessRunner};
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use tokio::sync::mpsc;

    fn remote_skill(id: &str) -> Skill {
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
            enabled: true,
        }
    }

    fn local_skill(id: &str) -> Skill {
        Skill {
            source: SkillSource::Local {
                path: format!("/tmp/{id}"),
            },
            local: true,
            installed: true,
            ..remote_skill(id)
        }
    }

    // On overlayfs (Docker's default storage driver), a file just written
    // and chmod'd can transiently exec-fail with ETXTBSY ("Text file busy")
    // under concurrent `cargo test` threads — a known overlayfs copy-up
    // race, not an incomplete write. `sync_all` flushes *this* write, but
    // under real parallel load the race is filesystem-wide: other threads
    // writing+chmod'ing+exec'ing their own fresh files at the same moment
    // can still transiently trip it here. A throwaway warm-up spawn,
    // retried briefly, settles that race before any real test logic
    // depends on this specific file being immediately spawnable (see
    // `platform::path_augment`'s `fake_shell`).
    fn resolver_with_fake_skills_executable() -> (DependencyResolver, tempfile::TempDir) {
        use std::io::Write;
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("skills");
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(b"#!/bin/sh\nexit 0\n").unwrap();
        file.sync_all().unwrap();
        drop(file);
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        for _ in 0..20 {
            if std::process::Command::new(&path).output().is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        (
            DependencyResolver::with_search_paths(vec![tmp.path().to_path_buf()]),
            tmp,
        )
    }

    async fn drain(
        mut rx: mpsc::UnboundedReceiver<InstallProgressEvent>,
    ) -> Vec<InstallProgressEvent> {
        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event);
        }
        events
    }

    #[tokio::test]
    async fn local_catalog_skills_are_installed_through_the_cli() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let (tx, rx) = mpsc::unbounded_channel();
        let batch = InstallBatch {
            skills: vec![local_skill("tauri-v2")],
            options: crate::domain::InstallOptions::default(),
        };
        let result = installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        drop(rx);

        assert_eq!(result.installed, 1);
        assert_eq!(process_runner.call_count(), 1);
        let call = process_runner.calls.lock().unwrap()[0].clone();
        assert!(call.args.iter().any(|arg| arg == "/tmp"));
    }

    #[tokio::test]
    async fn dry_run_never_spawns_and_reports_previews() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let (tx, rx) = mpsc::unbounded_channel();
        let options = crate::domain::InstallOptions {
            dry_run: true,
            ..Default::default()
        };
        let batch = InstallBatch {
            skills: vec![remote_skill("triage")],
            options,
        };
        let result = installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        drop(rx);

        assert_eq!(
            process_runner.call_count(),
            0,
            "dry run must never spawn a process"
        );
        assert_eq!(result.per_skill[0].status, SkillInstallStatus::Skipped);
        assert!(result.per_skill[0]
            .command_preview
            .contains("skills add mattpocock/skills --skill triage"));
    }

    #[tokio::test]
    async fn successful_install_increments_installed_count_and_streams_output() {
        let process_runner =
            Arc::new(FakeProcessRunner::new(vec![FakeProcessResult::success()
                .with_output(OutputStream::Stdout, "✓ triage installed")]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let (tx, rx) = mpsc::unbounded_channel();
        let batch = InstallBatch {
            skills: vec![remote_skill("triage")],
            options: crate::domain::InstallOptions::default(),
        };
        let result = installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        let events = drain(rx).await;

        assert_eq!(result.installed, 1);
        assert_eq!(process_runner.call_count(), 1);
        assert!(events.iter().any(|event| {
            matches!(
                event,
                InstallProgressEvent::Command { command, .. }
                    if command.contains("skills add mattpocock/skills --skill triage")
            )
        }));
        assert!(events
            .iter()
            .any(|e| matches!(e, InstallProgressEvent::Output { line, .. } if line == "✓ triage installed")));
    }

    #[tokio::test]
    async fn failed_install_continues_when_continue_on_error_is_true() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![
            FakeProcessResult::failure(1),
            FakeProcessResult::success(),
        ]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let (tx, rx) = mpsc::unbounded_channel();
        let options = crate::domain::InstallOptions {
            continue_on_error: true,
            ..Default::default()
        };
        let batch = InstallBatch {
            skills: vec![remote_skill("a"), remote_skill("b")],
            options,
        };
        let result = installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        drop(rx);

        assert_eq!(result.failed, 1);
        assert_eq!(result.installed, 1);
        assert_eq!(
            process_runner.call_count(),
            2,
            "should have attempted both skills"
        );
    }

    #[tokio::test]
    async fn failed_install_stops_when_continue_on_error_is_false() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![FakeProcessResult::failure(1)]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let (tx, rx) = mpsc::unbounded_channel();
        let options = crate::domain::InstallOptions {
            continue_on_error: false,
            ..Default::default()
        };
        let batch = InstallBatch {
            skills: vec![remote_skill("a"), remote_skill("b")],
            options,
        };
        let result = installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        drop(rx);

        assert_eq!(result.failed, 1);
        assert_eq!(
            process_runner.call_count(),
            1,
            "must stop after the first failure"
        );
    }

    #[tokio::test]
    async fn cancellation_before_the_loop_starts_marks_the_result_cancelled() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let cancel = CancellationToken::new();
        cancel.cancel();

        let (tx, rx) = mpsc::unbounded_channel();
        let batch = InstallBatch {
            skills: vec![remote_skill("a")],
            options: crate::domain::InstallOptions::default(),
        };
        let result = installer.install(batch, tx, cancel).await.unwrap();
        drop(rx);

        assert!(result.cancelled);
        assert_eq!(process_runner.call_count(), 0);
    }

    // --- cwd resolution -----------------------------------------------
    //
    // The GUI sends `InstallOptions.project_path` straight from
    // `state.projectRoot`, which starts out as `""` before the config load
    // populates it (see frontend `useAppState.ts`). `Command::current_dir("")`
    // fails to spawn at all (verified against the real OS: `ENOENT`), so an
    // empty string must be treated the same as `None` — fall back to
    // `project_root` — instead of being passed straight through.

    #[tokio::test]
    async fn empty_project_path_falls_back_to_project_root() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let project_root = PathBuf::from("/tmp/some-project-root");
        let installer = SkillsCliInstaller::with_resolver(
            process_runner.clone(),
            project_root.clone(),
            resolver,
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let options = crate::domain::InstallOptions {
            project_path: Some(String::new()),
            ..Default::default()
        };
        let batch = InstallBatch {
            skills: vec![remote_skill("triage")],
            options,
        };
        installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        drop(rx);

        let call = process_runner.calls.lock().unwrap()[0].clone();
        assert_eq!(call.cwd, Some(project_root));
    }

    #[tokio::test]
    async fn whitespace_only_project_path_falls_back_to_project_root() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let project_root = PathBuf::from("/tmp/some-project-root");
        let installer = SkillsCliInstaller::with_resolver(
            process_runner.clone(),
            project_root.clone(),
            resolver,
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let options = crate::domain::InstallOptions {
            project_path: Some("   ".to_string()),
            ..Default::default()
        };
        let batch = InstallBatch {
            skills: vec![remote_skill("triage")],
            options,
        };
        installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        drop(rx);

        let call = process_runner.calls.lock().unwrap()[0].clone();
        assert_eq!(call.cwd, Some(project_root));
    }

    #[tokio::test]
    async fn selected_project_path_is_used_as_cwd() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer = SkillsCliInstaller::with_resolver(
            process_runner.clone(),
            PathBuf::from("/tmp/some-project-root"),
            resolver,
        );

        let (tx, rx) = mpsc::unbounded_channel();
        let options = crate::domain::InstallOptions {
            project_path: Some("/tmp/user-selected-folder".to_string()),
            ..Default::default()
        };
        let batch = InstallBatch {
            skills: vec![remote_skill("triage")],
            options,
        };
        installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        drop(rx);

        let call = process_runner.calls.lock().unwrap()[0].clone();
        assert_eq!(call.cwd, Some(PathBuf::from("/tmp/user-selected-folder")));
    }

    #[tokio::test]
    async fn removes_the_skills_lock_file_the_real_cli_leaves_behind() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![FakeProcessResult::success()]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let tmp = tempfile::tempdir().unwrap();
        let installer = SkillsCliInstaller::with_resolver(
            process_runner.clone(),
            tmp.path().to_path_buf(),
            resolver,
        );

        // The FakeProcessRunner never actually runs `skills`, so simulate the
        // side effect the real CLI leaves behind in the install cwd.
        let lock_file = tmp.path().join("skills-lock.json");
        std::fs::write(&lock_file, "{}").unwrap();

        let (tx, rx) = mpsc::unbounded_channel();
        let batch = InstallBatch {
            skills: vec![remote_skill("triage")],
            options: crate::domain::InstallOptions::default(),
        };
        installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        drop(rx);

        assert!(
            !lock_file.exists(),
            "skills-lock.json should be removed after install"
        );
    }

    #[tokio::test]
    async fn missing_dependency_is_reported_as_a_failure_per_skill() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![]));
        let empty_dir = tempfile::tempdir().unwrap();
        let resolver = DependencyResolver::with_search_paths(vec![empty_dir.path().to_path_buf()]);
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let (tx, rx) = mpsc::unbounded_channel();
        let batch = InstallBatch {
            skills: vec![remote_skill("a")],
            options: crate::domain::InstallOptions::default(),
        };
        let result = installer
            .install(batch, tx, CancellationToken::new())
            .await
            .unwrap();
        drop(rx);

        assert_eq!(result.failed, 1);
        assert_eq!(process_runner.call_count(), 0);
        assert!(result.per_skill[0]
            .message
            .as_deref()
            .unwrap()
            .contains("Skills CLI"));
    }

    // --- remove() ------------------------------------------------------

    #[tokio::test]
    async fn remove_sends_every_skill_name_in_one_bulk_call() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![FakeProcessResult::success()]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let result = installer
            .remove(RemoveRequest {
                skills: vec![remote_skill("triage"), remote_skill("tdd")],
                options: crate::domain::InstallOptions::default(),
            })
            .await
            .unwrap();

        assert_eq!(result.requested, 2);
        assert_eq!(result.removed, 2);
        assert_eq!(
            process_runner.call_count(),
            1,
            "one process call for the whole bulk remove"
        );
        let call = process_runner.calls.lock().unwrap()[0].clone();
        // No owner/repo, no --skill flag — just the bare skill names, per the
        // real CLI's `skills remove name1 name2 --yes` syntax.
        assert_eq!(call.args, vec!["remove", "triage", "tdd", "--yes"]);
    }

    #[tokio::test]
    async fn remove_reports_failure_on_a_non_zero_exit_code() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![FakeProcessResult::failure(1)]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let result = installer
            .remove(RemoveRequest {
                skills: vec![remote_skill("triage")],
                options: crate::domain::InstallOptions::default(),
            })
            .await
            .unwrap();

        assert_eq!(result.removed, 0);
        assert_eq!(result.failed, 1);
        assert!(result.message.is_some());
    }

    #[tokio::test]
    async fn remove_with_no_skills_never_spawns_a_process() {
        let process_runner = Arc::new(FakeProcessRunner::new(vec![]));
        let (resolver, _guard) = resolver_with_fake_skills_executable();
        let installer =
            SkillsCliInstaller::with_resolver(process_runner.clone(), PathBuf::from("."), resolver);

        let result = installer
            .remove(RemoveRequest {
                skills: vec![],
                options: crate::domain::InstallOptions::default(),
            })
            .await
            .unwrap();

        assert_eq!(result.requested, 0);
        assert_eq!(process_runner.call_count(), 0);
    }
}
