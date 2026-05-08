# kit-engine — Implementation guide

## Patterns

- **Pi extension layout:** `packages/workflow-skills/extensions/kit-engine/index.ts` is the factory; `tools/`, `events/`, `helpers/` subfolders. Each `kit_*` tool is one file.
- **N-API bridge:** sibling `kit-engine-native` crate exposes session-store + code-maps. The TypeScript extension imports `require('./kit-engine-native')` once at load; all SQLite + tree-sitter calls go through it.
- **Schemas live as JSON files** at `extensions/kit-engine/schemas/` (`parallel.yaml.schema.json`, `learning-insert.schema.json`, etc.). Loaded once at startup via `ajv`.
- **Carry forward bash kit invariants** verbatim. Every CodeRabbit-vetted rule from the bash kit's six review rounds is encoded here. When a port is ambiguous, prefer the bash version's behaviour and add a comment citing the round.

## Conventions

- File naming: kebab-case for files, snake_case for Pi tool names.
- All tools return typed results; no untyped `any`.
- Errors are `Result`-style: `{ ok: false, error: string }` so Pi can surface them without crashing the harness.
- Branch naming: `track/<module>` (NOT `worktree-<module>` — distinguishes kit tracks from ad-hoc Claude Code worktrees).
- Lock paths: `.claude/parallel/locks/<module>/` for per-track locks; `.claude/parallel/.registry-lock/` for the registry mutex (sibling of `locks/`, not inside).

## Module boundaries

This module **owns**: spec validation, track lifecycle, learnings ingestion, Code Map orchestration on behalf of skills.

This module **must NOT**: spawn Claude/Codex directly (that's claude-bridge/codex-bridge); manage UI; manipulate the GUI's Tauri state.

This module **calls into**: session-store, code-maps (via N-API); codex-bridge, claude-bridge (as Pi tools).

## Known gotchas

- **Bash heredoc backtick bug** caught the kit in CodeRabbit's fourth pass. We don't have heredocs in TypeScript, but watch for similar string-escaping issues when generating prompts.
- **`with_registry_lock` released before write.** The bash kit's CodeRabbit critical finding. In TypeScript we use `try/finally` with explicit lock acquire/release; never rely on RAII or trap RETURN.
- **Substring slug matching** would re-introduce the "auth in oauth" false positive. Use the boundary regex from §5.
- **`KIT_ALLOW_CONCURRENT=1` was incorrectly set in the bash kit's track launches.** Don't set it. Per-track lock is the right namespace.
- **`append_registry_entry` must `return 1`, not `die`.** Caller wraps in try/catch and runs the kill-and-reap cleanup. Pi's process model is more forgiving but the principle stands: structured error handling, not crashes.
- **Harness validation early.** Reject claude/unknown harness BEFORE creating worktree state. The bash kit's fifth-pass finding.
- **Path traversal guard on `.worktreeinclude`.** Reject absolute + `..`-bearing entries; canonicalise both src and dst.

## Test patterns

- Pi extension tests use `vitest` with mocked Pi context.
- Use `node:fs` mocks for filesystem operations; never touch the real FS in unit tests.
- Integration tests spin up a `:memory:` session-store + a temp directory + git init; run a real `kit_track_start` against fixture modules; assert worktree creation + registry rows.
- Regression suite (port from bash kit's `project-tracks.sh` smoke tests): clean fixture, dep edge, slug boundary, dedupe, brownfield-without-yaml, claude-fails-fast.
