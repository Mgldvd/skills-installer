use serde::Deserialize;
use tauri::State;

use crate::app::skills_service::{NewSkillInput, PackImportResult, SkillUpdateInput};
use crate::domain::{ApplicationConfig, ParsedSkillSource, Skill};
use crate::error::AppError;
use crate::skills::SkillUrlParser;

use super::state::AppState;

#[tauri::command]
pub async fn preview_pack_url(
    raw_url: String,
) -> Result<crate::skills::pack_import::PackPreview, AppError> {
    crate::skills::pack_import::discover_pack(&raw_url).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPackArgs {
    pub url: String,
    pub pack_name: String,
    pub color: String,
    pub group_id: String,
}

#[tauri::command]
pub async fn import_pack(
    state: State<'_, AppState>,
    args: ImportPackArgs,
) -> Result<PackImportResult, AppError> {
    let preview = crate::skills::pack_import::discover_pack(&args.url).await?;
    state
        .services
        .skills
        .import_pack(preview, args.pack_name, args.color, args.group_id)
}

/// Thin IPC facade only — every command here does argument extraction plus
/// one call into `SkillsService`; no business logic lives in this module.
#[tauri::command]
pub fn get_application_state(state: State<'_, AppState>) -> Result<ApplicationConfig, AppError> {
    state.services.skills.load_state()
}

#[tauri::command]
pub fn get_skills(state: State<'_, AppState>) -> Result<Vec<Skill>, AppError> {
    Ok(state.services.skills.load_state()?.skills)
}

#[tauri::command]
pub fn get_installed_skills(state: State<'_, AppState>) -> Result<Vec<Skill>, AppError> {
    Ok(state
        .services
        .skills
        .load_state()?
        .skills
        .into_iter()
        .filter(|s| s.installed)
        .collect())
}

#[tauri::command]
pub fn preview_skill_url(raw_url: String) -> Result<ParsedSkillSource, AppError> {
    crate::skills::SkillsShUrlParser.parse(&raw_url)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSkillArgs {
    pub url: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub group_id: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub preselected: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[tauri::command]
pub fn add_skill(state: State<'_, AppState>, args: AddSkillArgs) -> Result<Skill, AppError> {
    state.services.skills.add_skill(NewSkillInput {
        url: args.url,
        display_name: args.display_name,
        description: args.description,
        group_id: args.group_id,
        tags: args.tags,
        preselected: args.preselected,
        enabled: args.enabled,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSkillArgs {
    pub skill_id: String,
    pub url: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub group_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub preselected: Option<bool>,
    pub enabled: Option<bool>,
}

#[tauri::command]
pub fn update_skill(state: State<'_, AppState>, args: UpdateSkillArgs) -> Result<Skill, AppError> {
    state.services.skills.update_skill(
        &args.skill_id,
        SkillUpdateInput {
            url: args.url,
            display_name: args.display_name,
            description: args.description,
            group_id: args.group_id,
            tags: args.tags,
            preselected: args.preselected,
            enabled: args.enabled,
        },
    )
}

#[tauri::command]
pub fn delete_skill(state: State<'_, AppState>, skill_id: String) -> Result<(), AppError> {
    state.services.skills.delete_skill(&skill_id)
}

/// Local-only, on-demand check (see `SkillsService::check_local_updates`):
/// never runs as part of the normal load/refresh path, only when the GUI's
/// "Check for Updates" is clicked. `project_path` follows the same
/// resolution as `refresh`'s.
#[tauri::command]
pub fn check_local_skill_updates(
    state: State<'_, AppState>,
    project_path: Option<String>,
) -> Result<Vec<String>, AppError> {
    state
        .services
        .skills
        .check_local_updates_for(project_path.as_deref())
}

fn default_true() -> bool {
    true
}
