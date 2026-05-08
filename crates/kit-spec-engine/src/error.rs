use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, SpecEngineError>;

#[derive(Debug, Error)]
pub enum SpecEngineError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("session-store error: {0}")]
    SessionStore(#[from] kit_session_store::SessionStoreError),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("invalid module slug '{slug}'; expected ^[a-z0-9][a-z0-9-]*$")]
    InvalidModuleSlug { slug: String },

    #[error("parallel.yaml is missing required field 'version'")]
    MissingParallelVersion,

    #[error("parallel.yaml requires version: 1, found {found}")]
    UnsupportedParallelVersion { found: String },

    #[error("parallel.yaml schema validation failed: {messages}")]
    ParallelSchemaValidation { messages: String },

    #[error("parallel.yaml parse error: {message}")]
    ParallelParse { message: String },

    #[error("required spec file is missing: {path}")]
    MissingSpecFile { path: PathBuf },

    #[error("atomic write rollback failed after '{operation}': {source}")]
    AtomicRollback {
        operation: String,
        #[source]
        source: std::io::Error,
    },
}
