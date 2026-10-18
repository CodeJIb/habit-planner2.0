//! The JARVIS cloud voice: Microsoft Edge's neural read-aloud voices.
//!
//! The browser's `speechSynthesis` can only use voices installed on the device,
//! and Windows ships without a British one. Edge exposes 400+ neural voices
//! (including `en-GB-RyanNeural` and `en-GB-ThomasNeural` — the JARVIS-style
//! British male voices) over its read-aloud endpoint: no API key, no account,
//! free for anyone. Each sentence is synthesised here into MP3 bytes, sent to
//! the frontend and played from an audio queue. When the network or the
//! service is unavailable the frontend falls back to the system voice.

use std::sync::Once;

use edge_tts_rust::{EdgeTtsClient, SpeakOptions, DEFAULT_OUTPUT_FORMAT_MP3};

use crate::errors::AppError;
use crate::models::tts::CloudVoice;

/// Several TLS-capable crates in the dependency tree (reqwest, the Edge
/// client) each pull rustls. rustls 0.23 refuses to guess which crypto
/// backend to use when more than one is compiled in, so we pick one —
/// whichever the dependency tree happens to ship — a single time, before the
/// first HTTPS or WebSocket connection.
fn install_crypto_provider() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

/// Default voice: a British male neural voice — the closest free match to the
/// JARVIS brief.
pub const DEFAULT_CLOUD_VOICE: &str = "en-GB-RyanNeural";

/// Longest text sent in one synthesis request. The service caps a single
/// utterance around 4 KB anyway, and spoken sentences are far shorter.
const MAX_TEXT: usize = 3000;

/// Built-in English picker shown when the live voice catalogue cannot be
/// fetched (i.e. when offline): (voice id, label, locale).
const FALLBACK_VOICES: [(&str, &str, &str); 10] = [
    ("en-GB-RyanNeural", "Ryan (Male, en-GB)", "en-GB"),
    ("en-GB-ThomasNeural", "Thomas (Male, en-GB)", "en-GB"),
    ("en-GB-SoniaNeural", "Sonia (Female, en-GB)", "en-GB"),
    ("en-GB-LibbyNeural", "Libby (Female, en-GB)", "en-GB"),
    ("en-US-GuyNeural", "Guy (Male, en-US)", "en-US"),
    ("en-US-JennyNeural", "Jenny (Female, en-US)", "en-US"),
    ("en-US-AriaNeural", "Aria (Female, en-US)", "en-US"),
    ("en-AU-WilliamNeural", "William (Male, en-AU)", "en-AU"),
    ("en-AU-NatashaNeural", "Natasha (Female, en-AU)", "en-AU"),
    ("en-IE-ConnorNeural", "Connor (Male, en-IE)", "en-IE"),
];

/// Turn one spoken line into MP3 bytes. `rate` is the same 0.5..=2.0 multiplier
/// the system voice uses, mapped onto the service's prosody rate percentage.
pub async fn synthesize(text: &str, voice: &str, rate: f64) -> Result<Vec<u8>, AppError> {
    install_crypto_provider();
    validate_voice(voice)?;
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::InvalidInput("There is nothing to say.".into()));
    }
    let text: String = text.chars().take(MAX_TEXT).collect();

    // The crate defaults to a webm/opus stream; MP3 is what the frontend
    // `<audio>` element plays most reliably across platforms.
    let options = SpeakOptions {
        voice: voice.to_string(),
        rate: format!("{:+}%", ((rate - 1.0) * 100.0).round() as i32),
        ..SpeakOptions::default()
    }
    .with_output_format(DEFAULT_OUTPUT_FORMAT_MP3);

    let client = EdgeTtsClient::new()
        .map_err(|e| AppError::Ai(format!("The voice service failed: {e}")))?;
    let result = client
        .synthesize(text, options)
        .await
        .map_err(|e| AppError::Ai(format!("The voice service failed: {e}")))?;

    if result.audio.is_empty() {
        return Err(AppError::Ai(
            "The voice service returned no audio.".into(),
        ));
    }
    Ok(result.audio)
}

/// The Edge voice catalogue for the picker. Never fails: when the catalogue
/// cannot be reached, a built-in English list is returned instead so the
/// dropdown still works offline.
pub async fn list_voices() -> Vec<CloudVoice> {
    install_crypto_provider();
    match EdgeTtsClient::new() {
        Ok(client) => match client.list_voices().await {
            Ok(voices) if !voices.is_empty() => voices
                .iter()
                .map(|voice| CloudVoice {
                    id: voice.short_name.clone(),
                    label: format!(
                        "{} ({}, {})",
                        voice.friendly_name.as_deref().unwrap_or(voice.name.as_str()),
                        voice.gender,
                        voice.locale
                    ),
                    locale: voice.locale.clone(),
                })
                .collect(),
            _ => fallback_voices(),
        },
        Err(_) => fallback_voices(),
    }
}

fn fallback_voices() -> Vec<CloudVoice> {
    FALLBACK_VOICES
        .iter()
        .map(|(id, label, locale)| CloudVoice {
            id: id.to_string(),
            label: label.to_string(),
            locale: locale.to_string(),
        })
        .collect()
}

/// A voice id goes straight into an SSML document, so only a conservative
/// charset is allowed (guards against markup injection through settings).
pub(crate) fn validate_voice(voice: &str) -> Result<(), AppError> {
    let acceptable = !voice.is_empty()
        && voice.len() <= 60
        && voice
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ':'));
    if acceptable {
        Ok(())
    } else {
        Err(AppError::InvalidInput(
            "That voice name is not valid.".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_is_mapped_to_a_signed_percentage() {
        let pct = |rate: f64| format!("{:+}%", ((rate - 1.0) * 100.0).round() as i32);
        assert_eq!(pct(1.0), "+0%");
        assert_eq!(pct(1.25), "+25%");
        assert_eq!(pct(0.75), "-25%");
        assert_eq!(pct(0.5), "-50%");
    }

    #[test]
    fn voice_names_are_conservatively_validated() {
        assert!(validate_voice("en-GB-RyanNeural").is_ok());
        assert!(validate_voice("en-GB-Ollie:DragonHDLatestNeural").is_ok());
        assert!(validate_voice("").is_err());
        assert!(validate_voice("<speak>hi</speak>").is_err());
        assert!(validate_voice("voice with spaces").is_err());
        assert!(&validate_voice(&"a".repeat(61)).is_err());
    }

    #[test]
    fn the_offline_picker_offers_ryan() {
        let voices = fallback_voices();
        assert!(voices.iter().any(|v| v.id == "en-GB-RyanNeural"));
    }

    /// Live check against the real service; run manually with:
    /// `cargo test --manifest-path src-tauri/Cargo.toml -- --ignored`
    #[tokio::test]
    #[ignore = "needs network"]
    async fn synthesises_real_audio() {
        let audio = synthesize("Hello, sir. What can I do for you today?", DEFAULT_CLOUD_VOICE, 1.0)
            .await
            .unwrap();
        assert!(audio.len() > 1000, "expected a real MP3, got {} bytes", audio.len());
        // MP3 frames start with 0xFF 0xE4..0xFF (or an ID3 header).
        let looks_like_mp3 = audio.starts_with(b"ID3")
            || (audio.len() > 1 && audio[0] == 0xFF && audio[1] & 0xE0 == 0xE0);
        assert!(looks_like_mp3, "audio does not look like MP3");
    }

    /// Live check that the voice catalogue arrives (or falls back cleanly).
    #[tokio::test]
    #[ignore = "needs network"]
    async fn lists_voices_with_british_male_options() {
        let voices = list_voices().await;
        assert!(!voices.is_empty());
        assert!(
            voices.iter().any(|v| v.id == "en-GB-RyanNeural"),
            "Ryan should be in the catalogue"
        );
    }
}
