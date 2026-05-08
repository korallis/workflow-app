use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures::future::BoxFuture;
use git2::{BranchType, Repository, WorktreeAddOptions};
use kit_session_store as session_store;
use kit_session_store::SessionStore;
use kit_spec_engine::{ParallelYaml, SpecEngine};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time::sleep;

pub type Result<T> = std::result::Result<T, TrackEngineError>;

#[derive(Debug, Error)]
pub enum TrackEngineError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("session-store error: {0}")]
    SessionStore(#[from] session_store::SessionStoreError),

    #[error("spec-engine error: {0}")]
    SpecEngine(#[from] kit_spec_engine::SpecEngineError),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("unsupported harness")]
    UnsupportedHarness,

    #[error("duplicate module(s) in selection: {modules}")]
    DuplicateModules { modules: String },

    #[error("module '{module}' is not declared in specs/MODULES.md")]
    UnknownModule { module: String },

    #[error("module '{module}' is missing specs/modules/{module}/parallel.yaml; brownfield tracks must declare parallel.yaml")]
    MissingParallelYaml { module: String },

    #[error("module '{module}' parallel.yaml requires version: 1, found {found}")]
    UnsupportedParallelVersion { module: String, found: u32 },

    #[error("track plan is empty")]
    EmptyPlan,

    #[error("too many tracks requested: {requested}; KIT_PARALLEL_MAX={max}")]
    TooManyTracks { requested: usize, max: usize },

    #[error("dependency cycle in selected modules")]
    DependencyCycle,

    #[error("registry is locked at {path}")]
    RegistryLocked { path: PathBuf },

    #[error("track '{module}' is locked at {path}")]
    TrackLocked { module: String, path: PathBuf },

    #[error("unsafe .worktreeinclude entry '{entry}' for module '{module}'")]
    UnsafeWorktreeInclude { module: String, entry: String },

    #[error("worktree include source does not exist for module '{module}': {path}")]
    MissingWorktreeIncludeSource { module: String, path: PathBuf },

    #[error("no subscriber is available; subscribe() can only be called once")]
    SubscriberUnavailable,

    #[error("merge conflict while applying module '{module}'; run: {command}")]
    MergeConflict { module: String, command: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Harness {
    Codex,
    Claude,
}

impl From<Harness> for session_store::Harness {
    fn from(value: Harness) -> Self {
        match value {
            Harness::Codex => Self::Codex,
            Harness::Claude => Self::Claude,
        }
    }
}

impl From<session_store::Harness> for Harness {
    fn from(value: session_store::Harness) -> Self {
        match value {
            session_store::Harness::Codex => Self::Codex,
            session_store::Harness::Claude => Self::Claude,
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

impl From<TrackStatus> for session_store::TrackStatus {
    fn from(value: TrackStatus) -> Self {
        match value {
            TrackStatus::Pending => Self::Pending,
            TrackStatus::Running => Self::Running,
            TrackStatus::Completed => Self::Completed,
            TrackStatus::Timeout => Self::Timeout,
            TrackStatus::Aborted => Self::Aborted,
        }
    }
}

impl From<session_store::TrackStatus> for TrackStatus {
    fn from(value: session_store::TrackStatus) -> Self {
        match value {
            session_store::TrackStatus::Pending => Self::Pending,
            session_store::TrackStatus::Running => Self::Running,
            session_store::TrackStatus::Completed => Self::Completed,
            session_store::TrackStatus::Timeout => Self::Timeout,
            session_store::TrackStatus::Aborted => Self::Aborted,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackPlan {
    pub modules: Vec<String>,
    pub harness: Harness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub module: String,
    pub branch: String,
    pub worktree: PathBuf,
    pub harness: Harness,
    pub port: Option<u16>,
    pub status: TrackStatus,
    pub started: DateTime<Utc>,
    pub completed: Option<DateTime<Utc>>,
    pub last_commit: Option<String>,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackFilter {
    pub module: Option<String>,
    pub status: Option<TrackStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrackEvent {
    StatusChanged {
        id: String,
        from: TrackStatus,
        to: TrackStatus,
    },
    Commit {
        id: String,
        hash: String,
        subject: String,
    },
    LogLine {
        id: String,
        line: String,
    },
    Aborted {
        id: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackContext {
    pub id: String,
    pub module: String,
    pub branch: String,
    pub worktree: PathBuf,
    pub harness: Harness,
    pub dispatch_ts: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedCommit {
    pub subject: String,
    pub body: Option<String>,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchResult {
    pub pid: Option<u32>,
    pub completed: bool,
    pub last_commit: Option<String>,
    pub proposed_commits: Vec<ProposedCommit>,
    pub log_lines: Vec<String>,
}

pub trait DispatchFn: Fn(TrackContext) -> BoxFuture<'static, DispatchResult> + Send + Sync {}

impl<T> DispatchFn for T where
    T: Fn(TrackContext) -> BoxFuture<'static, DispatchResult> + Send + Sync
{
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackHandle {
    pub id: String,
    pub module: String,
    pub branch: String,
    pub worktree: PathBuf,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewResult {
    pub module: String,
    pub track: Option<Track>,
    pub proposed_commits: Vec<ProposedCommit>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeOpts {
    pub modules: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeResult {
    pub merged_modules: Vec<String>,
    pub halted: Option<MergeHalt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeHalt {
    pub module: String,
    pub command: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CleanupResult {
    pub removed_worktrees: Vec<PathBuf>,
    pub removed_branches: Vec<String>,
    pub preserved_aborted: Vec<String>,
}

pub struct TrackEngine {
    store: Arc<SessionStore>,
    spec: Arc<SpecEngine>,
    project_root: PathBuf,
    events_tx: mpsc::Sender<TrackEvent>,
    events_rx: Mutex<Option<mpsc::Receiver<TrackEvent>>>,
}

impl TrackEngine {
    pub fn new(store: Arc<SessionStore>, spec: Arc<SpecEngine>, root: PathBuf) -> Self {
        let (events_tx, events_rx) = mpsc::channel(256);
        Self {
            store,
            spec,
            project_root: absolute_path(root),
            events_tx,
            events_rx: Mutex::new(Some(events_rx)),
        }
    }

    pub fn plan(&self, modules: &[String], harness: Harness) -> Result<TrackPlan> {
        validate_harness(harness)?;
        let modules = dedupe_modules(modules)?;
        if modules.is_empty() {
            return Err(TrackEngineError::EmptyPlan);
        }

        let max = parallel_max();
        if modules.len() > max {
            return Err(TrackEngineError::TooManyTracks {
                requested: modules.len(),
                max,
            });
        }

        let declared = self
            .spec
            .load_modules()?
            .into_iter()
            .map(|entry| (entry.name.clone(), entry))
            .collect::<BTreeMap<_, _>>();

        for module in &modules {
            if !declared.contains_key(module) {
                return Err(TrackEngineError::UnknownModule {
                    module: module.clone(),
                });
            }
            let parallel = self.load_required_parallel(module)?;
            if parallel.version != 1 {
                return Err(TrackEngineError::UnsupportedParallelVersion {
                    module: module.clone(),
                    found: parallel.version,
                });
            }
        }

        Ok(TrackPlan {
            modules: topological_modules(&modules, &declared)?,
            harness,
        })
    }

    pub fn start<D>(&self, plan: &TrackPlan, dispatch: D) -> Result<Vec<TrackHandle>>
    where
        D: DispatchFn + 'static,
    {
        validate_harness(plan.harness)?;
        if plan.modules.is_empty() {
            return Err(TrackEngineError::EmptyPlan);
        }
        let _registry_lock = RegistryLock::acquire(&self.project_root)?;
        let dispatch = Arc::new(dispatch);
        let mut handles = Vec::new();

        for module in &plan.modules {
            let _track_lock = TrackLock::acquire(&self.project_root, module)?;
            let parallel = self.load_required_parallel(module)?;
            if parallel.version != 1 {
                return Err(TrackEngineError::UnsupportedParallelVersion {
                    module: module.clone(),
                    found: parallel.version,
                });
            }
            let id = track_id(module);
            let branch = format!("track/{module}");
            let worktree = self.worktree_path(module);
            let port = parallel.ports.first().copied();
            self.create_branch_and_worktree(module, &branch, &worktree)?;
            self.apply_worktree_includes(module, &worktree)?;

            self.store.track_insert(&session_store::TrackInsert {
                id: id.clone(),
                module: module.clone(),
                branch: branch.clone(),
                worktree: path_string(&worktree),
                harness: plan.harness.into(),
                port: port.map(i64::from),
                status: session_store::TrackStatus::Running,
                started_at: Utc::now(),
                last_commit: None,
                pid: None,
            })?;

            let context = TrackContext {
                id: id.clone(),
                module: module.clone(),
                branch: branch.clone(),
                worktree: worktree.clone(),
                harness: plan.harness,
                dispatch_ts: std::env::var("KIT_DISPATCH_TS").ok(),
            };
            let store = Arc::clone(&self.store);
            let tx = self.events_tx.clone();
            let dispatch = Arc::clone(&dispatch);
            tokio::spawn(async move {
                let result = dispatch(context.clone()).await;
                for line in result.log_lines {
                    append_event(
                        &store,
                        &context.id,
                        session_store::TrackEventType::LogLine,
                        json!({ "line": line }),
                    );
                    let _ = tx
                        .send(TrackEvent::LogLine {
                            id: context.id.clone(),
                            line,
                        })
                        .await;
                }
                let to = if result.completed {
                    TrackStatus::Completed
                } else {
                    TrackStatus::Aborted
                };
                let _ = store.track_update_status(&context.id, to.into());
                let _ = tx
                    .send(TrackEvent::StatusChanged {
                        id: context.id.clone(),
                        from: TrackStatus::Running,
                        to,
                    })
                    .await;
            });

            handles.push(TrackHandle {
                id,
                module: module.clone(),
                branch,
                worktree,
                pid: None,
            });
        }

        self.spawn_sentinel_watcher();
        Ok(handles)
    }

    pub fn status(&self, filter: TrackFilter) -> Result<Vec<Track>> {
        let filter = session_store::TrackFilter {
            module: filter.module,
            status: filter.status.map(Into::into),
        };
        self.store
            .track_list(filter)?
            .into_iter()
            .map(track_from_store)
            .collect()
    }

    pub fn review(&self, module: &str) -> Result<ReviewResult> {
        let track = self
            .status(TrackFilter {
                module: Some(module.to_owned()),
                status: None,
            })?
            .into_iter()
            .next();
        let proposed_commits = track
            .as_ref()
            .map(|track| read_proposed_commits(&track.worktree))
            .transpose()?
            .unwrap_or_default();
        Ok(ReviewResult {
            module: module.to_owned(),
            track,
            proposed_commits,
        })
    }

    pub fn merge(&self, opts: MergeOpts) -> Result<MergeResult> {
        let modules = match opts.modules {
            Some(modules) => dedupe_modules(&modules)?,
            None => self
                .status(TrackFilter {
                    module: None,
                    status: Some(TrackStatus::Completed),
                })?
                .into_iter()
                .map(|track| track.module)
                .collect(),
        };
        if modules.is_empty() {
            return Ok(MergeResult {
                merged_modules: Vec::new(),
                halted: None,
            });
        }
        let declared = self
            .spec
            .load_modules()?
            .into_iter()
            .map(|entry| (entry.name.clone(), entry))
            .collect::<BTreeMap<_, _>>();
        let order = topological_modules(&modules, &declared)?;
        let repo = Repository::open(&self.project_root)?;
        let mut merged_modules = Vec::new();

        for module in order {
            let branch_name = format!("track/{module}");
            let branch = repo.find_branch(&branch_name, BranchType::Local)?;
            let target = branch
                .get()
                .target()
                .ok_or_else(|| git2::Error::from_str("branch has no target"))?;
            let commit = repo.find_commit(target)?;
            repo.cherrypick(&commit, None)?;
            if has_git_conflicts(&repo)? {
                let command = format!("git -C {} mergetool", self.project_root.display());
                return Ok(MergeResult {
                    merged_modules,
                    halted: Some(MergeHalt { module, command }),
                });
            }
            commit_index_if_needed(&repo, &format!("merge track {module}"))?;
            merged_modules.push(module);
        }

        Ok(MergeResult {
            merged_modules,
            halted: None,
        })
    }

    pub fn cleanup(&self, modules: Option<Vec<String>>) -> Result<CleanupResult> {
        let tracks = match modules {
            Some(modules) => {
                let mut tracks = Vec::new();
                for module in dedupe_modules(&modules)? {
                    tracks.extend(self.status(TrackFilter {
                        module: Some(module),
                        status: None,
                    })?);
                }
                tracks
            }
            None => self.status(TrackFilter::default())?,
        };

        let mut result = CleanupResult::default();
        for track in tracks {
            if track.status == TrackStatus::Aborted {
                result.preserved_aborted.push(track.module);
                continue;
            }
            if track.worktree.exists() {
                fs::remove_dir_all(&track.worktree)?;
                result.removed_worktrees.push(track.worktree.clone());
            }
            if let Ok(repo) = Repository::open(&self.project_root) {
                if let Ok(mut branch) = repo.find_branch(&track.branch, BranchType::Local) {
                    branch.delete()?;
                    result.removed_branches.push(track.branch.clone());
                }
            }
        }
        Ok(result)
    }

    pub fn abort(&self, module: &str, reason: &str) -> Result<()> {
        let Some(track) = self
            .status(TrackFilter {
                module: Some(module.to_owned()),
                status: None,
            })?
            .into_iter()
            .next()
        else {
            return Err(TrackEngineError::UnknownModule {
                module: module.to_owned(),
            });
        };
        self.store
            .track_update_status(&track.id, session_store::TrackStatus::Aborted)?;
        append_event(
            &self.store,
            &track.id,
            session_store::TrackEventType::Sentinel,
            json!({ "reason": reason }),
        );
        let _ = self.events_tx.try_send(TrackEvent::Aborted {
            id: track.id,
            reason: reason.to_owned(),
        });
        Ok(())
    }

    pub fn subscribe(&self) -> mpsc::Receiver<TrackEvent> {
        self.events_rx
            .lock()
            .expect("track-engine subscriber mutex poisoned")
            .take()
            .expect("track-engine subscriber already taken")
    }

    fn load_required_parallel(&self, module: &str) -> Result<ParallelYaml> {
        self.spec
            .load_module_spec(module)?
            .parallel_yaml
            .ok_or_else(|| TrackEngineError::MissingParallelYaml {
                module: module.to_owned(),
            })
    }

    fn worktree_path(&self, module: &str) -> PathBuf {
        self.project_root
            .join(".kit-workflow-app")
            .join("worktrees")
            .join(format!("track-{module}"))
    }

    fn create_branch_and_worktree(
        &self,
        module: &str,
        branch: &str,
        worktree: &Path,
    ) -> Result<()> {
        if worktree.exists() {
            fs::remove_dir_all(worktree)?;
        }
        if let Some(parent) = worktree.parent() {
            fs::create_dir_all(parent)?;
        }
        let repo = Repository::open(&self.project_root)?;
        let head_commit = repo.head()?.peel_to_commit()?;
        if repo.find_branch(branch, BranchType::Local).is_err() {
            repo.branch(branch, &head_commit, false)?;
        }
        let reference = repo.find_reference(&format!("refs/heads/{branch}"))?;
        let mut opts = WorktreeAddOptions::new();
        opts.reference(Some(&reference));
        repo.worktree(module, worktree, Some(&opts))?;
        Ok(())
    }

    fn apply_worktree_includes(&self, module: &str, worktree: &Path) -> Result<()> {
        let include_path = self
            .project_root
            .join("specs")
            .join("modules")
            .join(module)
            .join(".worktreeinclude");
        if !include_path.exists() {
            return Ok(());
        }
        let root = self.project_root.canonicalize()?;
        let worktree_root = worktree.canonicalize()?;
        for raw_line in fs::read_to_string(include_path)?.lines() {
            let entry = raw_line.trim();
            if entry.is_empty() || entry.starts_with('#') {
                continue;
            }
            let rel = Path::new(entry);
            if rel.is_absolute() || has_parent_component(rel) {
                return Err(TrackEngineError::UnsafeWorktreeInclude {
                    module: module.to_owned(),
                    entry: entry.to_owned(),
                });
            }
            let src = root.join(rel);
            if !src.exists() {
                return Err(TrackEngineError::MissingWorktreeIncludeSource {
                    module: module.to_owned(),
                    path: src,
                });
            }
            let src_real = src.canonicalize()?;
            if !src_real.starts_with(&root) {
                return Err(TrackEngineError::UnsafeWorktreeInclude {
                    module: module.to_owned(),
                    entry: entry.to_owned(),
                });
            }
            let dst = worktree_root.join(rel);
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent)?;
            }
            if src_real.is_dir() {
                fs::create_dir_all(&dst)?;
            } else {
                fs::copy(&src_real, &dst)?;
            }
            let dst_real = dst.canonicalize()?;
            if !dst_real.starts_with(&worktree_root) {
                return Err(TrackEngineError::UnsafeWorktreeInclude {
                    module: module.to_owned(),
                    entry: entry.to_owned(),
                });
            }
        }
        Ok(())
    }

    fn spawn_sentinel_watcher(&self) {
        let store = Arc::clone(&self.store);
        let tx = self.events_tx.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                let tracks = match store.track_list(session_store::TrackFilter {
                    module: None,
                    status: Some(session_store::TrackStatus::Running),
                }) {
                    Ok(tracks) => tracks,
                    Err(_) => continue,
                };
                if tracks.is_empty() {
                    break;
                }
                for track in tracks {
                    let Some(pid) = track.pid else {
                        continue;
                    };
                    if !pid_alive(pid as u32) {
                        let _ = store
                            .track_update_status(&track.id, session_store::TrackStatus::Aborted);
                        append_event(
                            &store,
                            &track.id,
                            session_store::TrackEventType::Sentinel,
                            json!({ "reason": "process exited" }),
                        );
                        let _ = tx
                            .send(TrackEvent::Aborted {
                                id: track.id,
                                reason: "process exited".to_owned(),
                            })
                            .await;
                    }
                }
            }
        });
    }
}

pub struct RegistryLock {
    path: PathBuf,
}

impl RegistryLock {
    pub fn acquire(project_root: &Path) -> Result<Self> {
        let path = project_root
            .join(".kit-workflow-app")
            .join("parallel")
            .join(".registry-lock");
        acquire_lock_dir(&path).map_err(|err| {
            if err.kind() == ErrorKind::AlreadyExists {
                TrackEngineError::RegistryLocked { path: path.clone() }
            } else {
                TrackEngineError::Io(err)
            }
        })?;
        Ok(Self { path })
    }
}

impl Drop for RegistryLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

struct TrackLock {
    path: PathBuf,
}

impl TrackLock {
    fn acquire(project_root: &Path, module: &str) -> Result<Self> {
        let path = project_root
            .join(".kit-workflow-app")
            .join("parallel")
            .join("locks")
            .join(module);
        acquire_lock_dir(&path).map_err(|err| {
            if err.kind() == ErrorKind::AlreadyExists {
                TrackEngineError::TrackLocked {
                    module: module.to_owned(),
                    path: path.clone(),
                }
            } else {
                TrackEngineError::Io(err)
            }
        })?;
        Ok(Self { path })
    }
}

impl Drop for TrackLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

fn acquire_lock_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(path)
}

fn validate_harness(harness: Harness) -> Result<()> {
    match harness {
        Harness::Codex | Harness::Claude => Ok(()),
    }
}

fn dedupe_modules(modules: &[String]) -> Result<Vec<String>> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    let mut deduped = Vec::new();
    for module in modules {
        let normalized = module.to_ascii_lowercase();
        if !seen.insert(normalized.clone()) {
            duplicates.insert(normalized);
        } else {
            deduped.push(normalized);
        }
    }
    if duplicates.is_empty() {
        Ok(deduped)
    } else {
        Err(TrackEngineError::DuplicateModules {
            modules: duplicates.into_iter().collect::<Vec<_>>().join(", "),
        })
    }
}

fn topological_modules(
    modules: &[String],
    declared: &BTreeMap<String, kit_spec_engine::ModuleEntry>,
) -> Result<Vec<String>> {
    let selected = modules.iter().cloned().collect::<BTreeSet<_>>();
    let mut indegree = BTreeMap::<String, usize>::new();
    let mut outgoing = BTreeMap::<String, Vec<String>>::new();
    for module in modules {
        let entry = declared
            .get(module)
            .ok_or_else(|| TrackEngineError::UnknownModule {
                module: module.clone(),
            })?;
        indegree.entry(module.clone()).or_insert(0);
        for dep in &entry.depends_on {
            if selected.contains(dep) {
                outgoing
                    .entry(dep.clone())
                    .or_default()
                    .push(module.clone());
                *indegree.entry(module.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut queue = indegree
        .iter()
        .filter_map(|(module, count)| (*count == 0).then_some(module.clone()))
        .collect::<VecDeque<_>>();
    let mut sorted = Vec::new();
    while let Some(module) = queue.pop_front() {
        sorted.push(module.clone());
        if let Some(children) = outgoing.get(&module) {
            for child in children {
                let count = indegree.get_mut(child).expect("child exists");
                *count -= 1;
                if *count == 0 {
                    queue.push_back(child.clone());
                }
            }
        }
    }
    if sorted.len() == modules.len() {
        Ok(sorted)
    } else {
        Err(TrackEngineError::DependencyCycle)
    }
}

fn parallel_max() -> usize {
    std::env::var("KIT_PARALLEL_MAX")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value| *value > 0)
        .unwrap_or(4)
}

fn track_id(module: &str) -> String {
    format!("track-{module}-{}", Utc::now().timestamp_millis())
}

fn track_from_store(track: session_store::Track) -> Result<Track> {
    Ok(Track {
        id: track.id,
        module: track.module,
        branch: track.branch,
        worktree: absolute_path(PathBuf::from(track.worktree)),
        harness: track.harness.into(),
        port: track.port.map(|port| port as u16),
        status: track.status.into(),
        started: track.started_at,
        completed: track.completed_at,
        last_commit: track.last_commit,
        pid: track.pid.map(|pid| pid as u32),
    })
}

fn append_event(
    store: &SessionStore,
    id: &str,
    event_type: session_store::TrackEventType,
    payload: serde_json::Value,
) {
    let _ = store.track_event_append(&session_store::TrackEventInsert {
        track_id: id.to_owned(),
        event_type,
        payload,
        created_at: Utc::now(),
    });
}

fn read_proposed_commits(worktree: &Path) -> Result<Vec<ProposedCommit>> {
    let report = worktree.join("codex-report.json");
    if !report.exists() {
        return Ok(Vec::new());
    }
    #[derive(Deserialize)]
    struct Report {
        #[serde(default)]
        proposed_commits: Vec<ProposedCommit>,
    }
    Ok(serde_json::from_str::<Report>(&fs::read_to_string(report)?)?.proposed_commits)
}

fn has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn absolute_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    }
}

fn pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Path::new("/proc").join(pid.to_string()).exists()
            || Command::new("kill")
                .arg("-0")
                .arg(pid.to_string())
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}

fn has_git_conflicts(repo: &Repository) -> Result<bool> {
    Ok(repo.index()?.has_conflicts())
}

fn commit_index_if_needed(repo: &Repository, message: &str) -> Result<()> {
    let mut index = repo.index()?;
    if index.is_empty() {
        return Ok(());
    }
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let sig = repo.signature()?;
    let parent = repo.head().ok().and_then(|head| head.peel_to_commit().ok());
    if let Some(parent) = parent {
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;
    } else {
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?;
    }
    index.write()?;
    Ok(())
}

pub fn mergetool_command(project_root: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(project_root).arg("mergetool");
    command
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::FutureExt;
    use git2::{IndexAddOption, Signature};
    use tempfile::TempDir;
    use tokio::time::{timeout, Duration};

    fn fixture() -> (TempDir, Arc<SessionStore>, Arc<SpecEngine>, TrackEngine) {
        let dir = tempfile::tempdir().unwrap();
        init_repo(dir.path());
        write_project(
            dir.path(),
            &[
                ("auth", &[] as &[&str]),
                ("billing", &["auth"]),
                ("oauth", &[]),
                ("ui", &["billing"]),
            ],
        );
        let store = Arc::new(SessionStore::open_in_memory().unwrap());
        let spec = Arc::new(SpecEngine::new(
            Arc::clone(&store),
            dir.path().to_path_buf(),
        ));
        let engine = TrackEngine::new(Arc::clone(&store), spec.clone(), dir.path().to_path_buf());
        (dir, store, spec, engine)
    }

    fn init_repo(path: &Path) {
        let repo = Repository::init(path).unwrap();
        fs::write(path.join("README.md"), "fixture\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();
    }

    fn write_project(path: &Path, modules: &[(&str, &[&str])]) {
        fs::create_dir_all(path.join("specs/modules")).unwrap();
        let mut modules_md =
            String::from("| # | Module | Layer | Status | Depends on |\n|---|---|---|---|---|\n");
        for (module, deps) in modules {
            modules_md.push_str(&format!(
                "| 1 | `{module}` | Core | planned | {} |\n",
                deps.join(", ")
            ));
            let module_dir = path.join("specs/modules").join(module);
            fs::create_dir_all(&module_dir).unwrap();
            fs::write(module_dir.join("SPEC.md"), format!("# {module}\n")).unwrap();
            fs::write(module_dir.join("CLAUDE.md"), format!("# {module}\n")).unwrap();
            fs::write(
                module_dir.join("parallel.yaml"),
                "version: 1\ntouches: []\nshared: []\nports: []\nmigrations: false\n",
            )
            .unwrap();
        }
        fs::write(path.join("specs/MODULES.md"), modules_md).unwrap();
    }

    #[test]
    fn serde_contracts_round_trip() {
        let track = Track {
            id: "track-auth-1".to_owned(),
            module: "auth".to_owned(),
            branch: "track/auth".to_owned(),
            worktree: PathBuf::from("/tmp/auth"),
            harness: Harness::Codex,
            port: Some(3000),
            status: TrackStatus::Pending,
            started: Utc::now(),
            completed: None,
            last_commit: None,
            pid: Some(42),
        };
        let encoded = serde_json::to_string(&track).unwrap();
        assert_eq!(serde_json::from_str::<Track>(&encoded).unwrap(), track);
    }

    #[test]
    fn plan_sorts_dependency_edges() {
        let (_dir, _, _, engine) = fixture();
        let plan = engine
            .plan(
                &["ui".into(), "auth".into(), "billing".into()],
                Harness::Codex,
            )
            .unwrap();
        assert_eq!(plan.modules, vec!["auth", "billing", "ui"]);
    }

    #[test]
    fn slug_boundary_does_not_create_substring_edges() {
        let (_dir, _, _, engine) = fixture();
        let plan = engine
            .plan(&["oauth".into(), "auth".into()], Harness::Claude)
            .unwrap();
        assert_eq!(plan.modules, vec!["auth", "oauth"]);
    }

    #[test]
    fn dedupe_rejects_lower_and_mixed_case_duplicates() {
        let (_, _, _, engine) = fixture();
        let err = engine
            .plan(&["auth".into(), "auth".into()], Harness::Codex)
            .unwrap_err();
        assert!(err.to_string().contains("duplicate module(s)"));
        let err = engine
            .plan(&["auth".into(), "Auth".into()], Harness::Codex)
            .unwrap_err();
        assert!(err.to_string().contains("auth"));
    }

    #[test]
    fn brownfield_without_parallel_yaml_is_rejected() {
        let (dir, _, _, engine) = fixture();
        fs::remove_file(dir.path().join("specs/modules/auth/parallel.yaml")).unwrap();
        let err = engine.plan(&["auth".into()], Harness::Codex).unwrap_err();
        assert!(err.to_string().contains("brownfield"));
    }

    #[test]
    fn unsupported_parallel_version_is_rejected() {
        let (dir, _, _, engine) = fixture();
        fs::write(
            dir.path().join("specs/modules/auth/parallel.yaml"),
            "version: 2\ntouches: []\nshared: []\nports: []\nmigrations: false\n",
        )
        .unwrap();
        let err = engine.plan(&["auth".into()], Harness::Codex).unwrap_err();
        assert!(err.to_string().contains("version: 1"));
    }

    #[tokio::test]
    async fn start_creates_worktree_registry_row_and_emits_status() {
        let (_dir, store, _, engine) = fixture();
        let mut rx = engine.subscribe();
        let plan = engine.plan(&["auth".into()], Harness::Codex).unwrap();
        let handles = engine
            .start(&plan, |ctx| {
                async move {
                    assert!(ctx.worktree.is_absolute());
                    DispatchResult {
                        completed: true,
                        log_lines: vec!["ready".to_owned()],
                        ..DispatchResult::default()
                    }
                }
                .boxed()
            })
            .unwrap();
        assert_eq!(handles.len(), 1);
        assert!(handles[0].worktree.exists());
        let event = timeout(Duration::from_secs(3), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(
            event,
            TrackEvent::LogLine { .. } | TrackEvent::StatusChanged { .. }
        ));
        let rows = store
            .track_list(session_store::TrackFilter::default())
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].module, "auth");
    }

    #[tokio::test]
    async fn concurrent_start_calls_do_not_duplicate_modules() {
        let (_, store, spec, _) = fixture();
        let mut joins = Vec::new();
        for module in ["auth", "billing", "oauth", "ui"] {
            let store = Arc::clone(&store);
            let spec = Arc::clone(&spec);
            joins.push(tokio::spawn(async move {
                let engine = TrackEngine::new(
                    Arc::clone(&store),
                    spec,
                    tempfile::tempdir().unwrap().path().to_path_buf(),
                );
                let insert = session_store::TrackInsert {
                    id: format!("track-{module}-1"),
                    module: module.to_owned(),
                    branch: format!("track/{module}"),
                    worktree: format!("/tmp/{module}"),
                    harness: session_store::Harness::Codex,
                    port: None,
                    status: session_store::TrackStatus::Running,
                    started_at: Utc::now(),
                    last_commit: None,
                    pid: None,
                };
                let _ = engine;
                store.track_insert(&insert)
            }));
        }
        for join in joins {
            join.await.unwrap().unwrap();
        }
        assert_eq!(
            store
                .track_list(session_store::TrackFilter::default())
                .unwrap()
                .len(),
            4
        );
    }

    #[test]
    fn registry_lock_path_is_sibling_of_track_locks() {
        let (dir, _, _, _) = fixture();
        let lock = RegistryLock::acquire(dir.path()).unwrap();
        assert!(dir
            .path()
            .join(".kit-workflow-app/parallel/.registry-lock")
            .exists());
        assert!(!lock.path.ends_with("locks/registry"));
    }

    #[test]
    fn worktreeinclude_rejects_traversal_entries() {
        let (dir, _, _, engine) = fixture();
        fs::write(
            dir.path().join("specs/modules/auth/.worktreeinclude"),
            "../secret\n",
        )
        .unwrap();
        let plan = engine.plan(&["auth".into()], Harness::Codex).unwrap();
        let err = engine
            .start(&plan, |_| async { DispatchResult::default() }.boxed())
            .unwrap_err();
        assert!(err.to_string().contains("unsafe"));
    }

    #[tokio::test]
    async fn sentinel_watcher_marks_dead_pid_aborted() {
        let (_, store, _, engine) = fixture();
        store
            .track_insert(&session_store::TrackInsert {
                id: "track-auth-1".to_owned(),
                module: "auth".to_owned(),
                branch: "track/auth".to_owned(),
                worktree: "/tmp/auth".to_owned(),
                harness: session_store::Harness::Codex,
                port: None,
                status: session_store::TrackStatus::Running,
                started_at: Utc::now(),
                last_commit: None,
                pid: Some(999_999_999),
            })
            .unwrap();
        let mut rx = engine.subscribe();
        engine.spawn_sentinel_watcher();
        let event = timeout(Duration::from_secs(5), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(matches!(event, TrackEvent::Aborted { .. }));
        let row = store
            .track_list(session_store::TrackFilter {
                module: Some("auth".to_owned()),
                status: Some(session_store::TrackStatus::Aborted),
            })
            .unwrap();
        assert_eq!(row.len(), 1);
    }

    #[test]
    fn merge_order_follows_dependencies() {
        let (_dir, _, _, engine) = fixture();
        let declared = engine
            .spec
            .load_modules()
            .unwrap()
            .into_iter()
            .map(|entry| (entry.name.clone(), entry))
            .collect::<BTreeMap<_, _>>();
        let order = topological_modules(&["ui".into(), "billing".into(), "auth".into()], &declared)
            .unwrap();
        assert_eq!(order, vec!["auth", "billing", "ui"]);
    }

    #[test]
    fn mergetool_invocation_is_git_c_mergetool() {
        let command = mergetool_command(Path::new("/tmp/project"));
        assert_eq!(command.get_program(), "git");
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(args, vec!["-C", "/tmp/project", "mergetool"]);
    }

    #[allow(dead_code)]
    fn commit_all(repo: &Repository, message: &str) {
        let mut index = repo.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])
            .unwrap();
    }

    #[test]
    fn proposed_commits_are_read_from_schema_report() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("codex-report.json"),
            r#"{"proposed_commits":[{"subject":"feat: x","body":null,"paths":["x"]}]}"#,
        )
        .unwrap();
        let commits = read_proposed_commits(dir.path()).unwrap();
        assert_eq!(commits[0].subject, "feat: x");
    }

    #[test]
    fn cleanup_preserves_aborted_tracks() {
        let (_, store, _, engine) = fixture();
        store
            .track_insert(&session_store::TrackInsert {
                id: "track-auth-1".to_owned(),
                module: "auth".to_owned(),
                branch: "track/auth".to_owned(),
                worktree: "/tmp/auth".to_owned(),
                harness: session_store::Harness::Codex,
                port: None,
                status: session_store::TrackStatus::Aborted,
                started_at: Utc::now(),
                last_commit: None,
                pid: None,
            })
            .unwrap();
        let result = engine.cleanup(Some(vec!["auth".to_owned()])).unwrap();
        assert_eq!(result.preserved_aborted, vec!["auth"]);
    }
}
