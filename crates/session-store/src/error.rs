use thiserror::Error;

pub type Result<T> = std::result::Result<T, SessionStoreError>;

#[derive(Debug, Error)]
pub enum SessionStoreError {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("lock poisoned")]
    LockPoisoned,

    #[error("track for module '{module}' already exists; cleanup the existing track before restarting this module")]
    DuplicateTrackModule { module: String },
}
