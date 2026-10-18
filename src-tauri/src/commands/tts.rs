use tauri::State;

use crate::errors::AppError;
use crate::models::tts::{CloudVoice, TtsSettings, TtsSettingsInput};
use crate::services::tts::TtsService;
use crate::state::AppState;

#[tauri::command]
pub async fn get_tts_settings(state: State<'_, AppState>) -> Result<TtsSettings, AppError> {
    TtsService::new(state.db.clone()).get_settings().await
}

#[tauri::command]
pub async fn set_tts_settings(
    state: State<'_, AppState>,
    input: TtsSettingsInput,
) -> Result<TtsSettings, AppError> {
    TtsService::new(state.db.clone())
        .set_settings(input)
        .await
}

/// Synthesise one spoken line with the cloud voice; returns MP3 bytes for the
/// frontend audio queue to play. The browser cannot fetch this endpoint
/// directly, so it goes through Rust.
#[tauri::command]
pub async fn synth_speech(
    text: String,
    voice: String,
    rate: f64,
) -> Result<tauri::ipc::Response, AppError> {
    let audio = crate::services::cloud_tts::synthesize(&text, &voice, rate).await?;
    Ok(tauri::ipc::Response::new(audio))
}

/// The Edge voice catalogue for the picker. Falls back to a built-in English
/// list when offline, so it never fails.
#[tauri::command]
pub async fn list_cloud_voices() -> Vec<CloudVoice> {
    crate::services::cloud_tts::list_voices().await
}

/// Open the OS speech settings, where another voice (a British one for JARVIS,
/// say) can be installed. Shown next to the voice picker when none matches.
#[tauri::command]
pub async fn open_speech_settings() -> Result<(), AppError> {
    if cfg!(target_os = "windows") {
        std::process::Command::new("explorer")
            .arg("ms-settings:speech")
            .spawn()?;
        Ok(())
    } else {
        Err(AppError::InvalidInput(
            "Install a new voice from your system's language settings.".into(),
        ))
    }
}
