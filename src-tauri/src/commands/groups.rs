use serde::Deserialize;
use tauri::State;

use crate::app::skills_service::{DeleteGroupStrategy, GroupUpdateInput, NewGroupInput};
use crate::domain::SkillGroup;
use crate::error::AppError;

use super::state::AppState;

#[tauri::command]
pub fn get_groups(state: State<'_, AppState>) -> Result<Vec<SkillGroup>, AppError> {
    Ok(state.services.skills.load_state()?.groups)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupArgs {
    pub name: String,
    pub color: String,
}

#[tauri::command]
pub fn create_group(
    state: State<'_, AppState>,
    args: CreateGroupArgs,
) -> Result<SkillGroup, AppError> {
    state.services.skills.create_group(NewGroupInput {
        name: args.name,
        color: args.color,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupArgs {
    pub group_id: String,
    pub name: Option<String>,
    pub color: Option<String>,
    pub order: Option<i32>,
    pub enabled: Option<bool>,
}

#[tauri::command]
pub fn update_group(
    state: State<'_, AppState>,
    args: UpdateGroupArgs,
) -> Result<SkillGroup, AppError> {
    state.services.skills.update_group(
        &args.group_id,
        GroupUpdateInput {
            name: args.name,
            color: args.color,
            order: args.order,
            enabled: args.enabled,
        },
    )
}

/// Mirrors `DeleteGroupStrategy`, but as a plain serde-friendly shape for
/// the IPC boundary (an internal enum with a `String` payload on one
/// variant round-trips more predictably as `{ "strategy": "...", "targetGroupId": "..." }`
/// than relying on serde's default externally-tagged enum representation).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteGroupArgs {
    pub group_id: String,
    pub strategy: DeleteGroupStrategyArg,
    #[serde(default)]
    pub target_group_id: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DeleteGroupStrategyArg {
    RequireEmpty,
    MoveToOther,
    MoveTo,
}

#[tauri::command]
pub fn delete_group(state: State<'_, AppState>, args: DeleteGroupArgs) -> Result<(), AppError> {
    let strategy = match args.strategy {
        DeleteGroupStrategyArg::RequireEmpty => DeleteGroupStrategy::RequireEmpty,
        DeleteGroupStrategyArg::MoveToOther => DeleteGroupStrategy::MoveToOther,
        DeleteGroupStrategyArg::MoveTo => {
            let target = args.target_group_id.ok_or_else(|| {
                AppError::Validation(
                    "targetGroupId is required for the moveTo strategy".to_string(),
                )
            })?;
            DeleteGroupStrategy::MoveTo(target)
        }
    };
    state.services.skills.delete_group(&args.group_id, strategy)
}
