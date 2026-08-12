use crate::app::skills_service::TagUpdateInput;
use crate::commands::state::AppState;
use crate::domain::{Skill, SkillTag};
use crate::error::AppError;
use serde::Deserialize;
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct CreateTagArgs {
    pub name: String,
    pub color: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTagArgs {
    pub tag_id: String,
    pub name: Option<String>,
    pub color: Option<String>,
}

#[tauri::command]
pub fn get_tags(state: State<'_, AppState>) -> Result<Vec<SkillTag>, AppError> {
    Ok(state.services.skills.load_state()?.tags)
}
#[tauri::command]
pub fn create_tag(state: State<'_, AppState>, args: CreateTagArgs) -> Result<SkillTag, AppError> {
    state.services.skills.create_tag(args.name, args.color)
}
#[tauri::command]
pub fn update_tag(state: State<'_, AppState>, args: UpdateTagArgs) -> Result<SkillTag, AppError> {
    state.services.skills.update_tag(
        &args.tag_id,
        TagUpdateInput {
            name: args.name,
            color: args.color,
        },
    )
}
#[tauri::command]
pub fn delete_tag(state: State<'_, AppState>, tag_id: String) -> Result<(), AppError> {
    state.services.skills.delete_tag(&tag_id)
}
#[tauri::command]
pub fn reorder_tags(
    state: State<'_, AppState>,
    tag_ids: Vec<String>,
) -> Result<Vec<SkillTag>, AppError> {
    state.services.skills.reorder_tags(tag_ids)
}
#[tauri::command]
pub fn assign_tag_to_skill(
    state: State<'_, AppState>,
    skill_id: String,
    tag_id: String,
) -> Result<Skill, AppError> {
    state
        .services
        .skills
        .set_tag_assignment(&skill_id, &tag_id, true)
}
#[tauri::command]
pub fn unassign_tag_from_skill(
    state: State<'_, AppState>,
    skill_id: String,
    tag_id: String,
) -> Result<Skill, AppError> {
    state
        .services
        .skills
        .set_tag_assignment(&skill_id, &tag_id, false)
}
