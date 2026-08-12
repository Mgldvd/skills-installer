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
        let config_override = cli_args.config.clone();
        run_gui(config_override);
    } else {
        let runtime = tokio::runtime::Runtime::new().expect("failed to start async runtime");
        let exit_code = runtime.block_on(cli::execute(cli_args));
        std::process::exit(exit_code);
    }
}

// No `#[cfg_attr(mobile, tauri::mobile_entry_point)]` here: this app
// targets Linux desktop only (AppImage/.deb), never mobile, so the
// main.rs/lib.rs split exists purely for module organization, not to
// satisfy a mobile entry-point requirement.
fn run_gui(config_override: Option<PathBuf>) {
    // Preserve the launch context: starting the app from a project must
    // install into that project, never silently redirect to HOME.
    let mut project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // AppImage launchers commonly set cwd inside their ephemeral
    // `/tmp/.mount_*` extraction tree. That is never a meaningful project
    // destination and disappears when the process exits.
    if project_root.to_string_lossy().starts_with("/tmp/.mount_") {
        if let Some(home) = std::env::var_os("HOME") {
            project_root = PathBuf::from(home);
        }
    }
    let services = app::ApplicationServices::new(config_override, project_root);
    let state = commands::AppState { services };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::skills::get_application_state,
            commands::skills::get_skills,
            commands::skills::get_installed_skills,
            commands::skills::preview_skill_url,
            commands::skills::add_skill,
            commands::skills::update_skill,
            commands::skills::delete_skill,
            commands::tags::get_tags,
            commands::tags::create_tag,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::assign_tag_to_skill,
            commands::tags::unassign_tag_from_skill,
            commands::configuration::export_portable_configuration,
            commands::configuration::import_portable_configuration,
            commands::preferences::get_preferences,
            commands::preferences::update_preferences,
            commands::installation::get_dependency_status,
            commands::installation::validate_installation,
            commands::installation::install_skills,
            commands::installation::cancel_installation,
            commands::installation::refresh,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
