# codex-bridge — Implementation guide

## Patterns

- **`CodexTransport` trait** isolates the underlying transport. v1 impl: `CodexExecTransport` (subprocess + JSONL). v1.1 impl: `CodexCodesTransport` (typed JSON-RPC via the `codex-codes` crate).
- **Tokio `Command`** for `codex exec`; same shape as claude-bridge.
- **Approval handler is async.** v1 impl polls Codex's interactive approval prompts in stderr (best-effort); v1.1 (codex-codes) gets native callbacks.
- **JSON Schema enforcement happens twice:** Codex via `--output-schema`, AND skill-runner cross-checks against repo state.

## Conventions

- Crate name: `kit-codex-bridge`.
- Default sandbox: `workspace-write`. Document `danger-full-access` opt-in clearly.
- Subprocess cwd is `opts.cwd`; never inherit.
- Report file extension: `-report.json`.
- Schema bundled via `include_str!`.

## Module boundaries

This module **owns**: Codex subprocess lifecycle, transport (subprocess or app-server), schema enforcement, approval routing, scrubbing of Codex outputs.

This module **must NOT**: assemble prompts (skill-runner); persist state (session-store); commit code (skill-runner does, after cross-check).

## Known gotchas

- **`codex exec` approval prompts are interactive.** v1 implementation either runs in `--full-auto` (no approvals) or uses a wrapper that intercepts stderr. v1.1 with codex-codes solves this properly.
- **Sandbox `workspace-write` blocks `.git/index.lock`.** This is by design — Codex doesn't commit; orchestrator does.
- **`--output-schema` requires a JSON Schema file path, not inline.** Write opts.schema to a temp file before invocation.
- **Schema feature interaction:** `--output-schema` + `-o <file>` writes the schema-conformant final message to `<file>`; `--json` + stdout streams events. We use both.

## Test patterns

- Mock subprocess in unit tests; assert correct args.
- Real Codex run gated on `CODEX` on PATH + `codex login` set.
- Approval-flow test (v1.1): simulate request; assert handler called; both paths.
- Schema-violation test: feed bad-JSON-producing prompt; assert violations.
- Timeout test.

## Cargo dependencies

- `tokio`, `serde`, `serde_json`, `jsonschema`, `thiserror`, `tracing`
- v1.1 only: `codex-codes`
- Sister crate: `kit-scrub`
