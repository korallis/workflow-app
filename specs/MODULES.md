# MODULES — workflow-app

> **Status:** Phase 3 of /project-init
> **Last updated:** 2026-05-08

Nine cohesive modules. **All Rust** (except for the React frontend inside gui-shell). No Pi, no Node, no npm. The split separates concerns by domain so each crate can be tested in isolation and the GUI shell is thin.

> **Architectural pivot 2026-05-08:** The previous version of this document had Pi extensions in TypeScript. See `SPEC_REVISION_2026-05-08.md` for the rationale (Anthropic ToS finding + simpler distribution).

## Module list

| # | Name | Layer | Language | Depends on | Used by | Purpose |
|---|---|---|---|---|---|---|
| 1 | **session-store** | Core | Rust crate (rusqlite + FTS5) | — | track-engine, gui-shell, skill-runner | Per-project SQLite. Tracks registry, learnings (FTS5-indexed), spec snapshots. Replaces bash `tracks.json` + `LEARNINGS.md` grep. |
| 2 | **code-maps** | Core | Rust crate (tree-sitter `binding_rust`) | — | gui-shell, skill-runner | Compressed structural summaries (classes, functions, imports, exports). Token-efficient prompt context. Repo-Prompt-style. |
| 3 | **spec-engine** | Core | Rust crate | session-store | track-engine, skill-runner, gui-shell | Spec hierarchy CRUD: `PROJECT_BRIEF.md`, `MASTER_BLUEPRINT.md`, `MODULES.md`, `ROADMAP.md`, per-module `SPEC.md`/`CLAUDE.md`/`parallel.yaml`. Schema-validated. |
| 4 | **track-engine** | Core | Rust crate | session-store, spec-engine | skill-runner, gui-shell | Parallel-tracks state machine. Worktree management. Merge sequencer. Sentinel-watcher. Port of bash `project-tracks.sh` with all CodeRabbit-vetted invariants carried forward. |
| 5 | **claude-bridge** | Core | Rust crate (Tokio subprocess + JSONL parser) | — | skill-runner | Spawns `claude --print --bare --append-system-prompt-file <ours> --output-format stream-json --include-partial-messages`. Uses `claude login` → Max plan. Our system prompt; not Claude Code's defaults. ToS-clean. |
| 6 | **codex-bridge** | Core | Rust crate (Tokio subprocess; `codex-codes` for richer JSON-RPC) | — | skill-runner | Spawns Codex via `codex exec` (or `codex-codes` typed transport). Uses `codex login` → ChatGPT Plus. Schema-validated final reports (`codex-report-schema.json`). Approval callbacks. |
| 7 | **context-mode-manager** | Core | Rust crate (Tokio subprocess + `rmcp` MCP client) | — | skill-runner (transparently) | Spawns and supervises [`mksglu/context-mode`](https://github.com/mksglu/context-mode) as an MCP server sidecar. Routes tool outputs through `ctx_execute`/`ctx_search`. Optional (`KIT_CONTEXT_MODE_DISABLE=1` to skip). |
| 8 | **skill-runner** | Core | Rust crate; bundles markdown skills as embedded assets | spec-engine, track-engine, claude-bridge, codex-bridge, context-mode-manager, code-maps | gui-shell | Interprets the 12 kit slash commands (markdown files bundled in `skills/`). For each skill: assembles prompts (using code-maps), routes to claude-bridge or codex-bridge, processes results. The "agent loop" sits here. |
| 9 | **gui-shell** | Shell | Rust + Tauri 2.11 + React 19 + Vite + Tailwind v4 | All core modules (Tauri commands) | (the user) | Plan board, visual file picker + Code Maps, Apply Mode diff viewer, learnings browser, onboarding/auth detection. Single binary. |

## Dependency graph

```text
                    ┌──────────────────┐
                    │   gui-shell      │  (Tauri 2 + React, the user surface)
                    └────────┬─────────┘
                             │  Tauri commands + events
                             ▼
                    ┌──────────────────┐
                    │  skill-runner    │  (interprets bundled markdown skills,
                    │                  │   owns the agent loop)
                    └──┬──┬──┬──┬──┬───┘
                       │  │  │  │  │
              ┌────────┘  │  │  │  └────────────┐
              │           │  │  │               │
              ▼           ▼  ▼  ▼               ▼
         ┌──────────┐ ┌─────┐┌─────┐ ┌────────────────┐ ┌──────────┐
         │ spec-    │ │claud││codex│ │ context-mode-  │ │ code-    │
         │ engine   │ │ -    │ -   │ │ manager        │ │ maps     │
         │          │ │bridge│bridge │                │ │          │
         └────┬─────┘ └──┬──┘ └──┬──┘ └────────┬───────┘ └──────────┘
              │          │       │             │
              ▼          ▼       ▼             ▼  (sidecar)
         ┌─────────┐  spawn   spawn         spawn
         │ track-  │  claude  codex          context-mode
         │ engine  │  --print exec           (MCP server)
         └────┬────┘
              │
              ▼
       ┌──────────────┐
       │session-store │  (rusqlite + FTS5 + WAL)
       └──────────────┘

External dependencies (system):
  claude CLI (Claude Code 2.x; uses claude login → Max)
  codex CLI (0.128+; uses codex login → ChatGPT Plus)
  context-mode (npm-installed once; spawned as MCP server sidecar; optional)
```

## Module responsibilities — quick reference

### 1. session-store (Rust crate) — `crates/session-store/`
- rusqlite v0.31+ with bundled SQLite, WAL mode, busy_timeout 5s.
- FTS5 virtual table for `LEARNINGS` full-text + BM25.
- Tables: `tracks`, `track_events`, `learnings`, `learnings_fts`, `spec_snapshots`, `schema_version`.
- Migration system: versioned Rust functions, idempotent.
- Per-project DB at `<project>/.kit-workflow-app/state.db`. Global cross-project DB at `~/.kit-workflow-app/global.db` (for cross-project queries like "all my projects' open questions").
- `Mutex<Connection>` shared state pattern (per ELVES DECISIONS.md rationale).
- **Status:** scaffolded; 7 tests passing in `cargo test --workspace`.

### 2. code-maps (Rust crate)
- Tree-sitter `binding_rust`. Pre-built grammars for: rust, typescript, javascript, python, go, ruby (v1); java, c, cpp (v1.1).
- Public API: `generate_code_map(path: &Path, lang: Language) -> CodeMap` returning a struct with classes, functions, methods, imports, exports — code body stripped, signatures kept.
- Token-budget mode: prune lowest-priority entries until fit.
- Exposed via Tauri commands for the GUI and called directly from `skill-runner` for prompt assembly.

### 3. spec-engine (Rust crate)
Spec hierarchy CRUD. Loads, saves, and validates the project's spec tree:
- `specs/PROJECT_BRIEF.md`, `specs/RESEARCH.md`, `specs/MASTER_BLUEPRINT.md`, `specs/MODULES.md`, `specs/ROADMAP.md`
- `specs/modules/<name>/SPEC.md`, `CLAUDE.md`, `parallel.yaml`

Schema-validates `parallel.yaml` against `parallel.yaml.schema.json`. Persists `spec_snapshots` rows to `session-store` so `skill-runner` can resume from compaction. Does not directly call `claude` or `codex`; it's a pure data layer.

### 4. track-engine (Rust crate)
Parallel-tracks state machine. Direct port of the bash kit's `project-tracks.sh` with all CodeRabbit-vetted invariants carried forward:
- Worktree creation/cleanup (one git worktree per track).
- Merge sequencer (dependency-ordered rebase + apply).
- Sentinel-watcher: detects operator-killed panes; transitions to `aborted` state with branch+worktree preservation.
- Learnings-fragment merge: per-track `LEARNINGS-fragment.md` rolls up into root `LEARNINGS.md`.
- Reads dependency edges from `parallel.yaml` via `spec-engine`; reads/writes track state via `session-store`.

### 5. claude-bridge (Rust crate)
Tokio subprocess wrapper for `claude --print --bare --append-system-prompt-file <ours> --output-format stream-json --include-partial-messages`. Uses the user's `claude login` (Max plan) — ToS-clean because we invoke Claude Code itself, never ingesting its OAuth token. `--bare` skips Claude Code's hook/skill/MCP/CLAUDE.md auto-discovery so we own the context. Parses JSONL events (`thread.started`, `turn.completed`, `item.completed` with `command_execution`/`file_change`/`agent_message`) into a typed event stream.

### 6. codex-bridge (Rust crate) — direct OAuth, NOT a subprocess
Implements PKCE OAuth against `auth.openai.com` (same `CLIENT_ID` as Pi/Codex CLI), stores tokens in `~/.kit-workflow-app/auth.json`, calls Codex Responses API directly at `chatgpt.com/backend-api/codex/responses` with `store: false, stream: true, instructions: <ours>`. Uses ChatGPT Plus/Pro via OpenAI's Codex-for-OSS program (explicitly endorsed). Forces `--output-schema` semantics against `codex-report-schema.json` and surfaces approval callbacks (`applyPatchApproval`, `execCommandApproval`) to the GUI's approval dialog. The asymmetry vs claude-bridge is mandated by provider policies — see `SPEC_REVISION_2026-05-08.md`.

### 7. context-mode-manager (Rust crate)
Tokio subprocess + `rmcp` MCP client. Spawns and supervises [`mksglu/context-mode`](https://github.com/mksglu/context-mode) as an MCP server sidecar. Routes tool outputs through `ctx_execute` (sandbox) or `ctx_search` (FTS5 store) — typically a 90%+ context reduction. Optional: skipped when `KIT_CONTEXT_MODE_DISABLE=1`. Exposes `ctx_stats` for the GUI status bar.

### 8. skill-runner (Rust crate)
The agent loop. Bundles the 12 kit slash commands as embedded markdown assets (`crates/skill-runner/skills/<name>.md`):
`/project-init`, `/project-research`, `/project-blueprint`, `/project-spec`, `/project-module`, `/project-execute`, `/project-review`, `/project-security-review`, `/project-tracks`, `/project-status`, `/project-deploy`, `/project-test`. For each invocation: assembles prompts (using `code-maps` for context efficiency), routes to `claude-bridge` or `codex-bridge`, processes results, applies orchestrator-commits via `git2`. Top-level orchestrator that everything else feeds into.

### 9. gui-shell (Tauri 2 + React 19)
- Tauri v2.11 (latest stable). Single binary; system WebView; bundle ≤30 MB.
- Frontend: React 19 + TypeScript 5.8 (strict) + Vite 7 + Tailwind v4 (CSS-first config) + Zustand. (Stack ratified by ELVES DECISIONS.md.)
- xterm.js + portable-pty for the optional embedded agent stream.
- CodeMirror 6 for the diff viewer (Apply Mode).
- Communicates with the Rust core via Tauri commands and Tauri events — **no Pi**, no stdio RPC. The core crates are linked directly into `src-tauri`.
- Screens: Projects landing, Onboarding/Auth, Plan Board, File Picker + Code Maps, Apply Mode diff viewer, Learnings Browser, Settings.

## Module ordering & criticality

| Order | Module | Critical path? | Why |
|---|---|---|---|
| 1 | session-store | ✅ | Foundation: every other module reads/writes through it. **(scaffolded)** |
| 2 | code-maps | ✅ | Used for prompt context assembly and the GUI's file picker. Zero internal deps. |
| 3 | spec-engine | ✅ | Spec CRUD; needed by track-engine and skill-runner. Depends only on session-store. |
| 4 | track-engine | ✅ | Parallel-tracks logic. Depends on session-store + spec-engine. |
| 5 | claude-bridge | ✅ | Cheapest harness to wire (subprocess + JSONL parsing). Zero internal deps. |
| 6 | codex-bridge | ✅ | Needed for `/project-execute`. Direct OAuth flow; zero internal deps. |
| 7 | context-mode-manager |   | Dramatic context efficiency, but kit functions without it; cut from MVP if needed. |
| 8 | skill-runner | ✅ | The agent loop; pulls every other crate together. The point of the project. |
| 9 | gui-shell |   | Most user value but slowest to build; can ship CLI-only first. |

A v0.1 MVP could skip modules 7 + 9 and ship as a CLI binary (`kit`) driven by the user typing slash commands at a prompt — equivalent to the bash kit's UX but cross-platform and single-install. v0.2 adds context-mode-manager. v0.3 ships the GUI.

## Open questions

- Does the GUI use **Tauri events** for core→frontend notifications, or do we run an embedded **WebSocket** for richer pub/sub? (Tauri events are simpler; WebSocket lets browser-mode work later.)
- Should `code-maps` support **incremental updates** in v1, or full-repo rescans only? Incremental is faster on large repos but more complex.
- For **dogfooding**, the natural flow is to scaffold v0.1 manually (no kit), then start using v0.1 to build v0.2. When does the dogfood handover happen? Currently bootstrapping with the bash kit (`/project-execute` via Codex CLI subprocess) — see `LEARNINGS.md` for dispatch-infra discoveries.
- **Local vs global learnings split:** `session-store` exposes a per-project DB. The blueprint mentions a cross-project DB at `~/.kit-workflow-app/global.db` but no API yet distinguishes "local-only learning" vs "promote to global." To resolve when `spec-engine` or `skill-runner` first needs the cross-project query.
