use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not found")]
    NotFound,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("{0}")]
    Ai(String),

    /// The assistant could not reach the network/provider. This is deliberately
    /// non-fatal: the rest of the app keeps working without the AI.
    #[error("You're offline or the AI service can't be reached. Everything else still works — try the assistant again once you're back online.")]
    Offline,

    #[error("Network error: {0}")]
    Http(#[from] reqwest::Error),
}

// Tauri sends command errors to the frontend as JSON, so the error type must be Serialize.
// We send just the message string; the frontend shows it as-is.
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
