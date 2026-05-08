use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CodeMap {
    pub path: PathBuf,
    pub language: Language,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub items: Vec<Item>,
    pub stats: CodeMapStats,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub module: String,
    pub items: Vec<String>,
    pub line: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Export {
    pub name: String,
    pub kind: String,
    pub line: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Item {
    Class {
        name: String,
        methods: Vec<Signature>,
        fields: Vec<Signature>,
        doc: Option<String>,
    },
    Function(Signature),
    Type {
        name: String,
        definition: String,
    },
    Constant {
        name: String,
        type_hint: Option<String>,
        doc: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<String>,
    pub doc: Option<String>,
    pub line: u32,
    pub is_async: bool,
    pub is_exported: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub type_hint: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Ruby,
    Unsupported,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
pub struct CodeMapStats {
    pub line_count: usize,
    pub char_count: usize,
    pub token_estimate: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ScanOpts {
    pub max_file_bytes: usize,
    pub include_hidden: bool,
}

impl Default for ScanOpts {
    fn default() -> Self {
        Self {
            max_file_bytes: 1_000_000,
            include_hidden: false,
        }
    }
}
