# session-store — Implementation guide

Read alongside `SPEC.md` and the root `CLAUDE.md`.

## Patterns to follow

- **`Mutex<Connection>` shared state** via Tauri-managed state (or N-API class). Single connection, WAL-enabled.
- **Migrations are Rust functions**, named `migrate_v1`, `migrate_v2`, … Each takes a `&Connection` and is idempotent. The `schema_version` table records the current version on every migration.
- **Errors:** `thiserror`-based `SessionStoreError` enum. Wraps `rusqlite::Error`, `serde_json::Error`, `io::Error`. Public API returns `Result<T, SessionStoreError>`.
- **All writes go through prepared statements** cached on the connection (`Connection::prepare_cached`).
- **JSON columns** (`tags`, `payload`) are serialised via serde_json; provide typed setter/getter helpers, never expose raw strings.

## Conventions

- Crate name: `kit-session-store`.
- File layout: `src/lib.rs` (public API), `src/migrations.rs` (versioned migration fns), `src/types.rs` (Track, Learning, etc.), `src/error.rs`, `tests/`.
- All public types are `Serialize + Deserialize` so N-API can marshal them.
- Time stored as `INTEGER` (unix epoch milliseconds), exposed as `chrono::DateTime<Utc>` in Rust types.
- No `unsafe` code.

## Module boundaries

This module **owns**: SQLite schema, migrations, FTS5 indexing, all rusqlite calls.

This module **must NOT**: read or write filesystem outside the DB file path; touch git; spawn processes; depend on Tauri or Pi.

This module **is read by**: `kit-engine` (writes), `gui-shell` (reads), `kit-mcp-server` (reads).

## Known gotchas

- **rusqlite `bundled` feature.** Don't use `bundled-sqlcipher` unless we add encryption. Plain `bundled` ships SQLite + FTS5 which is what we need.
- **WAL + read-only connections.** A second read-only connection is fine for the GUI; don't share the same `Mutex<Connection>` with the writer.
- **FTS5 contentless tables vs content-tied.** We use `content='learnings'` (content-tied) so a single source of truth. Maintain via triggers; no manual `INSERT INTO learnings_fts`.
- **Migrations + Tauri v2 dep tree.** Tauri may pin rusqlite to a specific version. Match it; don't fight it.
- **`UNIQUE(module)` on tracks** is strict — re-starts must `cleanup` first or pick a different module. The error message must be actionable per the kit's CodeRabbit fifth-pass finding.

## Test patterns

- Use `Connection::open_in_memory()` for unit tests; isolate per-test.
- Property tests for FTS5 search via `proptest`.
- Integration test: open store, write 100 tracks, mutate statuses, query — assert state.
- Migration tests: open a v0 DB, run `migrate()`, assert v1 schema; then `migrate()` again, assert no-op.
