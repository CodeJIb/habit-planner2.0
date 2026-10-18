//! Text-to-speech settings for the JARVIS voice.
//!
//! Speech is produced either from Microsoft's Edge neural voices (the cloud
//! source, synthesised in `services::cloud_tts`) or from the browser's
//! `speechSynthesis` (the system source). Rust only persists the on/off
//! switch, the source and the voice shape so they survive a restart — the
//! same `app_settings` pattern the phone reminders use.

use sqlx::SqlitePool;

use crate::errors::AppError;
use crate::models::tts::{TtsSettings, TtsSettingsInput};
use crate::repositories::settings as settings_repo;
use crate::services::cloud_tts::{validate_voice, DEFAULT_CLOUD_VOICE};

const KEY_ENABLED: &str = "tts_enabled";
const KEY_SOURCE: &str = "tts_source";
const KEY_VOICE: &str = "tts_voice";
const KEY_CLOUD_VOICE: &str = "tts_cloud_voice";
const KEY_RATE: &str = "tts_rate";
const KEY_PITCH: &str = "tts_pitch";

/// Defaults used the first time, or whenever a stored value is unreadable.
/// The cloud source is the default because it is what JARVIS sounds like;
/// the frontend falls back to the system voice when offline.
const DEFAULT_SOURCE: &str = "cloud";
const DEFAULT_RATE: f64 = 1.0;
const DEFAULT_PITCH: f64 = 1.0;

/// Speed the browser can speak at.
const RATE_MIN: f64 = 0.5;
const RATE_MAX: f64 = 2.0;
/// Pitch the browser can speak at.
const PITCH_MIN: f64 = 0.5;
const PITCH_MAX: f64 = 2.0;

/// Voice names come from the OS, so keep the stored value bounded.
const MAX_VOICE: usize = 120;

pub struct TtsService {
    pool: SqlitePool,
}

impl TtsService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Read the stored settings, filling in defaults for anything missing.
    pub async fn get_settings(&self) -> Result<TtsSettings, AppError> {
        let mut conn = self.pool.acquire().await?;

        let enabled = settings_repo::get_setting(&mut conn, KEY_ENABLED)
            .await?
            .as_deref()
            == Some("true");

        let source = parse_source(
            settings_repo::get_setting(&mut conn, KEY_SOURCE)
                .await?
                .as_deref(),
        );

        let voice = settings_repo::get_setting(&mut conn, KEY_VOICE)
            .await?
            .unwrap_or_default();

        let cloud_voice = settings_repo::get_setting(&mut conn, KEY_CLOUD_VOICE)
            .await?
            .map(|value| value.trim().to_string())
            .filter(|value| validate_voice(value).is_ok())
            .unwrap_or_else(|| DEFAULT_CLOUD_VOICE.to_string());

        let rate = clamp_number(
            "rate",
            parse_number(
                settings_repo::get_setting(&mut conn, KEY_RATE).await?.as_deref(),
                DEFAULT_RATE,
            ),
            RATE_MIN,
            RATE_MAX,
        )?;

        let pitch = clamp_number(
            "pitch",
            parse_number(
                settings_repo::get_setting(&mut conn, KEY_PITCH).await?.as_deref(),
                DEFAULT_PITCH,
            ),
            PITCH_MIN,
            PITCH_MAX,
        )?;

        Ok(TtsSettings {
            enabled,
            source,
            voice,
            cloud_voice,
            rate,
            pitch,
        })
    }

    /// Apply only the fields the caller sent, then return the full result.
    pub async fn set_settings(&self, input: TtsSettingsInput) -> Result<TtsSettings, AppError> {
        let mut conn = self.pool.acquire().await?;

        if let Some(enabled) = input.enabled {
            let stored = if enabled { "true" } else { "false" };
            settings_repo::set_setting(&mut conn, KEY_ENABLED, stored).await?;
        }

        if let Some(source) = input.source {
            let source = source.trim();
            if !matches!(source, "cloud" | "system") {
                return Err(AppError::InvalidInput(
                    "Voice source must be \"cloud\" or \"system\".".into(),
                ));
            }
            settings_repo::set_setting(&mut conn, KEY_SOURCE, source).await?;
        }

        if let Some(cloud_voice) = input.cloud_voice {
            let cloud_voice = cloud_voice.trim();
            validate_voice(cloud_voice)?;
            settings_repo::set_setting(&mut conn, KEY_CLOUD_VOICE, cloud_voice).await?;
        }

        if let Some(voice) = input.voice {
            let voice = voice.trim();
            if voice.chars().count() > MAX_VOICE {
                return Err(AppError::InvalidInput(format!(
                    "A voice name must be {MAX_VOICE} characters or fewer."
                )));
            }
            // An empty name means "auto-pick", so the empty string is stored as-is.
            settings_repo::set_setting(&mut conn, KEY_VOICE, voice).await?;
        }

        if let Some(rate) = input.rate {
            let rate = clamp_number("rate", rate, RATE_MIN, RATE_MAX)?;
            settings_repo::set_setting(&mut conn, KEY_RATE, &format!("{rate:.2}")).await?;
        }

        if let Some(pitch) = input.pitch {
            let pitch = clamp_number("pitch", pitch, PITCH_MIN, PITCH_MAX)?;
            settings_repo::set_setting(&mut conn, KEY_PITCH, &format!("{pitch:.2}")).await?;
        }

        drop(conn);
        self.get_settings().await
    }
}

/// Parse a stored number, falling back to `default` when missing or unreadable.
fn parse_number(raw: Option<&str>, default: f64) -> f64 {
    raw.and_then(|s| s.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(default)
}

/// Read the stored speech source; anything unexpected means "use the default".
fn parse_source(raw: Option<&str>) -> String {
    match raw.map(str::trim) {
        Some("system") => "system".to_string(),
        Some("cloud") => "cloud".to_string(),
        _ => DEFAULT_SOURCE.to_string(),
    }
}

/// Reject non-numeric input, then clamp into range.
fn clamp_number(name: &str, value: f64, min: f64, max: f64) -> Result<f64, AppError> {
    if !value.is_finite() {
        return Err(AppError::InvalidInput(format!(
            "The {name} must be a number."
        )));
    }
    Ok(value.clamp(min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_values_fall_back_to_defaults() {
        assert_eq!(parse_number(None, DEFAULT_RATE), 1.0);
        assert_eq!(parse_number(Some(""), DEFAULT_RATE), 1.0);
        assert_eq!(parse_number(Some("not a number"), DEFAULT_PITCH), 1.0);
        assert_eq!(parse_number(Some("nan"), DEFAULT_RATE), 1.0);
        assert_eq!(parse_number(Some("1.25"), DEFAULT_RATE), 1.25);
    }

    #[test]
    fn values_are_clamped_into_range() {
        assert_eq!(clamp_number("rate", 0.1, RATE_MIN, RATE_MAX).unwrap(), RATE_MIN);
        assert_eq!(clamp_number("rate", 9.9, RATE_MIN, RATE_MAX).unwrap(), RATE_MAX);
        assert_eq!(clamp_number("pitch", 1.3, PITCH_MIN, PITCH_MAX).unwrap(), 1.3);
    }

    #[test]
    fn non_finite_values_are_rejected() {
        let err = clamp_number("rate", f64::NAN, RATE_MIN, RATE_MAX).unwrap_err();
        assert!(err.to_string().contains("rate"));
    }

    #[test]
    fn unknown_sources_fall_back_to_the_default() {
        assert_eq!(parse_source(None), "cloud");
        assert_eq!(parse_source(Some("system")), "system");
        assert_eq!(parse_source(Some(" cloud ")), "cloud");
        assert_eq!(parse_source(Some("anything else")), "cloud");
    }

    #[test]
    fn missing_values_fall_back_to_a_sane_voice_shape() {
        // A settings row with nothing stored yields the cloud source, the
        // British default voice and neutral speed/pitch.
        let settings = TtsSettings {
            enabled: false,
            source: parse_source(None),
            voice: String::new(),
            cloud_voice: DEFAULT_CLOUD_VOICE.to_string(),
            rate: DEFAULT_RATE,
            pitch: DEFAULT_PITCH,
        };
        assert_eq!(settings.source, "cloud");
        assert_eq!(settings.cloud_voice, "en-GB-RyanNeural");
        assert_eq!(settings.voice, "");
        assert_eq!(settings.rate, 1.0);
        assert_eq!(settings.pitch, 1.0);
    }
}
