# spec-engine — SPEC

> **Layer:** Core (Rust crate `kit-spec-engine`)
> **Last updated:** 2026-05-08 (Pi-removal pivot)
> **Depends on:** session-store

## 1. Purpose

CRUD + validation over the kit's spec hierarchy: `PROJECT_BRIEF.md`, `RESEARCH.md`, `MODULES.md`, `MASTER_BLUEPRINT.md`, `ROADMAP.md`, and per-module `SPEC.md` / `CLAUDE.md` / `parallel.yaml`. Pure data layer; no agent calls. Replaces the spec-engine sub-domain of the previously-proposed kit-engine Pi extension.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| SE-1 | skill-runner | Load `MASTER_BLUEPRINT.md` and per-module specs to assemble a dispatch prompt | Each skill has structured access to the spec hierarchy |
| SE-2 | track-engine | Read all `parallel.yaml` files + validate `version: 1` | Plan operations are governed by typed contracts |
| SE-3 | gui-shell | Render the spec tree with edit affordance | Users see what specs exist and edit them in-app |
| SE-4 | `/project-spec` skill | Save a new module spec atomically (SPEC + CLAUDE + parallel.yaml in one tx) | A half-written spec set is never visible |
| SE-5 | session-store | Snapshot a spec on every meaningful change | History is queryable without external git tooling |

## 3. Public API

```rust
pub struct SpecEngine { /* Arc<SessionStore> */ }
impl SpecEngine {
    pub fn new(store: Arc<SessionStore>, project_root: PathBuf) -> Self;

    // Top-level specs
    pub fn load_brief(&self) -> Result<ProjectBrief>;
    pub fn save_brief(&self, brief: &ProjectBrief) -> Result<()>;
    pub fn load_blueprint(&self) -> Result<MasterBlueprint>;
    pub fn save_blueprint(&self, b: &MasterBlueprint) -> Result<()>;
    pub fn load_modules(&self) -> Result<Vec<ModuleEntry>>;
    pub fn load_roadmap(&self) -> Result<Roadmap>;

    // Per-module specs
    pub fn load_module_spec(&self, name: &str) -> Result<ModuleSpec>;
    pub fn save_module_spec(&self, s: &ModuleSpec) -> Result<()>;          // atomic: SPEC + CLAUDE + parallel.yaml
    pub fn load_parallel_yaml(&self, name: &str) -> Result<ParallelYaml>;
    pub fn validate_parallel_yaml_str(&self, yaml: &str) -> Result<()>;

    // Snapshots (auto on save)
    pub fn snapshot_history(&self, path: &str) -> Result<Vec<SpecSnapshot>>;
}
```

## 4. Data contracts

```rust
#[derive(Serialize, Deserialize)]
pub struct ParallelYaml {
    pub version: u32,                  // must == 1
    #[serde(default)] pub touches: Vec<String>,
    #[serde(default)] pub shared: Vec<String>,
    #[serde(default)] pub ports: Vec<u16>,
    #[serde(default)] pub migrations: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ModuleSpec {
    pub name: String,                   // kebab-case slug
    pub spec_md: String,                // raw markdown
    pub claude_md: String,
    pub parallel_yaml: Option<ParallelYaml>,
}

#[derive(Serialize, Deserialize)]
pub struct ModuleEntry {
    pub name: String,
    pub depends_on: Vec<String>,
    pub layer: String,                  // free text from MODULES.md
}
```

JSON Schema for `parallel.yaml` bundled at `crates/kit-spec-engine/schemas/parallel.yaml.schema.json`.

## 5. Business rules

- **`parallel.yaml` requires `version: 1`.** Files missing it are rejected with a clear error.
- **Module slug regex:** `^[a-z0-9][a-z0-9-]*$` (kebab-case).
- **Slug-boundary regex** for module references in `MODULES.md`: `(^|[^a-z0-9-])<slug>([^a-z0-9-]|$)` case-insensitive. Prevents "auth" matching "oauth".
- **Atomic module-spec saves:** SPEC.md + CLAUDE.md + parallel.yaml written in one transaction; partial failure rolls back all three.
- **Snapshots on save:** every meaningful change writes a `spec_snapshots` row keyed by `(path, sha256(body))`. Idempotent — same content twice = one row.
- **Path-traversal guard:** module names are validated and concatenated only via `Path::join`; never via string concat.
- **No git operations.** This crate is filesystem-only; track-engine handles git.

## 6. Integration points

| Module | Relationship | Notes |
|---|---|---|
| session-store | Depends on | snapshots persisted; module list cached |
| track-engine | Depends on | reads `parallel.yaml`s for plan validation |
| skill-runner | Depends on | every skill loads relevant specs |
| gui-shell | Depends on | spec tree rendering, in-app editing |
| code-maps | None | Independent |

## 7. Acceptance criteria

- [ ] `cargo test --package kit-spec-engine` passes (unit tests for every CRUD method, schema validation, slug regex, atomic save rollback).
- [ ] Round-trips a `ParallelYaml` fixture without lossy serialisation.
- [ ] Rejects `version: 2` and missing-version files with distinct error messages.
- [ ] Snapshots are deduplicated by content hash.
- [ ] In-memory test fixture (no real filesystem) supports the full API.
- [ ] Rust types are `Send + Sync` (used across Tauri command boundaries).

## 8. Out of scope

- Git operations (branch/commit/merge) — track-engine.
- Markdown parsing / structural extraction beyond schema validation.
- Cross-project spec aggregation (could be added later).

## 9. Open questions

- [ ] **JSON-LD or plain markdown?** Plain markdown for now; revisit if structural extraction becomes valuable.
- [ ] **Frontmatter standard.** SKILL.md has YAML frontmatter; SPEC.md doesn't currently. Should we standardise?
