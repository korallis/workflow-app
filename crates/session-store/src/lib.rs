mod error;
mod migrations;
mod types;

use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection};
use serde_json::json;
use sha2::{Digest, Sha256};

pub use error::{Result, SessionStoreError};
pub use types::*;

pub struct SessionStore {
    conn: Mutex<Connection>,
}

impl SessionStore {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        Self::from_connection(conn)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Self> {
        configure_connection(&conn)?;
        migrations::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn migrate(&self) -> Result<()> {
        let conn = self.conn()?;
        migrations::migrate(&conn)
    }

    pub fn schema_version(&self) -> Result<i64> {
        let conn = self.conn()?;
        migrations::current_version(&conn)
    }

    pub fn track_insert(&self, t: &TrackInsert) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        let result = {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO tracks (
                    id, module, branch, worktree, harness, port, status, started_at,
                    completed_at, last_commit, pid
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10)",
            )?;

            stmt.execute(params![
                t.id,
                t.module,
                t.branch,
                t.worktree,
                t.harness.as_str(),
                t.port,
                t.status.as_str(),
                to_millis(t.started_at),
                t.last_commit,
                t.pid
            ])
        };

        if let Err(err) = result {
            if is_unique_module_violation(&err) {
                return Err(SessionStoreError::DuplicateTrackModule {
                    module: t.module.clone(),
                });
            }
            return Err(err.into());
        }

        tx.commit()?;
        Ok(())
    }

    pub fn track_update_status(&self, id: &str, status: TrackStatus) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        let completed_at = status.is_terminal().then(|| to_millis(Utc::now()));

        {
            let mut stmt = tx.prepare_cached(
                "UPDATE tracks
                 SET status = ?1, completed_at = CASE WHEN ?2 IS NULL THEN completed_at ELSE ?2 END
                 WHERE id = ?3",
            )?;
            stmt.execute(params![status.as_str(), completed_at, id])?;
        }

        {
            let payload = serde_json::to_string(&json!({ "status": status.as_str() }))?;
            let mut stmt = tx.prepare_cached(
                "INSERT INTO track_events (track_id, event_type, payload, created_at)
                 VALUES (?1, 'status_change', ?2, ?3)",
            )?;
            stmt.execute(params![id, payload, to_millis(Utc::now())])?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn track_list(&self, filter: TrackFilter) -> Result<Vec<Track>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, module, branch, worktree, harness, port, status, started_at,
                    completed_at, last_commit, pid
             FROM tracks
             WHERE (?1 IS NULL OR module = ?1)
               AND (?2 IS NULL OR status = ?2)
             ORDER BY started_at DESC, id ASC",
        )?;
        let status = filter.status.map(TrackStatus::as_str);
        let rows = stmt.query_map(params![filter.module, status], row_to_track)?;
        collect_rows(rows)
    }

    pub fn track_event_append(&self, e: &TrackEventInsert) -> Result<()> {
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        let payload = serde_json::to_string(&e.payload)?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO track_events (track_id, event_type, payload, created_at)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;
            stmt.execute(params![
                e.track_id,
                e.event_type.as_str(),
                payload,
                to_millis(e.created_at)
            ])?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn track_events(&self, track_id: &str) -> Result<Vec<TrackEvent>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, track_id, event_type, payload, created_at
             FROM track_events
             WHERE track_id = ?1
             ORDER BY created_at ASC, id ASC",
        )?;
        let rows = stmt.query_map([track_id], row_to_track_event)?;
        collect_rows(rows)
    }

    pub fn learning_append(&self, l: &LearningInsert) -> Result<i64> {
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        let tags = serde_json::to_string(&l.tags)?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO learnings (module, project_name, body, tags, commit_hash, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            stmt.execute(params![
                l.module,
                l.project_name,
                l.body,
                tags,
                l.commit_hash,
                to_millis(l.created_at)
            ])?;
        }
        let id = tx.last_insert_rowid();
        tx.commit()?;
        Ok(id)
    }

    pub fn learnings_search(&self, query: &str, limit: usize) -> Result<Vec<LearningHit>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare_cached(
            "SELECT l.id, l.module, l.project_name, l.body, l.tags, l.commit_hash, l.created_at,
                    bm25(learnings_fts) AS score
             FROM learnings_fts
             JOIN learnings l ON l.id = learnings_fts.rowid
             WHERE learnings_fts MATCH ?1
             ORDER BY score ASC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit as i64], |row| {
            let learning = row_to_learning(row)?;
            let score = row.get(7)?;
            Ok(LearningHit { learning, score })
        })?;
        collect_rows(rows)
    }

    pub fn spec_snapshot(&self, path: &str, body: &str) -> Result<i64> {
        let hash = content_hash(body);
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT OR IGNORE INTO spec_snapshots (spec_path, content_hash, body, taken_at)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;
            stmt.execute(params![path, hash, body, to_millis(Utc::now())])?;
        }
        let id = tx.query_row(
            "SELECT id FROM spec_snapshots WHERE spec_path = ?1 AND content_hash = ?2",
            params![path, hash],
            |row| row.get(0),
        )?;
        tx.commit()?;
        Ok(id)
    }

    pub fn spec_history(&self, path: &str) -> Result<Vec<SpecSnapshot>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare_cached(
            "SELECT id, spec_path, content_hash, body, taken_at
             FROM spec_snapshots
             WHERE spec_path = ?1
             ORDER BY taken_at DESC, id DESC",
        )?;
        let rows = stmt.query_map([path], row_to_spec_snapshot)?;
        collect_rows(rows)
    }

    fn conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| SessionStoreError::LockPoisoned)
    }
}

fn configure_connection(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.busy_timeout(Duration::from_millis(5000))?;
    Ok(())
}

fn to_millis(value: DateTime<Utc>) -> i64 {
    value.timestamp_millis()
}

fn from_millis(value: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(value)
        .single()
        .expect("valid unix epoch milliseconds from sqlite")
}

fn collect_rows<T>(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>>,
) -> Result<Vec<T>> {
    rows.collect::<std::result::Result<Vec<_>, _>>()
        .map_err(SessionStoreError::from)
}

fn row_to_track(row: &rusqlite::Row<'_>) -> rusqlite::Result<Track> {
    let harness: String = row.get(4)?;
    let status: String = row.get(6)?;
    let completed_at: Option<i64> = row.get(8)?;

    Ok(Track {
        id: row.get(0)?,
        module: row.get(1)?,
        branch: row.get(2)?,
        worktree: row.get(3)?,
        harness: Harness::from_str(&harness),
        port: row.get(5)?,
        status: TrackStatus::from_str(&status),
        started_at: from_millis(row.get(7)?),
        completed_at: completed_at.map(from_millis),
        last_commit: row.get(9)?,
        pid: row.get(10)?,
    })
}

fn row_to_track_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<TrackEvent> {
    let event_type: String = row.get(2)?;
    let payload: String = row.get(3)?;
    let payload = serde_json::from_str(&payload).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(err))
    })?;

    Ok(TrackEvent {
        id: row.get(0)?,
        track_id: row.get(1)?,
        event_type: TrackEventType::from_str(&event_type),
        payload,
        created_at: from_millis(row.get(4)?),
    })
}

fn row_to_learning(row: &rusqlite::Row<'_>) -> rusqlite::Result<Learning> {
    let tags: String = row.get(4)?;
    let tags = serde_json::from_str(&tags).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(err))
    })?;

    Ok(Learning {
        id: row.get(0)?,
        module: row.get(1)?,
        project_name: row.get(2)?,
        body: row.get(3)?,
        tags,
        commit_hash: row.get(5)?,
        created_at: from_millis(row.get(6)?),
    })
}

fn row_to_spec_snapshot(row: &rusqlite::Row<'_>) -> rusqlite::Result<SpecSnapshot> {
    Ok(SpecSnapshot {
        id: row.get(0)?,
        spec_path: row.get(1)?,
        content_hash: row.get(2)?,
        body: row.get(3)?,
        taken_at: from_millis(row.get(4)?),
    })
}

fn content_hash(body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn is_unique_module_violation(err: &rusqlite::Error) -> bool {
    matches!(
        err,
        rusqlite::Error::SqliteFailure(sqlite_err, Some(message))
            if sqlite_err.code == rusqlite::ErrorCode::ConstraintViolation
                && message.contains("tracks.module")
    )
}
