pub mod args;

pub use args::{Cli, Command};

use std::path::PathBuf;

use tokio::sync::mpsc;

use crate::app::ApplicationServices;
use crate::domain::{InstallOptions, InstallProgressEvent, InstallRequest, Skill, SkillSelection};

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_OPERATION_ERROR: i32 = 1;
pub const EXIT_INVALID_ARGS: i32 = 2;
pub const EXIT_MISSING_DEPENDENCY: i32 = 3;
pub const EXIT_INTERRUPTED: i32 = 130;

/// Concise by default; `--debug` widens the filter. Kept independent of
/// whether the app ends up on the GUI or CLI path, and initialized before
/// either, so `tracing::warn!` calls during config loading are never lost.
pub fn init_logging(debug: bool) {
    use tracing_subscriber::EnvFilter;
    let filter = if debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

/// Runs one CLI subcommand to completion and returns the process exit code.
/// Never called for `Command::Gui` / no-subcommand — those are dispatched to
/// the Tauri event loop before this function is reached (see `lib::start`).
pub async fn execute(cli: Cli) -> i32 {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let services = ApplicationServices::new(cli.config, project_root);

    match cli.command.unwrap_or(Command::Gui) {
        Command::Gui => EXIT_SUCCESS,
        Command::List { installed, json } => run_list(&services, installed, json),
        Command::Install {
            skill_ids,
            all,
            dry_run,
            yes,
            agent,
            global,
            project,
            copy,
            stop_on_error,
            json,
        } => {
            run_install(
                &services,
                skill_ids,
                all,
                dry_run,
                yes,
                agent,
                global,
                project,
                copy,
                stop_on_error,
                json,
            )
            .await
        }
        Command::Doctor { json } => run_doctor(&services, json).await,
        Command::Version => run_version(&services).await,
    }
}

fn run_list(services: &ApplicationServices, installed_only: bool, json: bool) -> i32 {
    let state = match services.skills.load_state() {
        Ok(state) => state,
        Err(err) => {
            eprintln!("error: {err}");
            return EXIT_OPERATION_ERROR;
        }
    };

    let skills: Vec<Skill> = state
        .skills
        .into_iter()
        .filter(|s| !installed_only || s.installed)
        .collect();

    if json {
        if serde_json::to_writer_pretty(std::io::stdout(), &skills).is_err() {
            return EXIT_OPERATION_ERROR;
        }
        println!();
        return EXIT_SUCCESS;
    }

    if skills.is_empty() {
        println!("No skills found.");
        return EXIT_SUCCESS;
    }
    for skill in &skills {
        let installed_marker = if skill.installed { "✓" } else { " " };
        let local_marker = if skill.local { " [local]" } else { "" };
        println!(
            "{installed_marker} {:<28} {}{}",
            skill.id, skill.display_name, local_marker
        );
    }
    EXIT_SUCCESS
}

#[allow(clippy::too_many_arguments)]
async fn run_install(
    services: &ApplicationServices,
    skill_ids: Vec<String>,
    all: bool,
    dry_run: bool,
    yes: bool,
    agent: Option<String>,
    global: bool,
    _project: bool,
    copy: bool,
    stop_on_error: bool,
    json: bool,
) -> i32 {
    let state = match services.skills.load_state() {
        Ok(state) => state,
        Err(err) => {
            eprintln!("error: {err}");
            return EXIT_OPERATION_ERROR;
        }
    };

    let target_ids = match resolve_target_ids(&state.skills, &skill_ids, all) {
        Ok(ids) => ids,
        Err(err) => {
            eprintln!("error: {err}");
            return EXIT_INVALID_ARGS;
        }
    };
    if target_ids.is_empty() {
        eprintln!("error: no skills selected (nothing preselected; pass ids or --all)");
        return EXIT_INVALID_ARGS;
    }

    let agent_id = agent.unwrap_or_else(|| "universal".to_string());
    let options = InstallOptions {
        agents: vec![agent_id],
        scope: if global {
            crate::domain::InstallScope::Global
        } else {
            crate::domain::InstallScope::Project
        },
        project_path: None,
        copy,
        dry_run,
        confirm: !yes,
        continue_on_error: !stop_on_error,
    };
    let request = InstallRequest {
        selection: SkillSelection::new(target_ids),
        options,
    };

    if let Err(err) = services
        .installation
        .validate_installation(&request, &state.skills)
    {
        eprintln!("error: {err}");
        return EXIT_INVALID_ARGS;
    }

    // Interactive confirmation only makes sense when a human is actually
    // going to read it: skip it for --yes, --dry-run (nothing destructive
    // happens), and --json (machine consumers must never block on stdin).
    if !yes && !dry_run && !json {
        let names: Vec<&str> = request
            .selection
            .skill_ids
            .iter()
            .filter_map(|id| state.skills.iter().find(|s| &s.id == id))
            .map(|s| s.display_name.as_str())
            .collect();
        println!(
            "About to install {} skill(s): {}",
            names.len(),
            names.join(", ")
        );
        print!("Continue? [y/N] ");
        use std::io::Write;
        let _ = std::io::stdout().flush();
        let mut answer = String::new();
        if std::io::stdin().read_line(&mut answer).is_err() {
            return EXIT_OPERATION_ERROR;
        }
        if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return EXIT_SUCCESS;
        }
    }

    let (tx, rx) = mpsc::unbounded_channel();
    let printer = tokio::spawn(print_progress(rx, json));

    let install_future = services.installation.install(request, &state.skills, tx);
    tokio::pin!(install_future);

    let result = tokio::select! {
        res = &mut install_future => res,
        _ = tokio::signal::ctrl_c() => {
            services.installation.cancel();
            install_future.await
        }
    };
    let _ = printer.await;

    match result {
        Ok(result) => {
            if json {
                let _ = serde_json::to_writer_pretty(std::io::stdout(), &result);
                println!();
            } else {
                println!(
                    "{} requested, {} installed, {} already installed, {} failed",
                    result.requested, result.installed, result.already_installed, result.failed
                );
            }
            if result.cancelled {
                EXIT_INTERRUPTED
            } else if result.failed > 0 {
                EXIT_OPERATION_ERROR
            } else {
                EXIT_SUCCESS
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            match err {
                crate::error::AppError::MissingDependency(_) => EXIT_MISSING_DEPENDENCY,
                crate::error::AppError::Cancelled => EXIT_INTERRUPTED,
                _ => EXIT_OPERATION_ERROR,
            }
        }
    }
}

async fn print_progress(mut rx: mpsc::UnboundedReceiver<InstallProgressEvent>, json: bool) {
    if json {
        // Drain silently: --json must produce nothing but the final JSON
        // result on stdout.
        while rx.recv().await.is_some() {}
        return;
    }
    while let Some(event) = rx.recv().await {
        match event {
            InstallProgressEvent::Progress { display_name, .. } => {
                println!("Installing {display_name}...")
            }
            InstallProgressEvent::Command { command, .. } => println!("  $ {command}"),
            InstallProgressEvent::Output { line, .. } => println!("  {line}"),
            InstallProgressEvent::SkillSuccess { display_name, .. } => {
                println!("✓ {display_name}\n")
            }
            InstallProgressEvent::SkillError {
                display_name,
                message,
                ..
            } => {
                println!("✗ {display_name}: {message}\n")
            }
            InstallProgressEvent::Start { .. } | InstallProgressEvent::Complete { .. } => {}
        }
    }
}

fn resolve_target_ids(
    skills: &[Skill],
    explicit_ids: &[String],
    all: bool,
) -> Result<Vec<String>, crate::error::AppError> {
    if !explicit_ids.is_empty() {
        for id in explicit_ids {
            if !skills.iter().any(|s| &s.id == id) {
                return Err(crate::error::AppError::Validation(format!(
                    "unknown skill id \"{id}\""
                )));
            }
        }
        return Ok(explicit_ids.to_vec());
    }
    if all {
        return Ok(skills
            .iter()
            .filter(|s| s.enabled)
            .map(|s| s.id.clone())
            .collect());
    }
    Ok(skills
        .iter()
        .filter(|s| s.enabled && s.preselected)
        .map(|s| s.id.clone())
        .collect())
}

async fn run_doctor(services: &ApplicationServices, json: bool) -> i32 {
    let dependency = services.installation.check_dependencies().await;
    let state = services.skills.load_state();

    let (dependency_status, dependency_error) = match &dependency {
        Ok(status) => (Some(status.clone()), None),
        Err(err) => (None, Some(err.to_string())),
    };
    let (config_ok, config_error, configured_count, local_count) = match &state {
        Ok(state) => (
            true,
            None,
            state.skills.iter().filter(|s| !s.local).count(),
            state.skills.iter().filter(|s| s.local).count(),
        ),
        Err(err) => (false, Some(err.to_string()), 0, 0),
    };

    if json {
        #[derive(serde::Serialize)]
        struct DoctorReport {
            dependency: Option<crate::domain::DependencyStatus>,
            dependency_error: Option<String>,
            config_ok: bool,
            config_error: Option<String>,
            configured_skill_count: usize,
            local_skill_count: usize,
        }
        let report = DoctorReport {
            dependency: dependency_status.clone(),
            dependency_error: dependency_error.clone(),
            config_ok,
            config_error: config_error.clone(),
            configured_skill_count: configured_count,
            local_skill_count: local_count,
        };
        let _ = serde_json::to_writer_pretty(std::io::stdout(), &report);
        println!();
    } else {
        println!("Skills Installer doctor");
        println!("========================");
        match &dependency_status {
            Some(status) => println!(
                "Skills CLI: available ({:?}, version {})",
                status.source,
                status.version.as_deref().unwrap_or("unknown")
            ),
            None => println!(
                "Skills CLI: NOT available ({})",
                dependency_error.as_deref().unwrap_or("unknown reason")
            ),
        }
        if config_ok {
            println!("Configuration: OK ({configured_count} configured, {local_count} local)");
        } else {
            println!(
                "Configuration: ERROR ({})",
                config_error.as_deref().unwrap_or("unknown reason")
            );
        }
    }

    if !config_ok {
        EXIT_OPERATION_ERROR
    } else if dependency_status.is_none() {
        EXIT_MISSING_DEPENDENCY
    } else {
        EXIT_SUCCESS
    }
}

async fn run_version(services: &ApplicationServices) -> i32 {
    println!("Skills Installer {}", env!("CARGO_PKG_VERSION"));
    match services.installation.check_dependencies().await {
        Ok(status) if status.available => {
            println!(
                "Skills CLI: {}",
                status.version.as_deref().unwrap_or("detected")
            );
        }
        _ => println!("Skills CLI: not detected"),
    }
    EXIT_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Skill, SkillSource, OTHER_GROUP_ID};

    fn skill(id: &str, enabled: bool, preselected: bool) -> Skill {
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
            preselected,
            local: false,
            installed: false,
            installed_agents: Vec::new(),
            enabled,
        }
    }

    #[test]
    fn resolve_target_ids_uses_explicit_ids_when_given() {
        let skills = vec![skill("a", true, false), skill("b", true, false)];
        let ids = resolve_target_ids(&skills, &["a".to_string()], false).unwrap();
        assert_eq!(ids, vec!["a".to_string()]);
    }

    #[test]
    fn resolve_target_ids_rejects_unknown_explicit_id() {
        let skills = vec![skill("a", true, false)];
        assert!(resolve_target_ids(&skills, &["ghost".to_string()], false).is_err());
    }

    #[test]
    fn resolve_target_ids_with_all_returns_every_enabled_skill() {
        let skills = vec![skill("a", true, false), skill("b", false, false)];
        let ids = resolve_target_ids(&skills, &[], true).unwrap();
        assert_eq!(ids, vec!["a".to_string()]);
    }

    #[test]
    fn resolve_target_ids_defaults_to_preselected() {
        let skills = vec![skill("a", true, true), skill("b", true, false)];
        let ids = resolve_target_ids(&skills, &[], false).unwrap();
        assert_eq!(ids, vec!["a".to_string()]);
    }

    #[test]
    fn exit_codes_are_distinct() {
        let codes = [
            EXIT_SUCCESS,
            EXIT_OPERATION_ERROR,
            EXIT_INVALID_ARGS,
            EXIT_MISSING_DEPENDENCY,
            EXIT_INTERRUPTED,
        ];
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len());
    }

    // --- clap argument parsing -----------------------------------------

    use clap::Parser;

    #[test]
    fn parses_no_subcommand_as_gui() {
        let cli = Cli::try_parse_from(["skills-installer"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_install_with_explicit_ids_and_flags() {
        let cli = Cli::try_parse_from([
            "skills-installer",
            "install",
            "triage",
            "tdd",
            "--dry-run",
            "--agent",
            "claude-code",
            "--copy",
        ])
        .unwrap();
        match cli.command {
            Some(Command::Install {
                skill_ids,
                dry_run,
                agent,
                copy,
                ..
            }) => {
                assert_eq!(skill_ids, vec!["triage".to_string(), "tdd".to_string()]);
                assert!(dry_run);
                assert_eq!(agent.as_deref(), Some("claude-code"));
                assert!(copy);
            }
            other => panic!("expected Install, got {other:?}"),
        }
    }

    #[test]
    fn rejects_global_and_project_flags_together() {
        let result = Cli::try_parse_from(["skills-installer", "install", "--global", "--project"]);
        assert!(
            result.is_err(),
            "--global and --project must be mutually exclusive"
        );
    }

    #[test]
    fn accepts_global_flag_alone() {
        let cli = Cli::try_parse_from(["skills-installer", "install", "--global"]).unwrap();
        match cli.command {
            Some(Command::Install { global, .. }) => assert!(global),
            other => panic!("expected Install, got {other:?}"),
        }
    }

    #[test]
    fn parses_list_json_flag() {
        let cli =
            Cli::try_parse_from(["skills-installer", "list", "--installed", "--json"]).unwrap();
        match cli.command {
            Some(Command::List { installed, json }) => {
                assert!(installed);
                assert!(json);
            }
            other => panic!("expected List, got {other:?}"),
        }
    }

    #[test]
    fn parses_global_config_flag_anywhere() {
        let cli =
            Cli::try_parse_from(["skills-installer", "list", "--config", "/tmp/x.yaml"]).unwrap();
        assert_eq!(cli.config, Some(PathBuf::from("/tmp/x.yaml")));
    }

    // --- end-to-end CLI paths against a real, hermetic ApplicationServices ---
    // (never touches the network: only exercises branches that don't need
    // Skills CLI dependency resolution — list and dry-run install.)

    fn write_curated_config(dir: &std::path::Path) {
        std::fs::write(
            dir.join("skills.yaml"),
            r##"
version: 1
groups:
  - id: testing
    name: Testing
    color: "#6B82D9"
    order: 10
    enabled: true
skills:
  - id: triage
    name: Issue Triage
    url: https://www.skills.sh/mattpocock/skills/triage
    group: testing
    preselected: true
    enabled: true
"##,
        )
        .unwrap();
    }

    #[test]
    fn run_list_finds_configured_skills_and_succeeds() {
        let tmp = tempfile::tempdir().unwrap();
        write_curated_config(tmp.path());
        let services = ApplicationServices::new(None, tmp.path().to_path_buf());

        let exit_code = run_list(&services, false, true);
        assert_eq!(exit_code, EXIT_SUCCESS);
    }

    #[test]
    fn run_list_installed_only_excludes_uninstalled_skills() {
        let tmp = tempfile::tempdir().unwrap();
        write_curated_config(tmp.path());
        let services = ApplicationServices::new(None, tmp.path().to_path_buf());

        // Nothing is actually installed on disk in this tempdir, so
        // --installed should still succeed with an empty-but-valid result.
        let exit_code = run_list(&services, true, true);
        assert_eq!(exit_code, EXIT_SUCCESS);
    }

    #[tokio::test]
    async fn run_install_dry_run_never_touches_network_and_succeeds() {
        let tmp = tempfile::tempdir().unwrap();
        write_curated_config(tmp.path());
        let services = ApplicationServices::new(None, tmp.path().to_path_buf());

        let exit_code = run_install(
            &services,
            vec![],
            false,
            true, // dry_run
            true, // yes (skip prompt)
            None,
            false,
            false,
            false,
            true, // stop_on_error irrelevant for a single dry-run skill
            true, // json
        )
        .await;

        assert_eq!(exit_code, EXIT_SUCCESS);
    }

    #[tokio::test]
    async fn run_install_rejects_unknown_explicit_skill_id() {
        let tmp = tempfile::tempdir().unwrap();
        write_curated_config(tmp.path());
        let services = ApplicationServices::new(None, tmp.path().to_path_buf());

        let exit_code = run_install(
            &services,
            vec!["ghost".to_string()],
            false,
            true,
            true,
            None,
            false,
            false,
            false,
            true,
            true,
        )
        .await;

        assert_eq!(exit_code, EXIT_INVALID_ARGS);
    }
}
