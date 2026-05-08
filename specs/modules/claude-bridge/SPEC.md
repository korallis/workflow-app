# claude-bridge — SPEC

> **Layer:** Core (Rust crate `kit-claude-bridge`)
> **Last updated:** 2026-05-08 (Pi-removal pivot — was a Pi extension; now a Rust crate)

## 1. Purpose

Spawn Claude Code in headless `--bare` mode with our own system prompt and parse its JSONL event stream. **Uses the user's `claude login`** (Claude Pro/Max plan, OAuth) — does not consume API credits. ToS-clean: we invoke Claude Code itself, exactly the path Anthropic carves out in [their auth/credential policy](https://code.claude.com/docs/en/legal-and-compliance#authentication-and-credential-use).

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| CB-1 | skill-runner | Run a skill backed by Claude with our system prompt | The agent has our brand/identity, not Claude Code's defaults |
| CB-2 | skill-runner | Stream JSONL events back through Tauri events | The GUI shows live progress |
| CB-3 | onboarding | Detect Claude Code is installed + `claude login` is active + `ANTHROPIC_API_KEY` is unset | Max billing actually applies |
| CB-4 | track-engine via DispatchFn | Spawn one claude per track | Parallel tracks each have their own subprocess |

## 3. Public API

```rust
pub struct ClaudeBridge { /* config, paths */ }

impl ClaudeBridge {
    pub fn detect() -> Result<ClaudeInstall>;     // version + auth + ANTHROPIC_API_KEY warning

    pub async fn execute(&self, opts: ClaudeOpts) -> Result<ClaudeResult>;
}

pub struct ClaudeOpts {
    pub prompt: String,
    pub system_prompt_path: PathBuf,                // mandatory; --append-system-prompt-file
    pub cwd: PathBuf,
    pub model: Option<String>,                      // override (default: user's settings)
    pub allowed_tools: Vec<String>,                 // e.g. ["Read", "Edit", "Bash"]
    pub permission_mode: PermissionMode,            // default | acceptEdits | dontAsk | bypassPermissions | plan
    pub timeout: Duration,                          // default 1800s
    pub ts: String,                                 // KIT_DISPATCH_TS for filename determinism
    pub on_event: Box<dyn Fn(ClaudeEvent) + Send + Sync>,    // streaming hook
}

pub struct ClaudeResult {
    pub ok: bool,
    pub exit_code: i32,
    pub log_path: PathBuf,                          // raw .log
    pub jsonl_path: PathBuf,                        // raw JSONL events
    pub last_message_path: PathBuf,
    pub events: Vec<ClaudeEvent>,
    pub proposed_commits: Vec<ProposedCommit>,      // parsed from final message if JSON
    pub schema_violations: Vec<String>,             // empty when final message validates
}
```

## 4. Subprocess invocation

```bash
claude --print \
  --bare \
  --append-system-prompt-file <our-system-prompt.md> \
  --output-format stream-json \
  --include-partial-messages \
  --cwd <opts.cwd> \
  --allowedTools <opts.allowed_tools.join(",")> \
  [--permission-mode <opts.permission_mode>] \
  [--model <opts.model>]
```

`--bare` is critical: it skips Claude Code's auto-discovery of hooks, skills, plugins, MCP servers, auto-memory, and CLAUDE.md. We get a clean baseline; our own `--append-system-prompt-file` is the only context.

## 5. Event shape (normalised)

Mirrors codex-bridge's event shape for consistency in skill-runner:

```rust
pub enum ClaudeEvent {
    ThreadStarted { thread_id: String },
    TurnStarted,
    TurnCompleted { input_tokens: u32, output_tokens: u32, cached: u32 },
    TurnFailed { message: String },
    ToolUse { tool: String, input: serde_json::Value, status: ToolStatus, output: Option<String> },
    FileChange { kind: FileChangeKind, path: PathBuf },
    AgentMessage { text: String },
    Reasoning { summary: String },
    Error { message: String },
    ApiRetry { attempt: u32, max: u32, delay_ms: u32 },
}
```

## 6. Auth detection

`ClaudeBridge::detect()` checks:

| Check | Action on failure |
|---|---|
| `claude --version` exits 0 | Return `ClaudeInstall::NotInstalled` with install instructions |
| `claude login` status (parsed from `~/.claude/.credentials.json`) | Return `ClaudeInstall::NotLoggedIn` with login instructions |
| `ANTHROPIC_API_KEY` env var | Return `ClaudeInstall::ApiKeyOverride` with **warning** that Max billing won't apply unless this is unset |
| Version compatibility (Claude Code 2.x) | Warn but allow |

Onboarding (gui-shell) calls this and surfaces the appropriate UX.

## 7. Business rules

- **`--bare` is mandatory.** Never invoke `claude --print` without it from this crate. Otherwise Claude Code's hooks/CLAUDE.md/MCP discovery contaminate our run.
- **System prompt via `--append-system-prompt-file`** (not `--append-system-prompt`). The latter has shell-quoting fragility on long prompts.
- **Per-call subprocess.** No long-running daemon. Each `execute` spawns and reaps. Simpler lifecycle, smaller state surface.
- **Read-path scrubbing.** `.jsonl` and last-message files pass through `kit-scrub` before re-entering Rust string state. Catches `sk-ant-...`, `sk-...`, `ghp_...`, `xox[abps]-...`, JWTs.
- **Timeout enforcement.** `Tokio::time::timeout` around the subprocess; SIGTERM → 10s grace → SIGKILL.
- **Working directory.** Always `--cwd opts.cwd`; never inherit the app's cwd.
- **`KIT_DISPATCH_TS`.** When `opts.ts` is set, all output filenames use it for orchestrator-side determinism.

## 8. Integration points

| Module | Relationship |
|---|---|
| skill-runner | Caller |
| track-engine | Indirect via DispatchFn; one `execute` per track |
| codex-bridge | Sibling — same event/result shape |
| kit-scrub (sibling crate) | Read-path scrubbing |
| context-mode-manager | Orthogonal — context-mode runs as MCP server; Claude calls it via its own MCP client surface (when `KIT_CONTEXT_MODE_DISABLE` is unset) |

## 9. Acceptance criteria

- [ ] `cargo test --package kit-claude-bridge` — unit tests for arg construction, event normalisation, scrubber.
- [ ] Real subprocess test (gated on `claude` being on PATH + logged in): echo prompt round-trip in <5s.
- [ ] `--bare` is in every invocation (asserted by integration test inspecting subprocess args).
- [ ] System prompt does NOT include "You are Claude Code" string when produced via this bridge.
- [ ] JSONL parser handles partial-message streaming.
- [ ] Timeout SIGTERM → SIGKILL works on Linux + macOS.
- [ ] Detect path: ANTHROPIC_API_KEY present → returns `ApiKeyOverride` warning.

## 10. Out of scope

- Claude API direct integration (subprocess only — by design).
- Rust SDK integration (Anthropic doesn't ship one).
- Custom approval UI (use Claude Code's `permission_mode='acceptEdits'` for orchestration runs; interactive approvals not in v1).
- Session resume across runs in v1.

## 11. Open questions

- [ ] **`--mcp-config` opt-in.** Should our bridge inject a custom `--mcp-config` (e.g. to reach context-mode)? Or let Claude's user-global config handle it? v1: pass nothing; rely on user setup.
- [ ] **`--permission-mode dontAsk`** vs. fine-grained `--allowedTools`. Per Anthropic docs, allowedTools is preferred. Default to that.
