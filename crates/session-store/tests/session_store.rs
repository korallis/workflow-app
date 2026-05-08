use std::sync::Arc;
use std::thread;
use std::time::Instant;

use chrono::Utc;
use kit_session_store::{
    Harness, LearningInsert, SessionStore, SessionStoreError, TrackEventInsert, TrackEventType,
    TrackFilter, TrackInsert, TrackStatus,
};
use serde_json::json;
use tempfile::tempdir;

fn track(module: &str) -> TrackInsert {
    TrackInsert {
        id: format!("track-{module}-1"),
        module: module.to_string(),
        branch: format!("feature/{module}"),
        worktree: format!("/tmp/{module}"),
        harness: Harness::Codex,
        port: Some(3000),
        status: TrackStatus::Pending,
        started_at: Utc::now(),
        last_commit: None,
        pid: Some(42),
    }
}

#[test]
fn migration_runs_on_fresh_db_and_is_idempotent() {
    let store = SessionStore::open_in_memory().unwrap();

    assert_eq!(store.schema_version().unwrap(), 1);
    store.migrate().unwrap();
    assert_eq!(store.schema_version().unwrap(), 1);
}

#[test]
fn track_crud_filters_and_actionable_duplicate_error() {
    let store = SessionStore::open_in_memory().unwrap();
    let insert = track("session-store");

    store.track_insert(&insert).unwrap();
    let tracks = store
        .track_list(TrackFilter {
            module: Some("session-store".to_string()),
            status: Some(TrackStatus::Pending),
        })
        .unwrap();

    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].module, "session-store");

    let err = store.track_insert(&insert).unwrap_err();
    assert!(matches!(
        &err,
        SessionStoreError::DuplicateTrackModule { module } if module == "session-store"
    ));
    assert!(err.to_string().contains("cleanup"));
}

#[test]
fn status_update_is_transactional_and_appends_typed_event() {
    let store = SessionStore::open_in_memory().unwrap();
    store.track_insert(&track("engine")).unwrap();

    store
        .track_update_status("track-engine-1", TrackStatus::Completed)
        .unwrap();

    let tracks = store
        .track_list(TrackFilter {
            module: Some("engine".to_string()),
            status: Some(TrackStatus::Completed),
        })
        .unwrap();
    assert_eq!(tracks.len(), 1);
    assert!(tracks[0].completed_at.is_some());

    let events = store.track_events("track-engine-1").unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, TrackEventType::StatusChange);
    assert_eq!(events[0].payload, json!({ "status": "completed" }));
}

#[test]
fn event_payload_round_trips_as_json() {
    let store = SessionStore::open_in_memory().unwrap();
    store.track_insert(&track("events")).unwrap();

    store
        .track_event_append(&TrackEventInsert {
            track_id: "track-events-1".to_string(),
            event_type: TrackEventType::LogLine,
            payload: json!({ "line": "ready", "count": 2 }),
            created_at: Utc::now(),
        })
        .unwrap();

    let events = store.track_events("track-events-1").unwrap();
    assert_eq!(events[0].payload["line"], "ready");
    assert_eq!(events[0].payload["count"], 2);
}

#[test]
fn fts5_learnings_are_trigger_maintained_and_fast_enough() {
    let store = SessionStore::open_in_memory().unwrap();
    for i in 0..1000 {
        store
            .learning_append(&LearningInsert {
                module: Some("session-store".to_string()),
                project_name: "workflow-app".to_string(),
                body: format!("learning {i} about tauri-2 sqlite wal fts5 indexing"),
                tags: vec!["tauri-2".to_string(), "sqlite".to_string()],
                commit_hash: Some(format!("commit-{i}")),
                created_at: Utc::now(),
            })
            .unwrap();
    }

    let started = Instant::now();
    let hits = store.learnings_search("\"tauri-2\" sqlite", 10).unwrap();
    let elapsed = started.elapsed();

    assert_eq!(hits.len(), 10);
    assert!(hits
        .iter()
        .all(|hit| hit.learning.tags.contains(&"tauri-2".to_string())));
    assert!(
        elapsed.as_millis() < 50,
        "FTS5 query took {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn spec_snapshots_are_deduplicated_by_path_and_hash() {
    let store = SessionStore::open_in_memory().unwrap();

    let first = store
        .spec_snapshot("specs/MASTER_BLUEPRINT.md", "body")
        .unwrap();
    let second = store
        .spec_snapshot("specs/MASTER_BLUEPRINT.md", "body")
        .unwrap();
    let third = store
        .spec_snapshot("specs/MASTER_BLUEPRINT.md", "changed")
        .unwrap();

    assert_eq!(first, second);
    assert_ne!(first, third);
    assert_eq!(
        store
            .spec_history("specs/MASTER_BLUEPRINT.md")
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn wal_allows_reader_and_writer_threads_without_deadlock() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("state.db");
    let writer = Arc::new(SessionStore::open(&db_path).unwrap());
    writer.track_insert(&track("concurrent")).unwrap();

    let reader_path = db_path.clone();
    let reader = thread::spawn(move || {
        let reader = SessionStore::open(&reader_path).unwrap();
        for _ in 0..50 {
            let rows = reader.track_list(TrackFilter::default()).unwrap();
            assert!(!rows.is_empty());
        }
    });

    let writer_thread = {
        let writer = Arc::clone(&writer);
        thread::spawn(move || {
            for _ in 0..50 {
                writer
                    .track_event_append(&TrackEventInsert {
                        track_id: "track-concurrent-1".to_string(),
                        event_type: TrackEventType::Sentinel,
                        payload: json!({ "ok": true }),
                        created_at: Utc::now(),
                    })
                    .unwrap();
            }
        })
    };

    reader.join().unwrap();
    writer_thread.join().unwrap();
    assert_eq!(writer.track_events("track-concurrent-1").unwrap().len(), 50);
}
