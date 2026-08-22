use tauri::ipc::Channel;
use tauri::State;

use crate::domain::{
    ApplicationConfig, DependencyStatus, InstallProgressEvent, InstallRequest, InstallResult,
};
use crate::error::AppError;

use super::state::AppState;

#[tauri::command]
pub async fn get_dependency_status(
    state: State<'_, AppState>,
) -> Result<DependencyStatus, AppError> {
    state.services.installation.check_dependencies().await
}

#[tauri::command]
pub fn validate_installation(
    state: State<'_, AppState>,
    request: InstallRequest,
) -> Result<(), AppError> {
    let available = state.services.skills.load_state()?.skills;
    state
        .services
        .installation
        .validate_installation(&request, &available)
}

/// `on_event` is a `tauri::ipc::Channel` scoped to this single invocation —
/// the correct v2 primitive for high-frequency progress streaming, as
/// opposed to a global `emit`/`listen` event pair that every window would
/// receive regardless of which install triggered it.
#[tauri::command]
pub async fn install_skills(
    state: State<'_, AppState>,
    request: InstallRequest,
    on_event: Channel<InstallProgressEvent>,
) -> Result<InstallResult, AppError> {
    let available = state.services.skills.load_state()?.skills;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<InstallProgressEvent>();

    let forward = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = on_event.send(event);
        }
    });

    let result = state
        .services
        .installation
        .install(request, &available, tx)
        .await;
    let _ = forward.await;
    result
}

#[tauri::command]
pub fn cancel_installation(state: State<'_, AppState>) -> Result<(), AppError> {
    state.services.installation.cancel();
    Ok(())
}

/// `project_path` is the folder currently selected in the GUI header, so
/// the "Installed" badge is recomputed against wherever the user is
/// actually pointed right now rather than the app's launch directory. An
/// empty/whitespace-only value (or `None`, from the CLI which has no such
/// concept) falls back to that launch directory — see `SkillsService`.
#[tauri::command]
pub fn refresh(
    state: State<'_, AppState>,
    project_path: Option<String>,
) -> Result<ApplicationConfig, AppError> {
    match project_path
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(path) => state
            .services
            .skills
            .load_state_for(std::path::Path::new(path)),
        None => state.services.skills.load_state(),
    }
}
