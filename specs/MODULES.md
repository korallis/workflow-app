# MODULES — workflow-app

> **Status:** Phase 3 of /project-init
> **Last updated:** 2026-05-08

Eight cohesive modules. The split deliberately separates **engine** (Pi extensions, TypeScript) from **shell** (Tauri+Rust GUI) so the engine can run headless (TUI-only mode, CI mode, MCP-server mode) and the shell can stay thin.

## Module list

| # | Name | Layer | Language | Depends on | Used by | Purpose |
|---|---|---|---|---|---|---|
| 1 | **workflow-skills** | Engine | TypeScript (Pi prompts/skills) | (Pi) | All harness invocations | The kit's 12 slash commands as Pi prompt templates + `SKILL.md` files. Drop-in replacement for `.claude/skills/*` in the bash kit. |
| 2 | **kit-engine** | Engine | TypeScript Pi extension + native Rust crate (N-API) | session-store, code-maps | workflow-skills, gui-shell | Spec-engine (CRUD over `specs/*`), track-engine (parallel-tracks state, worktree management, merge sequencer). Replaces bash `dispatch.sh` + `project-tracks.sh`. |
| 3 | **codex-bridge** | Engine | TypeScript Pi extension + Rust crate (N-API) | (codex-codes crate) | workflow-skills, kit-engine | Spawns Codex via `codex-codes` typed transport. Schema-validated reports (`codex-report-schema.json`). Approval callbacks. |
| 4 | **claude-bridge** | Engine | TypeScript Pi extension | — | workflow-skills, kit-engine | Spawns `claude --print --output-format stream-json --include-partial-messages` and parses JSONL events. |
| 5 | **context-mode-bridge** | Engine | TypeScript Pi extension | (context-mode npm) | All harness invocations (transparently) | Manages [context-mode](https://github.com/mksglu/context-mode) as a sidecar MCP server. PreToolUse hook routes outputs through `ctx_execute`/`ctx_search`. |
| 6 | **session-store** | Engine | Rust crate (rusqlite + FTS5) | — | kit-engine, gui-shell | Per-project SQLite database. Tracks registry, learnings (FTS5-indexed), spec snapshots. Replaces `tracks.json` + `LEARNINGS.md` grep. |
| 7 | **code-maps** | Engine | Rust crate (tree-sitter binding_rust) | — | kit-engine, gui-shell | Compressed structural summaries: classes, functions, imports, exports. Token-efficient context for Pi prompts. Repo-Prompt-style. |
| 8 | **gui-shell** | Shell | Rust + Tauri 2 + React 19 + Vite + Tailwind v4 | All engine modules via Pi RPC + Tauri commands | (the user) | Plan board (parallel-tracks dashboard), visual file picker, Code Maps viewer, Apply Mode diff viewer, onboarding/auth detection screen. |

## Dependency graph

```text
                    ┌─────────────┐
                    │ gui-shell   │ (Tauri+React, the user surface)
                    └──┬─────┬────┘
                       │     │
                       ▼     ▼
              ┌─────────┐  ┌──────────┐
              │workflow-│  │kit-engine│ (Pi extensions + native Rust)
              │skills   │  └──┬──┬────┘
              └─────────┘     │  │
                              ▼  ▼
            ┌────────────┬─────────┬──────────────┐
            │codex-bridge│claude-  │context-mode- │
            │            │bridge   │bridge        │
            └─────┬──────┴──┬──────┴───────┬──────┘
                  │         │              │
                  ▼         ▼              ▼
              ┌─────────────────────────────┐
              │  session-store + code-maps  │ (Rust crates)
              └─────────────────────────────┘

External dependencies (system or npm):
  Pi (npm) | codex (CLI) | claude (CLI) | context-mode (npm) | tree-sitter (cargo)
```

## Module responsibilities — quick reference

### 1. workflow-skills (TypeScript)
The 12 kit slash commands (`/project-init`, `/project-research`, `/project-blueprint`, `/project-spec`, `/project-module`, `/project-execute`, `/project-review`, `/project-security-review`, `/project-tracks`, `/project-status`, `/project-deploy`, `/project-test`) as Pi prompt templates and `SKILL.md` files. Read by Pi from a versioned npm package `@korallis/workflow-skills`. Each skill mirrors the bash kit version 1:1, with content tweaks where the Pi context (vs Claude Code context) makes a difference.

### 2. kit-engine (TypeScript + native Rust)
Two sub-domains in one Pi extension:
- **Spec engine:** load/save/validate `specs/PROJECT_BRIEF.md`, `specs/RESEARCH.md`, `specs/MASTER_BLUEPRINT.md`, `specs/MODULES.md`, `specs/ROADMAP.md`, `specs/modules/*/SPEC.md|CLAUDE.md|parallel.yaml`. Schema validation via JSON Schema for `parallel.yaml`.
- **Track engine:** parallel-tracks state machine (port of bash `project-tracks.sh`). Worktree creation/cleanup. Merge sequencer (dependency-ordered rebase). Sentinel-watcher for operator-killed panes. Learnings-fragment merge.

Surface: Pi tools (e.g. `kit_track_plan`, `kit_track_start`, `kit_spec_load`, `kit_learnings_search`) + a native Rust crate via N-API for SQLite + tree-sitter calls.

### 3. codex-bridge (TypeScript + Rust)
Pi extension that exposes Codex execution as a Pi tool (`kit_codex_execute`). Internally uses [`codex-codes`](https://docs.rs/codex-codes) Rust crate via N-API for the JSON-RPC transport. Forces `--output-schema` against `codex-report-schema.json`. Surfaces approval callbacks (`applyPatchApproval`, `execCommandApproval`) to the GUI's approval dialog.

### 4. claude-bridge (TypeScript)
Pi extension that exposes Claude execution as a Pi tool (`kit_claude_execute`). Spawns `claude --print --output-format stream-json --include-partial-messages` as a child process, parses JSONL events (`thread.started`, `turn.completed`, `item.completed` with `command_execution`/`file_change`/`agent_message`), normalises into a Pi event stream.

### 5. context-mode-bridge (TypeScript)
Pi extension that:
1. On Pi startup, spawns context-mode as a managed MCP server subprocess.
2. Registers `ctx_*` tools with Pi.
3. Installs a PreToolUse hook that routes tool outputs through `ctx_execute` (sandbox) or `ctx_index` (FTS5 store).
4. Exposes `ctx_stats` to the GUI for context-saving telemetry.

### 6. session-store (Rust crate)
- rusqlite v0.31+ with bundled SQLite, WAL mode, busy_timeout 5s.
- FTS5 virtual table for `LEARNINGS` full-text + BM25.
- Tables: `tracks`, `track_events`, `learnings`, `learnings_fts`, `spec_snapshots`, `schema_version`.
- Migration system: versioned Rust functions, idempotent.
- Per-project DB at `<project>/.kit-workflow-app/state.db`. Global cross-project DB at `~/.kit-workflow-app/global.db` (for cross-project queries like "all my projects' open questions").
- Mutex<Connection> shared state pattern (per ELVES DECISIONS.md rationale).

### 7. code-maps (Rust crate)
- Tree-sitter `binding_rust`. Pre-built grammars for: rust, typescript, javascript, python, go, ruby (v1); java, c, cpp (v1.1).
- Public API: `generate_code_map(path: &Path, lang: Language) -> CodeMap` returning a struct with classes, functions, methods, imports, exports — code body stripped, signatures kept.
- Token-budget mode: prune lowest-priority entries until fit.
- Exposed via N-API for Pi extensions and Tauri commands for the GUI.

### 8. gui-shell (Tauri+React)
- Tauri v2.11 (latest stable).
- Frontend: React 19 + TypeScript 5.8 (strict) + Vite 7 + Tailwind v4 (CSS-first config) + Zustand for state. (Stack ratified by ELVES DECISIONS.md.)
- xterm.js for any embedded PTY views.
- CodeMirror 6 for the diff viewer (Apply Mode).
- Communicates with Pi via stdio RPC: spawns Pi at app start, holds a long-lived RPC connection, marshals responses to React via Tauri events.
- Screens: Projects landing, Onboarding/Auth, Plan Board, File Picker + Code Maps, Apply Mode diff viewer, Learnings Browser, Settings.

## Module ordering & criticality

| Order | Module | Critical path? | Why |
|---|---|---|---|
| 1 | session-store | ✅ | Foundation: every other module reads/writes through it. |
| 2 | code-maps | ✅ | Used by spec-engine for module dependency graphs and by the GUI's file picker. |
| 3 | kit-engine | ✅ | Drives everything. Without it, no workflow. |
| 4 | claude-bridge | ✅ | Cheapest harness to wire (just subprocess + JSONL parsing). |
| 5 | codex-bridge | ✅ | Needed for `/project-execute`. |
| 6 | workflow-skills | ✅ | The user-visible kit commands. Mostly content (markdown), so fast to land once 1-5 work. |
| 7 | context-mode-bridge |   | Adds dramatic context efficiency but kit functions without it; cut from MVP if needed. |
| 8 | gui-shell |   | Most user value but slowest to build; can ship engine-only TUI first. |

A v0.1 MVP could skip modules 7 + 8 and ship as a Pi-package only, driven by the user typing slash commands in Pi's interactive TUI — equivalent to the bash kit's UX but cross-platform. v0.2 adds context-mode-bridge. v0.3 ships the GUI.

## Open questions for Phase 4

- Does the GUI use **Tauri events** for Pi RPC notifications, or do we run an embedded **WebSocket** for richer pub/sub? (Tauri events are simpler; WebSocket lets browser-mode work later.)
- Should `code-maps` support **incremental updates** in v1, or full-repo rescans only? Incremental is faster on large repos but more complex.
- For **dogfooding**, the natural flow is to scaffold v0.1 manually (no kit), then start using v0.1 to build v0.2. When does the dogfood handover happen?
