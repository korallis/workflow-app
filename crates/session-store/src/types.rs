use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Harness {
    Codex,
    Claude,
}

impl Harness {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::Claude => "claude",
        }
    }

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "claude" => Self::Claude,
            _ => Self::Codex,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackStatus {
    Pending,
    Running,
    Completed,
    Timeout,
    Aborted,
}

impl TrackStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Timeout => "timeout",
            Self::Aborted => "aborted",
        }
    }

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "completed" => Self::Completed,
            "timeout" => Self::Timeout,
            "aborted" => Self::Aborted,
            _ => Self::Pending,
        }
    }

    pub(crate) fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Timeout | Self::Aborted)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackEventType {
    StatusChange,
    Commit,
    LogLine,
    Sentinel,
}

impl TrackEventType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::StatusChange => "status_change",
            Self::Commit => "commit",
            Self::LogLine => "log_line",
            Self::Sentinel => "sentinel",
        }
    }

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "commit" => Self::Commit,
            "log_line" => Self::LogLine,
            "sentinel" => Self::Sentinel,
            _ => Self::StatusChange,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub module: String,
    pub branch: String,
    pub worktree: String,
    pub harness: Harness,
    pub port: Option<i64>,
    pub status: TrackStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub last_commit: Option<String>,
    pub pid: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackInsert {
    pub id: String,
    pub module: String,
    pub branch: String,
    pub worktree: String,
    pub harness: Harness,
    pub port: Option<i64>,
    pub status: TrackStatus,
    pub started_at: DateTime<Utc>,
    pub last_commit: Option<String>,
    pub pid: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackFilter {
    pub module: Option<String>,
    pub status: Option<TrackStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackEvent {
    pub id: i64,
    pub track_id: String,
    pub event_type: TrackEventType,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackEventInsert {
    pub track_id: String,
    pub event_type: TrackEventType,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Learning {
    pub id: i64,
    pub module: Option<String>,
    pub project_name: String,
    pub body: String,
    pub tags: Vec<String>,
    pub commit_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningInsert {
    pub module: Option<String>,
    pub project_name: String,
    pub body: String,
    pub tags: Vec<String>,
    pub commit_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningHit {
    pub learning: Learning,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpecSnapshot {
    pub id: i64,
    pub spec_path: String,
    pub content_hash: String,
    pub body: String,
    pub taken_at: DateTime<Utc>,
}
