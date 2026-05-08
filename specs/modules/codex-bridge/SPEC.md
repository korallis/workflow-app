# codex-bridge — SPEC

> **Layer:** Core (Rust crate `kit-codex-bridge`)
> **Last updated:** 2026-05-08 (Pi-removal pivot — was a Pi extension; now a Rust crate)

## 1. Purpose

Spawn Codex CLI with our system prompt + `--output-schema` for schema-validated reports + approval callbacks. **Uses the user's `codex login`** (ChatGPT Plus/Pro subscription, OAuth) — does not consume API credits. Same shape as claude-bridge for consistency in skill-runner.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| CXB-1 | skill-runner running `/project-execute` | Run Codex with `--output-schema codex-report-schema.json` | Final reports are JSON-validated; bleed-through fails structurally |
| CXB-2 | track-engine via DispatchFn | Spawn one Codex per track | Parallel tracks each have their own Codex subprocess |
| CXB-3 | gui-shell approval dialog | Receive `applyPatchApproval` and `execCommandApproval` requests | User clicks "allow"/"deny" and Codex continues |
| CXB-4 | onboarding | Detect Codex installed + `codex login` active + ChatGPT Plus subscription valid | ChatGPT Plus billing actually applies |

## 3. Public API

```rust
pub struct CodexBridge { /* config */ }

impl CodexBridge {
    pub fn detect() -> Result<CodexInstall>;     // version + auth + subscription tier

    pub async fn execute(&self, opts: CodexOpts) -> Result<CodexResult>;
}

pub struct CodexOpts {
    pub prompt: String,
    pub system_prompt: String,                      // assembled by skill-runner
    pub cwd: PathBuf,
    pub model: Option<String>,                      // default "gpt-5.5"
    pub reasoning_effort: ReasoningEffort,          // low/medium/high/xhigh, default medium
    pub sandbox: SandboxMode,                       // read-only / workspace-write / danger-full-access
    pub schema: Option<JsonSchema>,                 // forces --output-schema
    pub timeout: Duration,
    pub ts: String,                                 // KIT_DISPATCH_TS
    pub on_event: Box<dyn Fn(CodexEvent) + Send + Sync>,
    pub on_approval: Box<dyn Fn(ApprovalRequest) -> BoxFuture<'static, ApprovalDecision> + Send + Sync>,
}

pub struct CodexResult {
    pub ok: bool,
    pub exit_code: i32,
    pub log_path: PathBuf,
    pub jsonl_path: PathBuf,
    pub report_path: PathBuf,                       // schema-validated final message (when schema is set)
    pub events: Vec<CodexEvent>,
    pub proposed_commits: Vec<ProposedCommit>,
    pub schema_violations: Vec<String>,
}

pub enum ApprovalRequest {
    ApplyPatch { conversation_id: String, call_id: String, file_changes: Vec<FileChange>, reason: Option<String>, grant_root: bool },
    ExecCommand { conversation_id: String, call_id: String, command: String, cwd: PathBuf, reason: Option<String> },
}

pub enum ApprovalDecision { Allow, Deny }
```

## 4. Subprocess invocation

Two implementation paths, behind a `CodexTransport` trait:

| Transport | Pros | Cons |
|---|---|---|
| **`codex exec`** (subprocess + JSONL) | Same shape as claude-bridge; simple; well-tested | Approval callbacks via interactive prompts (not async — would need a wrapper) |
| **`codex-codes`** Rust crate (typed JSON-RPC over app-server) | Native async approval callbacks; richer event model | Newer crate; more code; couples to Codex app-server protocol |

**Decision:** Start with `codex exec` for parity with claude-bridge. Add the `codex-codes` transport in v1.1 for richer approval flows.

```bash
codex exec \
  -m gpt-5.5 \
  -c model_reasoning_effort=medium \
  -s workspace-write \
  --skip-git-repo-check \
  -C <opts.cwd> \
  --json \
  --output-schema <opts.schema_path> \
  -o <report_path> \
  - < <prompt-file>
```

System prompt is included in the prompt-file (Codex doesn't have `--append-system-prompt-file` like Claude; we inject ours at the top of the prompt and instruct the agent to treat it as the system context).

## 5. Final-report schema

Bundled at `crates/kit-codex-bridge/schemas/codex-report-schema.json`. Same shape as the bash kit's: `phases_completed`, `files_modified`, `files_created`, `tests_run`, `proposed_commits`, `deviations`, `open_questions`. `additionalProperties: false`. The orchestrator (skill-runner) cross-checks every field against `git status` / test output before applying commits.

## 6. Business rules

- **`codex exec` for v1; `codex-codes` for v1.1.** Behind a `CodexTransport` trait so swap is one impl swap.
- **Approval flow:** when Codex requests approval, bridge calls `opts.on_approval` (async closure). Default deny on missing handler.
- **Approval timeout 60s.** Auto-deny + abort run with clear error.
- **Read-path scrubbing** identical to claude-bridge (shared `kit-scrub`).
- **Schema enforcement is non-negotiable.** Runs without a valid JSON final message return `ok: false` + `schema_violations[]`.
- **Orchestrator-commits canonical.** Codex MUST NOT commit. Sandbox `workspace-write` blocks `.git/index.lock` anyway; report carries `proposed_commits`.
- **`KIT_DISPATCH_TS` propagation.**

## 7. Integration points

| Module | Relationship |
|---|---|
| skill-runner | Caller |
| track-engine | Indirect via DispatchFn |
| claude-bridge | Sibling — same event/result shape |
| `kit-scrub` | Shared scrubber |
| `codex-codes` (external crate) | v1.1 transport |
| context-mode-manager | Orthogonal |

## 8. Acceptance criteria

- [ ] `cargo test --package kit-codex-bridge` — args, scrubber, schema validation.
- [ ] Real subprocess test: round-trip prompt; report file exists and validates against schema.
- [ ] Schema-violation test: feed Codex a prompt that produces invalid JSON; assert `schema_violations` non-empty.
- [ ] Approval round-trip: simulate `applyPatchApproval`; on_approval fires; both `Allow` and `Deny` paths work.
- [ ] Approval 60s timeout → auto-deny + abort.
- [ ] `proposed_commits` parsed correctly.
- [ ] `KIT_DISPATCH_TS` makes filenames deterministic.
- [ ] Subprocess cleanup on app shutdown.
- [ ] Detect path: missing `codex login` → returns `NotLoggedIn` with login instructions.

## 9. Out of scope

- Codex web search live mode.
- Multi-turn `codex-reply` orchestration in v1 (each `execute` is one turn).
- Custom Codex profiles.

## 10. Open questions

- [ ] **Approval queueing.** If multiple approval requests fire simultaneously, do we queue or fan out to multiple GUI dialogs?
- [ ] **`codex-codes` migration timing.** v1 ships with `codex exec` subprocess; v1.1 adds the typed transport behind the trait.
- [ ] **`--output-last-message` vs `-o`.** When schema is set, the schema-validated content goes to `-o`. When schema is unset, do we fall back to `--output-last-message`? Probably yes, both should work.
