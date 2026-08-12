use crate::commands::state::AppState;
use crate::domain::{ApplicationConfig, UiPreferences};
use crate::error::AppError;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PortableConfiguration {
    version: u32,
    catalog: ApplicationConfig,
    preferences: UiPreferences,
}

#[tauri::command]
pub fn export_portable_configuration(state: State<'_, AppState>) -> Result<String, AppError> {
    let export = PortableConfiguration {
        version: 1,
        catalog: state.services.skills.configured_state()?,
        preferences: state.services.preferences.load()?,
    };
    serde_json::to_string_pretty(&export)
        .map_err(|e| AppError::Config(format!("failed to serialize export: {e}")))
}

#[tauri::command]
pub fn import_portable_configuration(
    state: State<'_, AppState>,
    content: String,
) -> Result<(), AppError> {
    let imported: PortableConfiguration = serde_json::from_str(&content)
        .map_err(|e| AppError::Config(format!("invalid configuration export: {e}")))?;
    if imported.version != 1 {
        return Err(AppError::Validation(format!(
            "unsupported export version {}",
            imported.version
        )));
    }
    state.services.preferences.validate(&imported.preferences)?;
    let previous = state.services.skills.configured_state()?;
    state.services.skills.replace_config(imported.catalog)?;
    if let Err(error) = state.services.preferences.save(&imported.preferences) {
        let _ = state.services.skills.replace_config(previous);
        return Err(error);
    }
    Ok(())
}
