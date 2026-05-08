use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, CodeMapError>;

#[derive(Debug, Error)]
pub enum CodeMapError {
    #[error("io error while reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("tree-sitter language error: {0}")]
    Language(#[from] tree_sitter::LanguageError),

    #[error("tree-sitter failed to parse {path}")]
    ParseFailed { path: PathBuf },

    #[error("unsupported language for {path}")]
    UnsupportedLanguage { path: PathBuf },

    #[error("directory walk error at {path}: {source}")]
    WalkDir {
        path: PathBuf,
        #[source]
        source: walkdir::Error,
    },
}
