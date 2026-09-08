use serde::Deserialize;
use tauri::State;

use crate::commands::state::AppState;
use crate::domain::Preset;
use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePresetArgs {
    pub name: String,
    pub git_url: Option<String>,
    pub skill_names: Vec<String>,
}

#[tauri::command]
pub fn get_presets(state: State<'_, AppState>) -> Result<Vec<Preset>, AppError> {
    state.services.presets.list()
}

#[tauri::command]
pub fn save_preset(state: State<'_, AppState>, args: SavePresetArgs) -> Result<Preset, AppError> {
    state
        .services
        .presets
        .save(args.name, args.git_url, args.skill_names)
}

#[tauri::command]
pub fn update_preset(
    state: State<'_, AppState>,
    preset_id: String,
    skill_names: Vec<String>,
) -> Result<Preset, AppError> {
    state.services.presets.update(&preset_id, skill_names)
}

#[tauri::command]
pub fn delete_preset(state: State<'_, AppState>, preset_id: String) -> Result<(), AppError> {
    state.services.presets.delete(&preset_id)
}
