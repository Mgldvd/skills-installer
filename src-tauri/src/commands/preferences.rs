use std::path::PathBuf;

use serde::Serialize;
use tauri::State;

use crate::domain::UiPreferences;
use crate::error::AppError;

use super::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliInstallResult {
    pub command_path: PathBuf,
    pub executable_path: PathBuf,
    pub path_configured: bool,
}

#[tauri::command]
pub fn get_preferences(state: State<'_, AppState>) -> Result<UiPreferences, AppError> {
    state.services.preferences.load()
}

#[tauri::command]
pub fn update_preferences(
    state: State<'_, AppState>,
    preferences: UiPreferences,
) -> Result<UiPreferences, AppError> {
    let current = state.services.preferences.load()?;
    if preferences.local_source_path != current.local_source_path {
        if let Some(raw) = preferences.local_source_path.as_deref() {
            let path = expand_user_path(raw);
            if !path.is_dir() {
                return Err(AppError::Validation(format!(
                    "Local Skill Source is not an available directory: {}",
                    path.display()
                )));
            }
        }
    }
    state.services.preferences.save(&preferences)?;
    Ok(preferences)
}

#[tauri::command]
pub fn install_cli_command() -> Result<CliInstallResult, AppError> {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| AppError::Io("HOME is not available".into()))?;
        let executable = app_executable_path()?;
        install_linux_cli(&home, &executable, std::env::var_os("PATH").as_deref())
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err(AppError::Validation(
            "installing the skills launcher is currently supported on Linux only".into(),
        ))
    }
}

#[cfg(target_os = "linux")]
fn app_executable_path() -> Result<PathBuf, AppError> {
    // `current_exe` points into `/tmp/.mount_*` for an AppImage. APPIMAGE is
    // the persistent file selected by the user and is therefore the correct
    // symlink target.
    let path = std::env::var_os("APPIMAGE")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or(std::env::current_exe()?);
    if !path.is_file() {
        return Err(AppError::Io(format!(
            "application executable is not available at {}",
            path.display()
        )));
    }
    Ok(path.canonicalize().unwrap_or(path))
}

#[cfg(target_os = "linux")]
fn install_linux_cli(
    home: &std::path::Path,
    executable: &std::path::Path,
    path_env: Option<&std::ffi::OsStr>,
) -> Result<CliInstallResult, AppError> {
    use std::os::unix::fs::symlink;

    let bin_dir = home.join(".local/bin");
    std::fs::create_dir_all(&bin_dir)?;
    // Named "skills-control-deck", not the shorter "skills": the latter is
    // one character off from the Linux system's own `skill`/`snice`
    // (procps) commands, which was confusing enough in practice to drop.
    let command_path = bin_dir.join("skills-control-deck");

    match std::fs::symlink_metadata(&command_path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            if std::fs::read_link(&command_path)? != executable {
                std::fs::remove_file(&command_path)?;
                symlink(executable, &command_path)?;
            }
        }
        Ok(_) => {
            return Err(AppError::Io(format!(
                "{} already exists and was not created by Skills Control Deck",
                command_path.display()
            )));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            symlink(executable, &command_path)?;
        }
        Err(error) => return Err(error.into()),
    }

    let path_configured = path_env
        .map(std::env::split_paths)
        .is_some_and(|mut paths| paths.any(|entry| entry == bin_dir));

    Ok(CliInstallResult {
        command_path,
        executable_path: executable.to_path_buf(),
        path_configured,
    })
}

fn expand_user_path(raw: &str) -> PathBuf {
    if raw == "~" {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(raw));
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(raw)
}

#[cfg(all(test, target_os = "linux"))]
mod cli_tests {
    use super::install_linux_cli;

    #[test]
    fn installs_skills_control_deck_symlink_in_user_local_bin() {
        let temp = tempfile::tempdir().unwrap();
        let executable = temp.path().join("Skills.AppImage");
        std::fs::write(&executable, "binary").unwrap();
        let bin = temp.path().join(".local/bin");
        let path = std::env::join_paths([bin.as_path()]).unwrap();

        let result = install_linux_cli(temp.path(), &executable, Some(&path)).unwrap();

        assert_eq!(std::fs::read_link(result.command_path).unwrap(), executable);
        assert!(result.path_configured);
    }

    #[test]
    fn refuses_to_overwrite_an_existing_regular_command() {
        let temp = tempfile::tempdir().unwrap();
        let executable = temp.path().join("app");
        std::fs::write(&executable, "binary").unwrap();
        let command = temp.path().join(".local/bin/skills-control-deck");
        std::fs::create_dir_all(command.parent().unwrap()).unwrap();
        std::fs::write(&command, "user command").unwrap();

        let error = install_linux_cli(temp.path(), &executable, None).unwrap_err();
        assert!(error.to_string().contains("already exists"));
    }
}
