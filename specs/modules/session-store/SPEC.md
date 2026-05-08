# session-store — SPEC

> **Status:** Phase 5 of /project-init
> **Layer:** Engine (Rust crate)
> **Last updated:** 2026-05-08

## 1. Purpose

A typed, durable, queryable store for **tracks**, **track events**, **learnings (FTS5-indexed)**, and **spec snapshots**. Replaces the bash kit's `tracks.json` + `LEARNINGS.md` + filesystem-grep model with proper SQL semantics, transactions, and full-text search. Per-project DB at `<project>/.kit-workflow-app/state.db`; optional global DB at `~/.kit-workflow-app/global.db` for cross-project queries.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| SS-1 | Track engine | Atomically append a track row + merge-order entry | Two parallel starts can't corrupt the registry |
| SS-2 | GUI plan board | Subscribe to track-status changes | The dashboard updates live without polling files |
| SS-3 | Compound-learning skill | Append a learning fragment with module + commit + tags | Future sessions can FTS5-search learnings by topic |
| SS-4 | `/project-status` skill | Query "all open questions across all tracks" in one round trip | Status dashboards aren't constructed by 50 file reads |
| SS-5 | Cross-project query | Find learnings tagged `tauri-2` across all my projects | I don't repeat mistakes I already made elsewhere |

## 3. Data model

```sql
-- versioned via schema_version
CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);

CREATE TABLE tracks (
  id TEXT PRIMARY KEY,                  -- "track-<module>-<ts>"
  module TEXT NOT NULL,
  branch TEXT NOT NULL,
  worktree TEXT NOT NULL,
  harness TEXT NOT NULL,                -- 'codex'|'claude'
  port INTEGER,
  status TEXT NOT NULL,                 -- 'pending'|'running'|'completed'|'timeout'|'aborted'
  started_at INTEGER NOT NULL,
  completed_at INTEGER,
  last_commit TEXT,
  pid INTEGER,
  UNIQUE(module)                        -- enforces dedupe (CodeRabbit finding from kit retro)
);

CREATE TABLE track_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  track_id TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  event_type TEXT NOT NULL,             -- 'status_change'|'commit'|'log_line'|'sentinel'
  payload TEXT NOT NULL,                -- JSON
  created_at INTEGER NOT NULL
);

CREATE TABLE learnings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  module TEXT,                          -- nullable: cross-module learnings
  project_name TEXT NOT NULL,
  body TEXT NOT NULL,                   -- markdown
  tags TEXT NOT NULL,                   -- JSON array of strings
  commit_hash TEXT,
  created_at INTEGER NOT NULL
);
CREATE VIRTUAL TABLE learnings_fts USING fts5(body, content='learnings', content_rowid='id');
-- triggers maintain fts in sync with learnings

CREATE TABLE spec_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  spec_path TEXT NOT NULL,              -- 'specs/MASTER_BLUEPRINT.md'
  content_hash TEXT NOT NULL,           -- sha256 of body
  body TEXT NOT NULL,
  taken_at INTEGER NOT NULL,
  UNIQUE(spec_path, content_hash)
);
```

WAL mode + busy_timeout 5000ms.

## 4. Public API (Rust)

```rust
pub struct SessionStore { /* Mutex<Connection> */ }
impl SessionStore {
    pub fn open(path: &Path) -> Result<Self>;
    pub fn migrate(&self) -> Result<()>;              // idempotent

    pub fn track_insert(&self, t: &TrackInsert) -> Result<()>;
    pub fn track_update_status(&self, id: &str, status: TrackStatus) -> Result<()>;
    pub fn track_list(&self, filter: TrackFilter) -> Result<Vec<Track>>;
    pub fn track_event_append(&self, e: &TrackEventInsert) -> Result<()>;

    pub fn learning_append(&self, l: &LearningInsert) -> Result<i64>;
    pub fn learnings_search(&self, query: &str, limit: usize) -> Result<Vec<LearningHit>>;

    pub fn spec_snapshot(&self, path: &str, body: &str) -> Result<i64>;
    pub fn spec_history(&self, path: &str) -> Result<Vec<SpecSnapshot>>;
}
```

Exposed to TypeScript via N-API in `crates/kit-engine-native`.

## 5. Business rules

- **`UNIQUE(module)` on tracks** enforces the dedupe rule from the kit's CodeRabbit feedback.
- **WAL + busy_timeout** so the GUI's read-only queries never block engine writes.
- **Migrations are versioned Rust functions**, not SQL files. Idempotent. Run on every `open()`.
- **FTS5 trigger-maintained** — `learnings` writes auto-sync to `learnings_fts`; no manual indexing.
- **All writes are transactional.** No partial writes on crash.
- **Connection is `Mutex<Connection>`**, not a pool. Single desktop user; pooling adds complexity without benefit.

## 6. Integration points

| Module | Relationship | Notes |
|---|---|---|
| `kit-engine` | Depends on (write) | Track CRUD, learnings append. |
| `code-maps` | None | Doesn't touch storage. |
| `gui-shell` | Depends on (read) | Direct rusqlite read access for plan board / learnings browser; no write path. |
| `kit-mcp-server` | Depends on (read) | Exposes track/learning queries as MCP tools. |
| `workflow-skills` | Indirect (via kit-engine) | — |

## 7. Acceptance criteria

- [ ] `cargo test --package session-store` passes (unit tests for migrations, FTS5 search, track CRUD, transactional rollback).
- [ ] In-memory mode (`:memory:`) works for tests.
- [ ] Migration v0→v1 runs on a fresh DB and is no-op on a current DB.
- [ ] Two concurrent writers (worker thread + GUI read thread) don't deadlock under WAL.
- [ ] FTS5 query on 1000-row `learnings` table returns top-10 results in <50ms.
- [ ] N-API bindings expose all methods to TypeScript with correct error propagation.
- [ ] Schema documented in `crates/session-store/MIGRATIONS.md`.

## 8. Out of scope

- Cross-project sync (cloud, P2P).
- Encrypted storage (use FS-level encryption if needed; SQLCipher adds complexity for low payoff in v1).
- Time-series queries (`track_events` is append-only diagnostic data, not a metric store).
- Per-user sharding within a single project DB.

## 9. Open questions

- [ ] Do we expose `learnings_fts` snippet/highlight extracts to the GUI, or just the row + score?
- [ ] Should `track_events` payloads be schema-validated, or free-form JSON? (Latter for v1; revisit if events accumulate types.)
- [ ] When a project is deleted/renamed, what happens to its global-DB references?
