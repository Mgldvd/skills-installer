use serde::Deserialize;
use tauri::State;

use crate::commands::state::AppState;
use crate::domain::Project;
use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProjectArgs {
    pub name: String,
    pub git_url: Option<String>,
    pub skill_names: Vec<String>,
}

#[tauri::command]
pub fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    state.services.projects.list()
}

#[tauri::command]
pub fn save_project(
    state: State<'_, AppState>,
    args: SaveProjectArgs,
) -> Result<Project, AppError> {
    state
        .services
        .projects
        .save(args.name, args.git_url, args.skill_names)
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, project_id: String) -> Result<(), AppError> {
    state.services.projects.delete(&project_id)
}
