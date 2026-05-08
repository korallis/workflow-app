# claude-bridge — SPEC

> **Status:** Phase 5 of /project-init
> **Layer:** Engine (TypeScript Pi extension)
> **Last updated:** 2026-05-08

## 1. Purpose

Spawn Claude Code in headless mode (`claude --print --output-format stream-json --include-partial-messages`) and expose its execution as a Pi tool (`kit_claude_execute`). Parse the JSONL event stream into a normalised event shape that Pi can route. No Anthropic Rust SDK exists; this is the canonical integration path.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| CB-1 | `/project-module` skill | Run Claude on a module spec and stream events back | Single-harness implementation works without leaving Pi |
| CB-2 | `/project-execute` (Claude harness option, future) | Run Claude with the same dispatch contract as Codex | Harness choice is a flag, not a code path |
| CB-3 | GUI plan board | See live agent_message + file_change events | The pane is informative without raw JSON |
| CB-4 | Logs subsystem | Capture JSONL to `.kit-orchestration/<TS>.jsonl` while events route through Pi | Both human review and machine reanalysis are possible |

## 3. Public Pi tool

```typescript
kit_claude_execute(opts: {
  prompt: string;                                       // assembled by the caller (skill)
  cwd: string;
  model?: string;                                       // override
  permission_mode?: 'default' | 'acceptEdits' | 'bypassPermissions' | 'plan';
  timeout_seconds?: number;                             // default 1800
  ts?: string;                                          // KIT_DISPATCH_TS equivalent for filename determinism
}): ClaudeExecResult

interface ClaudeExecResult {
  ok: boolean;
  exit_code: number;
  log_path: string;
  jsonl_path: string;
  last_message_path: string;
  events: ClaudeEvent[];                                // full event log (also streamed live)
  proposed_commits?: ProposedCommit[];                  // populated if final message is JSON
}
```

## 4. Event shape (normalised)

```typescript
type ClaudeEvent =
  | { type: 'thread_started'; thread_id: string }
  | { type: 'turn_started' }
  | { type: 'turn_completed'; usage: { input: number; output: number; cached: number } }
  | { type: 'turn_failed'; error: { message: string } }
  | { type: 'tool_use'; tool: string; input: unknown; status: 'running' | 'completed' | 'failed'; output?: string }
  | { type: 'file_change'; kind: 'created' | 'modified' | 'deleted'; path: string }
  | { type: 'agent_message'; text: string }
  | { type: 'reasoning'; summary: string }              // when reasoning summaries enabled
  | { type: 'error'; message: string }
  | { type: 'api_retry'; attempt: number; max: number; delay_ms: number };
```

Mirrors the Codex event shape (codex-bridge) so kit-engine + GUI can treat them identically.

## 5. Business rules

- **Prompt is provided by the caller** (kit-engine assembles it). claude-bridge never interpolates context itself.
- **Read-path scrubbing:** `.jsonl` and `last-message` files pass through scrub-secrets before re-entering Pi context. Bundled scrubber crate (`kit-scrub`) ports the kit's bash `scrub-secrets.sh`.
- **Exit code propagation.** Non-zero exits surface as `ok: false`; the caller decides what to do.
- **Timeout enforcement.** SIGTERM after `timeout_seconds`; SIGKILL after +10s. Default 1800s.
- **Process lifecycle owned by claude-bridge.** No leaking child processes; cleanup on Pi shutdown.
- **No model defaulting.** Caller must pass `model` if they want to override the user's `~/.claude` settings.
- **Schema-validated output is OPTIONAL for Claude** (Anthropic doesn't ship `--output-schema`). When the caller wants structured reports, prompt them and validate after — best-effort.

## 6. Integration points

| Module | Relationship | Notes |
|---|---|---|
| kit-engine | Used by | Pi tool `kit_claude_execute` |
| codex-bridge | Sibling (mirror shape) | Event normalisation matches |
| context-mode-bridge | Adjacent | Tool outputs route through context-mode for free (it intercepts at the Pi tool layer) |
| session-store | None | claude-bridge is stateless |
| gui-shell | Indirect (via Pi events) | — |

## 7. Acceptance criteria

- [ ] `kit_claude_execute` round-trip on a fixture (echo prompt) within 5s.
- [ ] JSONL parser handles partial-message streaming (`--include-partial-messages`) without buffer overflow.
- [ ] All `ClaudeEvent` variants covered by tests; unknown event types pass through as `{ type: 'unknown'; raw: string }` rather than throwing.
- [ ] Timeout SIGTERM → SIGKILL works on Linux + macOS.
- [ ] `last_message_path` exists and is non-empty after a successful run.
- [ ] Read-path scrubber strips `sk-ant-...`, `sk-...`, `ghp_...`, `xox[abps]-...`, JWTs.
- [ ] Pi extension exposes `kit_claude_execute` and unloads cleanly on Pi shutdown.

## 8. Out of scope

- Claude API direct integration (subprocess only).
- Custom permission UI in v1 (relies on Claude's own approval flow when `permission_mode='default'`).
- Session resumption (Claude has its own; we don't proxy it in v1).
- File checkpointing (Claude SDK feature; we don't expose v1).

## 9. Open questions

- [ ] **Approval surfacing.** Claude's `permission_mode='default'` blocks on user approval; do we forward those prompts to the GUI, or require `acceptEdits` for orchestration runs?
- [ ] **Stream-json schema stability.** Per [issue #53516](https://github.com/anthropics/claude-code/issues/53516), schema isn't versioned. Pin Claude version at install or detect at startup?
