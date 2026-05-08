# track-engine — SPEC

> **Layer:** Core (Rust crate `kit-track-engine`)
> **Last updated:** 2026-05-08 (Pi-removal pivot)
> **Depends on:** session-store, spec-engine

## 1. Purpose

Parallel-tracks state machine, worktree management, merge sequencer, sentinel-watcher. Direct port of the bash kit's `project-tracks.sh` into Rust, carrying forward every CodeRabbit-vetted invariant from PR #7's six review rounds. No agent calls — this crate is pure orchestration over filesystem + git + SQLite. Replaces the track-engine sub-domain of the previously-proposed kit-engine Pi extension.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| TE-1 | skill-runner running `/project-tracks plan` | Validate a candidate set of modules + return a topological merge order | Plans are deterministic and reject invalid sets (shared paths, dep edges, missing parallel.yaml) |
| TE-2 | skill-runner running `/project-tracks start` | Create per-track worktrees, register in SQLite, dispatch one harness per track, watch sentinels | Multiple modules implement in parallel without conflict |
| TE-3 | gui-shell plan board | Subscribe to track-state changes via Tauri events | Dashboard updates live |
| TE-4 | track-engine itself | Detect pid-died tracks (operator killed pane) and mark `aborted` | Status remains accurate without losing branch+worktree work |
| TE-5 | skill-runner running `/project-tracks merge` | Apply tracks dependency-ordered with rebase + per-track test | Merges follow a clean linear history; halt at first conflict |

## 3. Public API

```rust
pub struct TrackEngine {
    store: Arc<SessionStore>,
    spec: Arc<SpecEngine>,
    project_root: PathBuf,
}

impl TrackEngine {
    pub fn new(store: Arc<SessionStore>, spec: Arc<SpecEngine>, root: PathBuf) -> Self;

    pub fn plan(&self, modules: &[String], harness: Harness) -> Result<TrackPlan>;
    pub fn start(&self, plan: &TrackPlan, dispatch: impl DispatchFn) -> Result<Vec<TrackHandle>>;
    pub fn status(&self, filter: TrackFilter) -> Result<Vec<Track>>;
    pub fn review(&self, module: &str) -> Result<ReviewResult>;
    pub fn merge(&self, opts: MergeOpts) -> Result<MergeResult>;
    pub fn cleanup(&self, modules: Option<Vec<String>>) -> Result<CleanupResult>;
    pub fn abort(&self, module: &str, reason: &str) -> Result<()>;

    pub fn subscribe(&self) -> mpsc::Receiver<TrackEvent>;       // for GUI
}

pub trait DispatchFn: Fn(TrackContext) -> BoxFuture<'static, DispatchResult> + Send + Sync;
```

`DispatchFn` is the seam where skill-runner injects the actual claude-bridge / codex-bridge invocation. track-engine never calls them directly.

## 4. Data contracts

```rust
#[derive(Serialize, Deserialize)]
pub enum Harness { Codex, Claude }

#[derive(Serialize, Deserialize)]
pub enum TrackStatus { Pending, Running, Completed, Timeout, Aborted }

#[derive(Serialize, Deserialize)]
pub struct TrackPlan {
    pub modules: Vec<String>,                          // topologically sorted
    pub harness: Harness,
}

#[derive(Serialize, Deserialize)]
pub struct Track {
    pub id: String,                                    // "track-<module>-<ts>"
    pub module: String,
    pub branch: String,                                // "track/<module>"
    pub worktree: PathBuf,
    pub harness: Harness,
    pub port: Option<u16>,
    pub status: TrackStatus,
    pub started: DateTime<Utc>,
    pub completed: Option<DateTime<Utc>>,
    pub last_commit: Option<String>,
    pub pid: Option<u32>,
}

pub enum TrackEvent {
    StatusChanged { id: String, from: TrackStatus, to: TrackStatus },
    Commit { id: String, hash: String, subject: String },
    LogLine { id: String, line: String },
    Aborted { id: String, reason: String },
}
```

## 5. Business rules (carried forward from bash kit)

All of these came from CodeRabbit-vetted findings in PR #7's six rounds:

- **`UNIQUE(module)` enforcement.** Two starts of the same module are rejected — must `cleanup` first.
- **Slug-boundary regex** for module references in `MODULES.md`: `(^|[^a-z0-9-])<slug>([^a-z0-9-]|$)` case-insensitive.
- **Whole-word dedupe** in `start` — `start auth Auth billing` rejects with `duplicate module(s) in selection: auth`.
- **`parallel.yaml` requires `version: 1`.**
- **Brownfield modules without `parallel.yaml` are rejected outright.** No heuristics.
- **`KIT_PARALLEL_MAX` defaults to 4** (env var only).
- **One harness per session.** `harness` fixed across one `start`.
- **Path-traversal guard.** `.worktreeinclude` entries are rejected if absolute or contain `..`; canonicalised via realpath; src+dst verified within their roots.
- **Orchestrator-commits canonical.** Track engine never has the harness commit; reads `proposed_commits` from the schema-validated report and applies them under the integrator.
- **Sentinel-watcher** marks pid-died tracks `aborted`; preserves branch + worktree.
- **Harness validation FIRST** — before any worktree state is created.
- **append_registry_entry returns errors** — never panics or `die`s; caller reaps the spawned child.
- **REGISTRY_LOCK lives at `<project>/.kit-workflow-app/.registry-lock`** — sibling of per-track locks, not inside (avoids slug collision with module named "registry").

## 6. Integration points

| Module | Relationship | Notes |
|---|---|---|
| session-store | Depends on | All persisted state |
| spec-engine | Depends on | Reads `MODULES.md` + per-module `parallel.yaml` |
| skill-runner | Used by | Every `/project-tracks` subcommand calls into this crate |
| claude-bridge | Indirect via DispatchFn | track-engine takes a closure; skill-runner provides one that routes to the right bridge |
| codex-bridge | Indirect via DispatchFn | Same |
| gui-shell | Subscribes to events | Plan board live updates |

## 7. Acceptance criteria

- [ ] `cargo test --package kit-track-engine` passes the **full bash-kit regression suite** (clean fixture, dep edge, slug boundary, dedupe lower, dedupe mixed-case, brownfield-without-yaml, claude-fails-fast, registry-write-failure-cleanup, harness-validates-first).
- [ ] Sentinel-watcher detects pid-died tracks within 5s.
- [ ] `merge` halts cleanly at first conflict with a `git mergetool` invocation printed.
- [ ] Concurrent `start` calls don't corrupt `tracks.json` (mkdir-based registry lock).
- [ ] Tauri events emitted on every status change.
- [ ] Per-track lock dir at `<project>/.kit-workflow-app/parallel/locks/<module>/`.

## 8. Out of scope

- Multi-track-per-module (winner-picking) — permanently out per bash kit decision.
- Auto-PR / auto-push.
- Auto-merge-conflict resolution.
- Cross-machine track distribution.

## 9. Open questions

- [ ] **Use `git2` crate or shell out to `git`?** `git2` is more robust + typed; shell-out is simpler for tooling parity. v1: `git2` for branch/worktree/rebase; shell-out for `git mergetool` (interactive).
- [ ] **Background sentinel-watcher** as a separate Tokio task or polled from the GUI? Separate task with mpsc channel.
- [ ] **Per-track cancellation tokens** for clean SIGTERM during shutdown.
