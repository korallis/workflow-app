# track-engine — Implementation guide

## Patterns

- **`DispatchFn` trait** is the seam between track-engine (orchestration) and the bridges (execution). track-engine never imports claude-bridge or codex-bridge.
- **Sentinel-watcher** as a Tokio task spawned from `start`; emits to an mpsc channel that the GUI subscribes to via Tauri events.
- **Git operations via `git2` crate** for branch/worktree/rebase; shell-out (`Command::new("git")`) only for the one-shot `git mergetool` invocation surfaced to the user.
- **mkdir-based locks** for both per-track lock dir and the registry mutex. Portable; no `flock` dependency.

## Conventions

- Crate name: `kit-track-engine`.
- Branches use `track/<module>` (NOT `worktree-<module>` — distinguishes kit tracks from ad-hoc Claude Code worktrees).
- Worktrees at `<project>/.kit-workflow-app/worktrees/track-<module>` (NOT `.claude/worktrees/` — that's Claude Code's namespace).
- Locks: `<project>/.kit-workflow-app/parallel/locks/<module>/` (per-track) and `<project>/.kit-workflow-app/parallel/.registry-lock/` (registry, sibling of `locks/`).
- All filesystem paths absolute; never relative.

## Module boundaries

This module **owns**: git worktrees, parallel-tracks state machine, merge sequencer, sentinel-watcher.

This module **must NOT**: spawn claude/codex (DispatchFn does it); persist learnings (skill-runner does it); render UI.

## Known gotchas (from bash kit, do NOT reintroduce)

These are all CodeRabbit findings from PR #7 — port-and-test rather than reinvent:

- **Substring slug match was the cause of false dependency edges.** Use the boundary regex.
- **Bash heredoc evaluated backticks.** Not applicable in Rust, but watch for similar string-escaping issues when generating prompts.
- **`with_registry_lock` released the lock before write.** Hold the lock through the critical section; in Rust use a guard struct (`RegistryLock`) with `Drop`.
- **`KIT_ALLOW_CONCURRENT=1` was set in track launches** — disabled the per-track lock. Don't.
- **`append_registry_entry` called `die` on errors** — bypassed the caller's cleanup. Return `Result`, never panic.
- **Harness validation hoisted to top.** Reject unsupported harness BEFORE any worktree state.
- **Path-traversal guard on `.worktreeinclude`.** Reject absolute + `..`-bearing entries; canonicalise both src and dst via `Path::canonicalize`.
- **`KIT_DISPATCH_TS` propagation.** Caller can pass a deterministic timestamp; track-engine threads it through to the dispatched harness.

## Test patterns

- **`tempfile::TempDir`** + `git2::Repository::init` for FS+git tests; isolated per-test.
- **Fixture project** with synthetic `MODULES.md` + per-module `parallel.yaml`s for plan validation.
- **Concurrent `start` test:** 4 parallel calls; assert exactly 4 rows in tracks; no duplicate module entries.
- **Sentinel-watcher test:** spawn a sleep subprocess, kill the pid, assert track flips to `aborted` within 5s.
- **Merge sequencer test:** 3 modules with deps `c → b → a`; merge order must be `[a, b, c]`.

## Cargo dependencies (load-bearing)

- `tokio` (full features)
- `git2` (worktrees, branches, rebase)
- `kit-spec-engine`, `kit-session-store` (sister crates)
- `serde` + `serde_json` + `serde_yaml` (DTOs)
- `chrono` (timestamps)
- `thiserror` (errors)
- `tracing` (structured logs)
