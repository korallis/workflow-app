# claude-bridge — Implementation guide

## Patterns

- **Tokio `Command`** for subprocess; `stdout`/`stderr` as `Pipe`; consume via `tokio::io::BufReader::lines()`.
- **JSONL parser** as a stream combinator: `lines.map(parse_event).buffered()`.
- **`on_event` callback** invoked for each parsed event; non-blocking; never panics inside the bridge.
- **Read-path scrubber** in a sibling crate (`kit-scrub`) shared with codex-bridge.

## Conventions

- Crate name: `kit-claude-bridge`.
- Subprocess args constants in `args.rs`; tested in isolation via `expect_args!`.
- All filesystem paths absolute; `opts.cwd` always passed via `--cwd`.
- Output paths derived from `opts.ts` when set (orchestrator-side determinism).

## Module boundaries

This module **owns**: Claude subprocess lifecycle, JSONL parsing, auth detection, read-path scrubbing of Claude's outputs.

This module **must NOT**: assemble prompts (skill-runner does that); persist state (session-store does); write commits.

## Known gotchas

- **`ANTHROPIC_API_KEY` overrides OAuth.** Onboarding must warn: when set, Claude Code uses API billing instead of Max. We can't unset env vars from a subprocess after spawn — must surface via UI.
- **`--bare` skips ALL `~/.claude` hooks and skills.** Document this in onboarding so users know per-project Claude Code customisations don't apply via our bridge.
- **`stdout` vs `stderr`.** `--output-format stream-json` writes JSONL to stdout; logs go to stderr. Capture both; surface stderr only on non-zero exit.
- **`--include-partial-messages` increases event volume dramatically.** UI throttles; bridge passes everything through.
- **Schema versioning of stream-json is not stable** ([anthropic/claude-code#53516](https://github.com/anthropics/claude-code/issues/53516)). Detect Claude version on startup; fail noisily on mismatch.

## Test patterns

- **Mock subprocess** via a trait + `MockClaude` in tests; assert correct args.
- **Real subprocess test** gated on `CLAUDE` on `PATH` + logged in.
- **Parser fuzz test** against captured JSONL fixtures (build a corpus from real runs).
- **Auth detect test:** mock `~/.claude/.credentials.json` content; assert correct `ClaudeInstall` variant.

## Cargo dependencies

- `tokio` (full), `tokio-stream`, `serde`, `serde_json`, `thiserror`, `tracing`, `chrono`
- Sister crate: `kit-scrub`
