# codex-bridge — SPEC

> **Status:** Phase 5 of /project-init
> **Layer:** Engine (TypeScript Pi extension + Rust crate via N-API)
> **Last updated:** 2026-05-08

## 1. Purpose

Run Codex CLI (`gpt-5.5`) with **schema-validated final reports** and **approval callbacks** wired through to the GUI. Uses the typed [`codex-codes`](https://docs.rs/codex-codes) Rust crate over the Codex app-server JSON-RPC protocol. Insulates against the experimental status of `codex mcp-server`.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| CXB-1 | `/project-execute` skill | Run Codex on a module spec with `--output-schema codex-report-schema.json` | Final reports are JSON-validated; bleed-through fails structurally |
| CXB-2 | `/project-tracks start` (codex harness) | Spawn one Codex per track with per-track `KIT_DISPATCH_TS` + `KIT_PARALLEL_TRACK` | Each track's artefacts share a deterministic identifier |
| CXB-3 | GUI approval dialog | Receive `applyPatchApproval` and `execCommandApproval` requests via Pi events | User clicks "allow"/"deny" and the agent continues |
| CXB-4 | `/project-execute` orchestrator | Read scrubbed `-report.json` after the run + cross-check against `git status` | Spec deviations and bleed-through are caught before commit |

## 3. Public Pi tool

```typescript
kit_codex_execute(opts: {
  prompt: string;
  cwd: string;
  model?: string;                                       // default 'gpt-5.5'
  reasoning_effort?: 'low' | 'medium' | 'high' | 'xhigh';   // default 'medium'
  sandbox?: 'read-only' | 'workspace-write' | 'danger-full-access';   // default 'workspace-write'
  schema?: object;                                      // default codex-report-schema
  timeout_seconds?: number;                             // default 1800
  ts?: string;                                          // KIT_DISPATCH_TS for filename determinism
  on_approval?: (req: ApprovalRequest) => Promise<ApprovalDecision>;  // GUI dialog hook
}): CodexExecResult

interface CodexExecResult {
  ok: boolean;
  exit_code: number;
  log_path: string;
  jsonl_path: string;
  report_path: string;                                  // schema-validated final message
  events: CodexEvent[];
  proposed_commits: ProposedCommit[];                   // parsed from report
  schema_violations?: string[];                         // if final wasn't valid against schema
}

type ApprovalRequest =
  | { kind: 'apply_patch'; conversation_id: string; call_id: string; file_changes: FileChange[]; reason?: string; grant_root?: boolean }
  | { kind: 'exec_command'; conversation_id: string; call_id: string; command: string; cwd: string; reason?: string };

type ApprovalDecision = { decision: 'allow' | 'deny' };
```

## 4. Final-report schema (carried forward)

Embedded at `extensions/codex-bridge/schemas/codex-report-schema.json`. Same shape as the bash kit's `.claude/skills/project-execute/codex-report-schema.json`: `phases_completed`, `files_modified`, `files_created`, `tests_run`, `proposed_commits`, `deviations`, `open_questions`. `additionalProperties: false`. The orchestrator (kit-engine) cross-checks every field against `git status` / test output before applying commits.

## 5. Business rules

- **Use `codex-codes` Rust crate** as the primary transport (typed JSON-RPC, well-tested) wrapped via N-API. Fallback to `codex mcp-server` via `rmcp` if a future change makes it preferable; abstract behind a `CodexTransport` trait.
- **Approval flow:** when Codex requests `applyPatchApproval` / `execCommandApproval`, the bridge calls `opts.on_approval`. If absent, default to **deny** (safer than silent allow).
- **Read-path scrubbing** identical to claude-bridge (shared `kit-scrub` package).
- **`KIT_DISPATCH_TS` propagation.** When set, all output paths derive from it for orchestrator-side determinism.
- **Orchestrator-commits canonical.** Codex MUST NOT commit. Sandbox `workspace-write` blocks `.git/index.lock` anyway; report carries `proposed_commits` for the orchestrator to apply.
- **Schema enforcement is non-negotiable.** Runs without a valid JSON final message return `ok: false` + `schema_violations[]`.
- **Approval timeouts.** If `on_approval` doesn't resolve within 60s, deny + abort the run with a clear error.

## 6. Integration points

| Module | Relationship | Notes |
|---|---|---|
| kit-engine | Used by | Pi tool `kit_codex_execute` |
| claude-bridge | Sibling | Same `proposed_commits` + event shape |
| context-mode-bridge | Adjacent | Tool outputs route through |
| codex-codes (external crate) | Depends on | Primary transport |
| `kit-scrub` (sibling package) | Depends on | Read-path redaction |
| gui-shell | Indirect via Pi events + approval callbacks | — |

## 7. Acceptance criteria

- [ ] `kit_codex_execute` end-to-end against a fixture prompt; report file exists and validates against the schema.
- [ ] Schema violation produces `ok: false` + non-empty `schema_violations`.
- [ ] `applyPatchApproval` round-trip: bridge → `on_approval` → returned decision → Codex continues or aborts.
- [ ] Approval timeout (60s) → auto-deny + run abort.
- [ ] `proposed_commits` parsed correctly from the JSON report.
- [ ] `KIT_DISPATCH_TS` makes log/jsonl/report filenames deterministic.
- [ ] Subprocess cleanup on Pi shutdown.
- [ ] Read-path scrubber catches `sk-ant-...`, `sk-...`, `ghp_...`, etc. (shared with claude-bridge).
- [ ] Bridge handles `codex-codes` crate panics without taking down the Pi extension.

## 8. Out of scope

- Codex's web search live mode (use cached default).
- Multi-turn `codex-reply` orchestration in v1 (each `kit_codex_execute` is one turn).
- API-key-only mode without `codex login` (we still detect both auth paths in the auth-detect module).
- Custom Codex profiles (use defaults; revisit if users push for it).

## 9. Open questions

- [ ] **`codex mcp-server` vs `codex app-server`.** Both transports exist; `codex-codes` targets app-server. Confirm version compatibility with our pinned Codex CLI.
- [ ] **N-API panic safety.** If the Rust transport panics, does the Node thread survive? Need to verify and wrap with `catch_unwind`.
- [ ] **Approval queueing.** If multiple approval requests fire simultaneously (e.g. a parallel batch of file changes), do we queue or fan out to multiple GUI dialogs?
