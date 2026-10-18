use serde::{Deserialize, Serialize};

/// JARVIS text-to-speech settings as the UI may see them.
///
/// Rust only stores the switch, the chosen source and the voice shape — the
/// speech itself is produced by the frontend: either from the Edge cloud
/// voices (synthesised here) or the browser's `speechSynthesis`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsSettings {
    pub enabled: bool,
    /// Where speech is produced: "cloud" (Edge neural voices, needs internet)
    /// or "system" (the browser's speechSynthesis, offline).
    pub source: String,
    /// Voice name as reported by the OS. Empty means "auto-pick a voice".
    pub voice: String,
    /// Edge voice short name (e.g. "en-GB-RyanNeural") when source is "cloud".
    pub cloud_voice: String,
    /// Speaking speed; 1.0 is normal. Clamped to 0.5..=2.0.
    pub rate: f64,
    /// Voice pitch; 1.0 is normal. Clamped to 0.5..=2.0. System voice only.
    pub pitch: f64,
}

/// Fields a null leaves untouched, so the UI can send only what changed.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsSettingsInput {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub source: Option<String>,
    /// Omitted leaves it unchanged; an empty string means "auto-pick".
    #[serde(default)]
    pub voice: Option<String>,
    #[serde(default)]
    pub cloud_voice: Option<String>,
    #[serde(default)]
    pub rate: Option<f64>,
    #[serde(default)]
    pub pitch: Option<f64>,
}

/// One voice from the Edge cloud catalogue, for the voice picker.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudVoice {
    /// Short name sent back as the synthesis voice, e.g. "en-GB-RyanNeural".
    pub id: String,
    /// Dropdown text, e.g. "Ryan (Male, en-GB)".
    pub label: String,
    /// BCP-47 locale, used to sort British voices to the top.
    pub locale: String,
}
