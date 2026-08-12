use crate::app::ApplicationServices;

/// Managed Tauri state. `ApplicationServices` is `Send + Sync` on its own
/// (its only interior mutability is the `Mutex` inside `InstallationService`),
/// so this needs no additional locking — Tauri's `State<'_, AppState>` just
/// hands out shared references to the one instance created in `lib::run`.
pub struct AppState {
    pub services: ApplicationServices,
}
