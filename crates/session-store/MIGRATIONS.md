# kit-session-store migrations

Migrations are Rust functions in `src/migrations.rs` and are recorded in the
`schema_version` table. `SessionStore::open` configures the SQLite connection
and runs migrations every time; current databases no-op after version `1`.

Connection pragmas:

- `PRAGMA foreign_keys = ON`
- `PRAGMA journal_mode = WAL`
- `busy_timeout = 5000ms`

## v1

Creates the durable project state schema:

- `schema_version(version, applied_at)`
- `tracks`
  - Primary key: `id`
  - Unique key: `module`
  - Status values: `pending`, `running`, `completed`, `timeout`, `aborted`
  - Harness values: `codex`, `claude`
- `track_events`
  - Cascades on track deletion
  - Event values: `status_change`, `commit`, `log_line`, `sentinel`
  - `payload` is JSON text managed by typed API helpers
- `learnings`
  - `tags` is a JSON string array managed by typed API helpers
- `learnings_fts`
  - FTS5 external-content table tied to `learnings`
  - Maintained only by SQLite triggers: `learnings_ai`, `learnings_ad`,
    `learnings_au`
- `spec_snapshots`
  - Unique key: `(spec_path, content_hash)`

All write APIs wrap their statements in transactions. Learning FTS rows are not
inserted by application code; the triggers keep the FTS table synchronized with
the `learnings` source table.
