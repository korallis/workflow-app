# context-mode-manager — SPEC

> **Layer:** Core (Rust crate `kit-context-mode-manager`)
> **Last updated:** 2026-05-08 (Pi-removal pivot — was a Pi extension; now a Rust crate)

## 1. Purpose

Spawn and supervise [`mksglu/context-mode`](https://github.com/mksglu/context-mode) as an MCP server subprocess; connect via [`rmcp`](https://docs.rs/rmcp) MCP client; expose its 8 `ctx_*` tools to skill-runner so all tool outputs route through it for 98% context-window reduction. Surface `ctx_stats` telemetry to the GUI status bar.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| CMM-1 | skill-runner running a `bash` command via Claude/Codex | Have its output route through `ctx_execute` automatically | A 56KB log becomes a 299B summary |
| CMM-2 | skill-runner running `/project-research` | Use `ctx_fetch_and_index` to load Exa results into FTS5 | Future calls retrieve via BM25, not re-fetch |
| CMM-3 | gui-shell status bar | See live "context saved this session" telemetry | The user sees the value being delivered |
| CMM-4 | Power user | Disable context-mode for a session via `KIT_CONTEXT_MODE_DISABLE=1` | Debug raw outputs when something looks wrong |

## 3. Public API

```rust
pub struct ContextModeManager {
    rmcp_client: rmcp::Client,
    process: tokio::process::Child,
    config: ContextModeConfig,
}

impl ContextModeManager {
    pub async fn spawn(config: ContextModeConfig) -> Result<Self>;
    pub async fn shutdown(self) -> Result<()>;

    pub async fn ctx_execute(&self, lang: &str, code: &str) -> Result<CtxExecuteResult>;
    pub async fn ctx_index(&self, content: &str, source: &str) -> Result<CtxIndexResult>;
    pub async fn ctx_search(&self, queries: &[String]) -> Result<Vec<CtxSearchHit>>;
    pub async fn ctx_fetch_and_index(&self, urls: &[String]) -> Result<CtxFetchResult>;
    pub async fn ctx_batch_execute(&self, ops: Vec<BatchOp>) -> Result<CtxBatchResult>;
    pub async fn ctx_stats(&self) -> Result<CtxStats>;
    pub async fn ctx_purge(&self) -> Result<()>;

    pub async fn health(&self) -> HealthStatus;
}

pub struct ContextModeConfig {
    pub binary_path: PathBuf,                       // resolved by detect()
    pub data_dir: PathBuf,                          // ~/.kit-workflow-app/context-mode/
    pub disabled: bool,                             // KIT_CONTEXT_MODE_DISABLE=1
}
```

## 4. Lifecycle

```text
App startup
   └─ ContextModeManager::spawn(config)
         ├─ if config.disabled, return a no-op manager (all tools become identity)
         ├─ resolve binary (bundled or user-installed via npm)
         ├─ spawn `context-mode --mcp` subprocess (stdio transport)
         ├─ rmcp::Client::connect_stdio(child.stdin, child.stdout)
         ├─ MCP handshake (initialize + initialized)
         ├─ tools/list call → cache the 8 ctx_* tools
         ├─ start health task (poll every 30s)
         └─ return Manager

App shutdown
   └─ ContextModeManager::shutdown()
         ├─ flush rmcp client
         ├─ SIGTERM subprocess (10s grace)
         └─ SIGKILL on timeout
```

## 5. Business rules

- **Optional, not required.** `KIT_CONTEXT_MODE_DISABLE=1` returns a no-op manager. skill-runner doesn't crash; just gets raw outputs.
- **`rmcp` 1.6.0** is the MCP client. Apache-2.0. Official Anthropic SDK.
- **Health check.** Poll every 30s; restart on dead subprocess (max 3 restarts/session before warning + falling back to no-op mode).
- **Path:** `~/.kit-workflow-app/context-mode/` for SQLite content store.
- **Telemetry surfacing.** `ctx_stats` cached for 5s; GUI polls.
- **Subprocess crash handling.** Log + warn; attempt restart; fall back to no-op if 3 restarts fail.
- **Bundle vs detect.** v1: detect npm-installed `context-mode` on the system; install on first run if missing (with consent). v1.1: bundle pinned version.

## 6. Integration points

| Module | Relationship |
|---|---|
| skill-runner | Calls into; routes tool outputs |
| claude-bridge | Indirect — Claude's tools may end up calling context-mode if the user has it in their MCP config |
| codex-bridge | Indirect — same |
| gui-shell | Reads telemetry via `ctx_stats` |
| `rmcp` (external) | MCP client transport |

## 7. Acceptance criteria

- [ ] Manager spawns context-mode subprocess; MCP handshake completes; 8 tools available.
- [ ] `KIT_CONTEXT_MODE_DISABLE=1` returns a no-op manager that doesn't spawn anything.
- [ ] Subprocess crash → 1 restart → continue. After 3 restarts → no-op fallback + warning.
- [ ] `ctx_stats` returns valid telemetry; GUI shows "X KB saved this session".
- [ ] App shutdown cleanly reaps the subprocess.
- [ ] First-run user without context-mode installed gets a clear install guide (or auto-install with consent if bundled).
- [ ] All public methods are `Send + Sync` (used across Tauri command boundaries).

## 8. Out of scope

- Modifying context-mode itself (use upstream).
- Custom routing rules per project (use context-mode's own config).
- Multi-instance context-mode (one per app, not per track).

## 9. Open questions

- [ ] **Bundle vs install.** v1 detects user-installed; v1.1 bundles. When does the switch happen?
- [ ] **Per-project or global instance?** Global with project-tagged events.
- [ ] **Where does the install consent dialog live?** Onboarding UI; same place as Claude/Codex install detection.
