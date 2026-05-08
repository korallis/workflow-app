# codex-bridge — Implementation guide

## Patterns

- **Two crates**: `kit-codex-bridge` (Rust core, uses `codex-codes`) and `kit-codex-bridge-napi` (N-API wrapper). Pi extension imports the napi build.
- **`CodexTransport` trait** in the core crate isolates `codex-codes` so we can add a `CodexMcpTransport` later.
- **Approval handler is async.** N-API's threadsafe-function pattern lets the Rust side await a JS callback.
- **JSON Schema enforcement happens twice**: once via Codex's `--output-schema` flag (Codex enforces), once on the orchestrator side (we double-check). Defence in depth.

## Conventions

- Pi tool name: `kit_codex_execute` (snake_case to match Pi conventions).
- All filesystem paths absolute.
- Default sandbox: `workspace-write`. Document `danger-full-access` opt-in clearly.
- Subprocess cwd is `opts.cwd`; never inherit Pi's cwd.
- Report file extension: `-report.json`. Last-message file: `-last.md` (only if no schema; redundant with report otherwise).

## Module boundaries

This module **owns**: Codex subprocess lifecycle, JSON-RPC transport, schema enforcement, approval routing, scrubbing of Codex outputs.

This module **must NOT**: assemble prompts (kit-engine), persist state (session-store), commit code (orchestrator does).

## Known gotchas

- **Codex `mcp-server` is experimental.** Prefer `codex-codes` (app-server protocol) which is more stable.
- **Approval requests are server→client.** The bridge MUST register a callback handler; missing handler = silent denial = run aborts mysteriously.
- **`applyPatchApproval` for self-modifying runs.** When Codex modifies the kit itself, approval requests cluster. Default-deny is safest unless the user explicitly opted in.
- **Sandbox `workspace-write`** blocks `.git/index.lock`. This is by design — Codex doesn't commit; orchestrator does. Document loudly.
- **`codex-codes` API is sync OR async.** Use the async client (Tokio); the sync client doesn't compose with N-API threadsafe functions.

## Test patterns

- Mock Codex subprocess in unit tests; assert correct args.
- Real Codex run gated on `CODEX` being on `PATH` + `OPENAI_API_KEY` or `codex login` set.
- Approval-flow test: simulate an `applyPatchApproval` request; assert handler called; return both `allow` and `deny` paths.
- Schema-violation test: feed Codex a prompt that produces invalid JSON; assert `schema_violations` non-empty.
- Timeout test: 60s `on_approval` hang → auto-deny + abort.
