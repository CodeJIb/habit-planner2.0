use sqlx::SqlitePool;

/// Shared application state, registered with `app.manage(...)` and
/// injected into commands as `tauri::State<'_, AppState>`.
/// `SqlitePool` is cheap to clone (it is a handle to the shared pool).
pub struct AppState {
    pub db: SqlitePool,
    /// Handle used to push events (e.g. streamed assistant text) to the UI.
    pub app: tauri::AppHandle,
}
