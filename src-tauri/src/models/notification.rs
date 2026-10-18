use serde::{Deserialize, Serialize};

/// Phone-reminder settings as the UI may see them. The topic is shown back so
/// the user can check it against the ntfy app on their phone.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub enabled: bool,
    pub topic: String,
    /// Empty means the public ntfy.sh server.
    pub server: String,
}

/// Fields a null leaves untouched; an empty topic disables delivery.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettingsInput {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub server: Option<String>,
}

/// What a manual "check now" reports back.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderResult {
    pub sent: usize,
}
