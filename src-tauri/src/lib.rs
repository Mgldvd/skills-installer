pub mod app;
pub mod cli;
pub mod commands;
pub mod config;
pub mod domain;
pub mod error;
pub mod installer;
pub mod platform;
pub mod preferences;
pub mod process;
pub mod projects;
pub mod skills;

use std::path::PathBuf;

use clap::Parser;

/// Parses CLI arguments *before* touching Tauri at all (required: CLI
/// subcommands must run and exit without ever starting the GUI event
/// loop). Only the no-subcommand / `gui` path reaches `run_gui`.
pub fn start() {
    let cli_args = cli::Cli::parse();
    cli::init_logging(cli_args.debug);

    let is_gui = matches!(&cli_args.command, None | Some(cli::Command::Gui));
    if is_gui {
        if should_relaunch_detached() {
            match relaunch_gui_detached() {
                Ok(()) => return,
                Err(err) => {
                    eprintln!(
                        "warning: could not detach from the terminal, continuing in the foreground: {err}"
                    );
                }
            }
        }
        let config_override = cli_args.config.clone();
        run_gui(config_override);
    } else {
        let runtime = tokio::runtime::Runtime::new().expect("failed to start async runtime");
        let exit_code = runtime.block_on(cli::execute(cli_args));
        std::process::exit(exit_code);
    }
}

/// AppImage runtimes chdir into their ephemeral `/tmp/.mount_*` FUSE mount
/// before exec'ing the embedded binary, so a plain `current_dir()` no
/// longer reflects where the user actually launched it from. `$OWD`
/// ("original working directory") is the AppImage runtime's own
/// convention for preserving that — prefer it whenever it's set to an
/// existing directory.
fn original_cwd() -> PathBuf {
    let owd = std::env::var_os("OWD").map(PathBuf::from);
    let current_dir = std::env::current_dir().ok();
    resolve_original_cwd(owd, current_dir)
}

fn resolve_original_cwd(owd: Option<PathBuf>, current_dir: Option<PathBuf>) -> PathBuf {
    owd.filter(|path| path.is_dir())
        .or(current_dir)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// True only for the first, terminal-attached launch of an AppImage — e.g.
/// via the `skills` launcher `install_cli_command` symlinks into
/// `~/.local/bin`. A `.desktop` entry or file-manager double-click already
/// has no controlling terminal to release, so this leaves those alone.
/// `SKILLS_INSTALLER_GUI_DETACHED` marks the already-detached relaunch so
/// this doesn't loop.
fn should_relaunch_detached() -> bool {
    use std::io::IsTerminal;
    should_relaunch(
        std::env::var_os("APPIMAGE").is_some(),
        std::env::var_os("SKILLS_INSTALLER_GUI_DETACHED").is_some(),
        std::io::stdout().is_terminal(),
    )
}

fn should_relaunch(is_appimage: bool, already_detached: bool, stdout_is_terminal: bool) -> bool {
    is_appimage && !already_detached && stdout_is_terminal
}

/// Spawns a second, independent copy of this AppImage — detached from the
/// terminal's stdio *and* its controlling terminal — and returns
/// immediately so the shell prompt comes back right away, the way any
/// other desktop launcher behaves, instead of blocking until the window
/// closes.
///
/// Re-launches `$APPIMAGE` (the original file) rather than
/// `current_exe()` (which resolves inside *this* process's own
/// `/tmp/.mount_*`): once this process exits right after spawning the
/// child, the AppImage runtime tears its mount down, which would orphan a
/// child still reading from it. A fresh `$APPIMAGE` launch mounts and owns
/// its own independent copy. `.current_dir(original_cwd())` matters here
/// too: this process's own `current_dir()` is already inside the (soon to
/// be torn down) mount, so without it the relaunch would inherit that
/// wrong directory instead of the terminal's real one.
fn relaunch_gui_detached() -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};

    let appimage = std::env::var_os("APPIMAGE").expect("checked by should_relaunch_detached");
    // SAFETY: `setsid()` is async-signal-safe (POSIX-guaranteed safe to
    // call between fork and exec) and touches only this not-yet-exec'd
    // child, so it's sound inside `pre_exec`. Without it, redirecting
    // stdio alone leaves the child attached to the terminal's controlling
    // tty — the shell prompt returns, but the terminal emulator still
    // considers a process "running in this terminal" and warns on close.
    // `setsid()` makes the child a new session leader with no controlling
    // terminal at all, the standard Unix way to fully detach.
    unsafe {
        Command::new(appimage)
            .current_dir(original_cwd())
            .env("SKILLS_INSTALLER_GUI_DETACHED", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .pre_exec(|| {
                libc::setsid();
                Ok(())
            })
            .spawn()?;
    }
    Ok(())
}

// No `#[cfg_attr(mobile, tauri::mobile_entry_point)]` here: this app
// targets Linux desktop only (AppImage/.deb), never mobile, so the
// main.rs/lib.rs split exists purely for module organization, not to
// satisfy a mobile entry-point requirement.
fn run_gui(config_override: Option<PathBuf>) {
    // Preserve the launch context: starting the app from a project must
    // install into that project, never silently redirect to HOME.
    let mut project_root = original_cwd();
    // Last-resort safety net: only reachable if `$OWD` was unset (a
    // non-standard AppImage runtime, or none at all) and `current_dir()`
    // still landed inside the ephemeral mount tree — never a meaningful
    // project destination, and it disappears when the process exits.
    if project_root.to_string_lossy().starts_with("/tmp/.mount_") {
        if let Some(home) = std::env::var_os("HOME") {
            project_root = PathBuf::from(home);
        }
    }
    let services = app::ApplicationServices::new(config_override, project_root);
    let state = commands::AppState { services };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::skills::get_application_state,
            commands::skills::get_skills,
            commands::skills::get_installed_skills,
            commands::skills::preview_skill_url,
            commands::skills::preview_pack_url,
            commands::skills::import_pack,
            commands::skills::add_skill,
            commands::skills::update_skill,
            commands::skills::delete_skill,
            commands::skills::check_local_skill_updates,
            commands::tags::get_tags,
            commands::tags::create_tag,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::reorder_tags,
            commands::tags::assign_tag_to_skill,
            commands::tags::unassign_tag_from_skill,
            commands::configuration::export_portable_configuration,
            commands::configuration::import_portable_configuration,
            commands::preferences::get_preferences,
            commands::preferences::update_preferences,
            commands::preferences::install_cli_command,
            commands::projects::get_projects,
            commands::projects::save_project,
            commands::projects::delete_project,
            commands::installation::get_dependency_status,
            commands::installation::validate_installation,
            commands::installation::install_skills,
            commands::installation::cancel_installation,
            commands::installation::refresh,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_original_cwd_prefers_owd_when_it_is_a_real_directory() {
        let owd = tempfile::tempdir().unwrap();
        let current_dir = PathBuf::from("/tmp/.mount_whatever");

        let resolved = resolve_original_cwd(Some(owd.path().to_path_buf()), Some(current_dir));

        assert_eq!(resolved, owd.path());
    }

    #[test]
    fn resolve_original_cwd_falls_back_to_current_dir_when_owd_is_unset() {
        let current_dir = PathBuf::from("/home/someone/project");

        let resolved = resolve_original_cwd(None, Some(current_dir.clone()));

        assert_eq!(resolved, current_dir);
    }

    #[test]
    fn resolve_original_cwd_falls_back_to_current_dir_when_owd_does_not_exist_on_disk() {
        let owd = PathBuf::from("/no/such/directory/should/ever/exist");
        let current_dir = PathBuf::from("/home/someone/project");

        let resolved = resolve_original_cwd(Some(owd), Some(current_dir.clone()));

        assert_eq!(resolved, current_dir);
    }

    #[test]
    fn should_relaunch_only_when_appimage_not_yet_detached_and_attached_to_a_terminal() {
        assert!(should_relaunch(true, false, true));
        assert!(!should_relaunch(false, false, true), "not an AppImage");
        assert!(!should_relaunch(true, true, true), "already detached once");
        assert!(
            !should_relaunch(true, false, false),
            "no controlling terminal to release"
        );
    }
}
