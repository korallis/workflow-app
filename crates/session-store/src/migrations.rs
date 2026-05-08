use rusqlite::{Connection, OptionalExtension};

use crate::error::Result;

pub(crate) fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )?;

    let current = current_version(conn)?;
    if current < 1 {
        migrate_v1(conn)?;
    }

    Ok(())
}

pub(crate) fn current_version(conn: &Connection) -> Result<i64> {
    let version = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
            row.get::<_, Option<i64>>(0)
        })
        .optional()?
        .flatten()
        .unwrap_or(0);

    Ok(version)
}

pub(crate) fn migrate_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "BEGIN;

        CREATE TABLE IF NOT EXISTS tracks (
            id TEXT PRIMARY KEY,
            module TEXT NOT NULL,
            branch TEXT NOT NULL,
            worktree TEXT NOT NULL,
            harness TEXT NOT NULL CHECK (harness IN ('codex', 'claude')),
            port INTEGER,
            status TEXT NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'timeout', 'aborted')),
            started_at INTEGER NOT NULL,
            completed_at INTEGER,
            last_commit TEXT,
            pid INTEGER,
            UNIQUE(module)
        );

        CREATE TABLE IF NOT EXISTS track_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            track_id TEXT NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
            event_type TEXT NOT NULL CHECK (event_type IN ('status_change', 'commit', 'log_line', 'sentinel')),
            payload TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_track_events_track_id_created_at
            ON track_events(track_id, created_at);

        CREATE TABLE IF NOT EXISTS learnings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            module TEXT,
            project_name TEXT NOT NULL,
            body TEXT NOT NULL,
            tags TEXT NOT NULL,
            commit_hash TEXT,
            created_at INTEGER NOT NULL
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS learnings_fts
            USING fts5(body, content='learnings', content_rowid='id');

        CREATE TRIGGER IF NOT EXISTS learnings_ai AFTER INSERT ON learnings BEGIN
            INSERT INTO learnings_fts(rowid, body) VALUES (new.id, new.body);
        END;

        CREATE TRIGGER IF NOT EXISTS learnings_ad AFTER DELETE ON learnings BEGIN
            INSERT INTO learnings_fts(learnings_fts, rowid, body)
            VALUES('delete', old.id, old.body);
        END;

        CREATE TRIGGER IF NOT EXISTS learnings_au AFTER UPDATE ON learnings BEGIN
            INSERT INTO learnings_fts(learnings_fts, rowid, body)
            VALUES('delete', old.id, old.body);
            INSERT INTO learnings_fts(rowid, body) VALUES (new.id, new.body);
        END;

        CREATE TABLE IF NOT EXISTS spec_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            spec_path TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            body TEXT NOT NULL,
            taken_at INTEGER NOT NULL,
            UNIQUE(spec_path, content_hash)
        );

        INSERT OR IGNORE INTO schema_version(version, applied_at)
            VALUES (1, CAST(strftime('%s', 'now') AS INTEGER) * 1000);

        COMMIT;",
    )?;

    Ok(())
}
