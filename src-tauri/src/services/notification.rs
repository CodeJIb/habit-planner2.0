//! Phone reminders (Phase 6): push due-task alerts to the user's phone through a
//! free ntfy.sh topic, plus an optional daily digest. Everything runs locally;
//! the only network call is the push itself.

use std::time::Duration;

use chrono::Local;
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::errors::AppError;
use crate::models::notification::{
    NotificationSettings, NotificationSettingsInput, ReminderResult,
};
use crate::repositories::notification as repo;
use crate::repositories::settings as settings_repo;
use crate::services::quest::local_today;

/// Public server used when the user does not name their own.
const DEFAULT_SERVER: &str = "https://ntfy.sh";
const KEY_ENABLED: &str = "notify_enabled";
const KEY_TOPIC: &str = "notify_topic";
const KEY_SERVER: &str = "notify_server";
const KEY_LAST_DIGEST: &str = "notify_last_digest_date";

/// ntfy topics are short URL-safe slugs.
const MAX_TOPIC: usize = 64;
const MAX_SERVER: usize = 300;
const MAX_MESSAGE: usize = 1000;
/// Alerts pushed in one tick, so a backlog cannot flood the phone.
const MAX_PER_TICK: i64 = 5;
/// Tasks listed in a daily digest.
const MAX_DIGEST: i64 = 20;
/// How often the background loop looks for newly due tasks.
const TICK: Duration = Duration::from_secs(60);

pub struct NotificationService {
    pool: SqlitePool,
    /// Present in the running app; used to raise native OS notifications.
    /// `None` in unit tests, where there is no window to notify.
    app: Option<AppHandle>,
}

impl NotificationService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool, app: None }
    }

    pub fn with_app(app: AppHandle, pool: SqlitePool) -> Self {
        Self {
            pool,
            app: Some(app),
        }
    }

    // ---- settings ---------------------------------------------------------

    pub async fn get_settings(&self) -> Result<NotificationSettings, AppError> {
        let mut conn = self.pool.acquire().await?;
        let enabled = settings_repo::get_setting(&mut conn, KEY_ENABLED)
            .await?
            .as_deref()
            == Some("true");
        let topic = settings_repo::get_setting(&mut conn, KEY_TOPIC)
            .await?
            .unwrap_or_default();
        let server = settings_repo::get_setting(&mut conn, KEY_SERVER)
            .await?
            .unwrap_or_default();
        Ok(NotificationSettings {
            enabled,
            topic,
            server,
        })
    }

    pub async fn set_settings(
        &self,
        input: NotificationSettingsInput,
    ) -> Result<NotificationSettings, AppError> {
        let mut conn = self.pool.acquire().await?;

        if let Some(topic) = &input.topic {
            let topic = topic.trim();
            if topic.is_empty() {
                settings_repo::delete_setting(&mut conn, KEY_TOPIC).await?;
            } else {
                validate_topic(topic)?;
                settings_repo::set_setting(&mut conn, KEY_TOPIC, topic).await?;
            }
        }

        if let Some(server) = &input.server {
            let server = server.trim();
            if server.is_empty() {
                settings_repo::delete_setting(&mut conn, KEY_SERVER).await?;
            } else {
                validate_server(server)?;
                settings_repo::set_setting(&mut conn, KEY_SERVER, server).await?;
            }
        }

        if let Some(enabled) = input.enabled {
            settings_repo::set_setting(&mut conn, KEY_ENABLED, if enabled { "true" } else { "false" })
                .await?;
        }

        drop(conn);
        self.get_settings().await
    }

    // ---- sending ----------------------------------------------------------

    /// Send a test push so the user can confirm the phone is wired up.
    pub async fn send_test(&self) -> Result<(), AppError> {
        self.push(
            "Habit Planner",
            "Reminders are working. I'll nudge you here when a task is due.",
            "default",
            "white_check_mark",
        )
        .await
    }

    /// Send a notification on the assistant's behalf (the `send_notification` tool).
    pub async fn send_custom(&self, title: Option<&str>, message: &str) -> Result<(), AppError> {
        let message = message.trim();
        if message.is_empty() {
            return Err(AppError::InvalidInput("A notification needs a message.".into()));
        }
        if message.chars().count() > MAX_MESSAGE {
            return Err(AppError::InvalidInput(format!(
                "That notification is too long ({MAX_MESSAGE} characters max)."
            )));
        }
        let title = title.map(str::trim).filter(|t| !t.is_empty());
        self.push(
            title.unwrap_or("JARVIS"),
            message,
            "default",
            "speech_balloon",
        )
        .await
    }

    /// Alert about any open task that has just become due. Returns how many
    /// notifications were sent.
    pub async fn notify_due(&self) -> Result<usize, AppError> {
        let settings = self.get_settings().await?;
        if !should_deliver(&settings) {
            return Ok(0);
        }

        let now = Local::now();
        let now_stamp = now.format("%Y-%m-%dT%H:%M").to_string();
        let today = now.format("%Y-%m-%d").to_string();

        let mut conn = self.pool.acquire().await?;
        let due = repo::due_unreminded(&mut conn, &now_stamp, MAX_PER_TICK).await?;
        drop(conn);

        let mut sent = 0usize;
        for task in due {
            let overdue = task.due_at.get(0..10).is_some_and(|d| d < today.as_str());
            let when = task.due_at.get(0..10).unwrap_or(&task.due_at);
            let body = if overdue {
                format!("\"{}\" was due {when}.", task.title)
            } else {
                format!("\"{}\" is due today.", task.title)
            };
            self.push(
                &truncate(&task.title, 120),
                &body,
                if overdue { "high" } else { "default" },
                if overdue { "warning" } else { "bell" },
            )
            .await?;

            let mut conn = self.pool.acquire().await?;
            repo::mark_reminded(&mut conn, task.id, &now_stamp).await?;
            sent += 1;
        }
        Ok(sent)
    }

    /// Alert about any active habit whose reminder time has arrived today.
    /// Returns how many notifications were sent.
    pub async fn notify_habits(&self) -> Result<usize, AppError> {
        let settings = self.get_settings().await?;
        if !should_deliver(&settings) {
            return Ok(0);
        }

        let now = Local::now();
        let time = now.format("%H:%M").to_string();
        let today = now.format("%Y-%m-%d").to_string();

        let mut conn = self.pool.acquire().await?;
        let due = repo::habits_due(&mut conn, &time, &today, MAX_PER_TICK).await?;
        drop(conn);

        let mut sent = 0usize;
        for habit in due {
            self.push(
                &truncate(&habit.title, 120),
                "Time to check in on this habit.",
                "default",
                "alarm_clock",
            )
            .await?;

            let mut conn = self.pool.acquire().await?;
            repo::mark_habit_reminded(&mut conn, habit.id, &today).await?;
            sent += 1;
        }
        Ok(sent)
    }

    /// One digest per day: everything due today or overdue. No-op if already
    /// sent today or nothing is pending.
    pub async fn maybe_send_digest(&self, today: &str) -> Result<bool, AppError> {
        let settings = self.get_settings().await?;
        if !should_deliver(&settings) {
            return Ok(false);
        }

        let mut conn = self.pool.acquire().await?;
        let last = settings_repo::get_setting(&mut conn, KEY_LAST_DIGEST).await?;
        if last.as_deref() == Some(today) {
            return Ok(false);
        }
        settings_repo::set_setting(&mut conn, KEY_LAST_DIGEST, today).await?;
        let tasks = repo::due_or_overdue(&mut conn, today, MAX_DIGEST).await?;
        drop(conn);

        if tasks.is_empty() {
            return Ok(false);
        }

        let mut body = format!("{} task(s) on the list today:", tasks.len());
        for task in &tasks {
            let when = task.due_at.get(0..10).unwrap_or(&task.due_at);
            body.push_str(&format!("\n- {} (due {})", task.title, when));
        }
        self.push("Your day at a glance", &body, "default", "sunrise")
            .await?;
        Ok(true)
    }

    /// Manually run the due check (the "send now" button / AI tool).
    pub async fn run_due_check(&self) -> Result<ReminderResult, AppError> {
        let tasks = self.notify_due().await?;
        let habits = self.notify_habits().await?;
        Ok(ReminderResult { sent: tasks + habits })
    }

    // ---- the push itself --------------------------------------------------

    async fn push(
        &self,
        title: &str,
        message: &str,
        priority: &str,
        tags: &str,
    ) -> Result<(), AppError> {
        let settings = self.get_settings().await?;
        if !should_deliver(&settings) {
            return Err(AppError::InvalidInput(
                "Phone reminders aren't switched on yet. Turn them on first.".into(),
            ));
        }
        // Native notification on the device running the app (desktop + mobile).
        self.show_local(title, message);
        // Optional push to the phone through ntfy (works when the app is closed).
        if is_configured(&settings) {
            self.push_ntfy(&settings, title, message, priority, tags)
                .await?;
        }
        Ok(())
    }

    fn show_local(&self, title: &str, message: &str) {
        let Some(app) = &self.app else { return };
        let _ = app
            .notification()
            .builder()
            .title(title)
            .body(message)
            .show();
    }

    async fn push_ntfy(
        &self,
        settings: &NotificationSettings,
        title: &str,
        message: &str,
        priority: &str,
        tags: &str,
    ) -> Result<(), AppError> {
        let server = if settings.server.trim().is_empty() {
            DEFAULT_SERVER
        } else {
            settings.server.trim()
        }
        .trim_end_matches('/');
        let url = format!("{server}/{}", settings.topic);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(AppError::Http)?;

        client
            .post(url)
            .header("Title", title)
            .header("Priority", priority)
            .header("Tags", tags)
            .body(message.to_string())
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

fn should_deliver(settings: &NotificationSettings) -> bool {
    settings.enabled
}

/// ntfy requires a topic; the native OS notification works without one.
fn is_configured(settings: &NotificationSettings) -> bool {
    settings.enabled && !settings.topic.trim().is_empty()
}

/// Topics are slugs; reject anything ntfy would treat as a path or query.
fn validate_topic(topic: &str) -> Result<(), AppError> {
    if topic.chars().count() > MAX_TOPIC {
        return Err(AppError::InvalidInput(format!(
            "A topic must be {MAX_TOPIC} characters or fewer."
        )));
    }
    if !topic
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(AppError::InvalidInput(
            "A topic may only contain letters, digits, '-' and '_'.".into(),
        ));
    }
    Ok(())
}

fn validate_server(server: &str) -> Result<(), AppError> {
    if server.len() > MAX_SERVER {
        return Err(AppError::InvalidInput("That server URL is too long.".into()));
    }
    if server.chars().any(char::is_whitespace) {
        return Err(AppError::InvalidInput(
            "The server URL cannot contain spaces.".into(),
        ));
    }
    if !(server.starts_with("http://") || server.starts_with("https://")) {
        return Err(AppError::InvalidInput(
            "The server URL must start with http:// or https://".into(),
        ));
    }
    Ok(())
}

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Background loop: once a minute, send the day's digest (at most once) and any
/// newly due alerts. Runs for the life of the app; errors are swallowed so a
/// flaky network never takes the app down.
pub async fn run_loop(app: tauri::AppHandle, pool: SqlitePool) {
    let service = NotificationService::with_app(app, pool);
    loop {
        let today = local_today().to_string();
        let _ = service.maybe_send_digest(&today).await;
        let _ = service.notify_due().await;
        let _ = service.notify_habits().await;
        tokio::time::sleep(TICK).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topics_must_be_slugs() {
        assert!(validate_topic("my-reminders_01").is_ok());
        assert!(validate_topic("with space").is_err());
        assert!(validate_topic("a/b").is_err());
        assert!(validate_topic("q?x=1").is_err());
        assert!(validate_topic(&"x".repeat(MAX_TOPIC + 1)).is_err());
    }

    #[test]
    fn servers_must_be_http_urls() {
        assert!(validate_server("https://ntfy.sh").is_ok());
        assert!(validate_server("http://192.168.1.10:8080").is_ok());
        assert!(validate_server("ntfy.sh").is_err());
        assert!(validate_server("https://nt fy.sh").is_err());
    }

    #[test]
    fn truncate_keeps_short_text_and_marks_long_text() {
        assert_eq!(truncate("hello", 10), "hello");
        let long = truncate(&"a".repeat(50), 10);
        assert_eq!(long.chars().count(), 10);
        assert!(long.ends_with('…'));
    }

    #[test]
    fn only_an_enabled_topic_counts_as_configured() {
        let base = NotificationSettings {
            enabled: true,
            topic: "t".into(),
            server: String::new(),
        };
        assert!(is_configured(&base));
        assert!(!is_configured(&NotificationSettings {
            enabled: false,
            ..base.clone()
        }));
        assert!(!is_configured(&NotificationSettings {
            topic: "  ".into(),
            ..base
        }));
    }
}
