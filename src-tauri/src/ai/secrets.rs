//! Secure storage for the AI credential.
//!
//! The API key (or proxy access token) must never sit in the SQLite file or the
//! frontend. It lives in the OS-protected store via `tauri-plugin-secure-keystore`
//! (Android Keystore, iOS Keychain, Windows Credential Manager, macOS Keychain,
//! Linux Secret Service).
//!
//! The store is reached through the Tauri `AppHandle`, which the service only
//! has when running as a real app — unit tests build the service without one and
//! fall back to the legacy DB path (see `services::ai`).

use tauri_plugin_secure_keystore::{ItemKey, SecureKeystoreExt, SetItemRequest};

use crate::errors::AppError;

/// Key under which the single AI credential is stored.
pub const CREDENTIAL_KEY: &str = "ai_credential";

/// Save (or overwrite) the credential. An empty value clears it.
pub fn store(app: &tauri::AppHandle, value: &str) -> Result<(), AppError> {
    if value.trim().is_empty() {
        return clear(app);
    }
    app.secure_keystore()
        .set_item(SetItemRequest {
            key: CREDENTIAL_KEY.to_string(),
            value: value.to_string(),
        })
        .map_err(|error| {
            AppError::Ai(format!(
                "Could not save the key to secure storage: {error}"
            ))
        })
}

/// Load the stored credential, if any.
pub fn load(app: &tauri::AppHandle) -> Result<Option<String>, AppError> {
    let response = app
        .secure_keystore()
        .get_item(ItemKey {
            key: CREDENTIAL_KEY.to_string(),
        })
        .map_err(|error| {
            AppError::Ai(format!(
                "Could not read the key from secure storage: {error}"
            ))
        })?;
    Ok(response.value.filter(|value| !value.trim().is_empty()))
}

/// Remove the stored credential.
pub fn clear(app: &tauri::AppHandle) -> Result<(), AppError> {
    app.secure_keystore()
        .delete_item(ItemKey {
            key: CREDENTIAL_KEY.to_string(),
        })
        .map_err(|error| {
            AppError::Ai(format!(
                "Could not remove the key from secure storage: {error}"
            ))
        })
}
