use serde::{Deserialize, Serialize};

pub use kit_session_store::SpecSnapshot;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectBrief {
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MasterBlueprint {
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roadmap {
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParallelYaml {
    pub version: u32,
    #[serde(default)]
    pub touches: Vec<String>,
    #[serde(default)]
    pub shared: Vec<String>,
    #[serde(default)]
    pub ports: Vec<u16>,
    #[serde(default)]
    pub migrations: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleSpec {
    pub name: String,
    pub spec_md: String,
    pub claude_md: String,
    pub parallel_yaml: Option<ParallelYaml>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleEntry {
    pub name: String,
    pub depends_on: Vec<String>,
    pub layer: String,
}
