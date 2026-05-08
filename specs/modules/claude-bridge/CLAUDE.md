# claude-bridge — Implementation guide

## Patterns

- **`child_process.spawn`** for the Claude binary; never `exec` (avoids shell parsing).
- **JSONL parser** as a Node `Transform` stream; line-by-line, never buffer the full output.
- **Event normalisation** is a pure function: `normalise(rawEvent: any): ClaudeEvent` with exhaustive switch on `type` + `subtype`.
- **Read-path scrubbing** is a separate npm package (`@korallis/kit-scrub`) shared with codex-bridge.

## Conventions

- Pi extension entry: `packages/workflow-skills/extensions/claude-bridge/index.ts`.
- Subprocess args constant in `args.ts`; tested in isolation.
- All filesystem paths absolute; never relative.
- No `cwd` reliance for Claude invocation — pass `--cwd` explicitly.

## Module boundaries

This module **owns**: the Claude subprocess lifecycle, JSONL parsing, read-path scrubbing for Claude's outputs.

This module **must NOT**: assemble prompts (kit-engine does that), persist state (session-store does that), invoke Codex.

## Known gotchas

- **`--include-partial-messages`** dramatically increases event volume. UI can throttle; the bridge passes everything through.
- **Claude's `claude` binary path** — assume it's on `PATH`; surface a clear error if not (`Auth detection` module's job to prevent this earlier).
- **Stream-json events can have unknown types** (Claude updates ahead of our parser). Keep an "unknown" passthrough lane.
- **Exit code 124** = timeout from `coreutils timeout`. We're not using `timeout` in Node; we use Node's `child.kill('SIGTERM')` then `setTimeout(SIGKILL)`.
- **Stderr is mostly noise.** Capture it but only surface to user on non-zero exit.

## Test patterns

- **Mock the subprocess** in unit tests (record args, feed canned JSONL).
- **Real subprocess test** (gated on `CLAUDE` being on `PATH`) for end-to-end.
- **Parser fuzz test** against ~50 captured-real-world JSONL fixtures.
- **Lifecycle test:** spawn, kill mid-stream, assert cleanup.
