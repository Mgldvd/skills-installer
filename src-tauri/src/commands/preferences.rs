use std::path::PathBuf;
use tauri::State;

use crate::domain::UiPreferences;
use crate::error::AppError;

use super::state::AppState;

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
