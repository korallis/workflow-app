# MASTER_BLUEPRINT — workflow-app

> **Status:** Phase 4 of /project-init
> **Owner:** Lee Barry
> **Last updated:** 2026-05-08
> **Depends on:** PROJECT_BRIEF.md, RESEARCH.md, MODULES.md

## 1. Project Overview

**Problem.** The bash AI Project Kit (`korallis/workflow`) implements a disciplined spec-first / dual-harness / parallel-tracks workflow but ships as shell scripts + markdown skills that only run inside Claude Code. Discoverability, context efficiency, parallel-tracks UX, auth onboarding, and compound learning all hit hard floors.

**Solution.** A single-install desktop app: a Tauri+Rust shell wrapping a [Pi](https://pi.dev/)-based engine. The engine is a TypeScript pi-package (`@korallis/workflow-skills`) plus a few Rust crates (session-store, code-maps) bridged via N-API. The shell is React + Tauri 2.11 with Repo-Prompt-style visual context engineering (file picker, Code Maps, Apply Mode) and a parallel-tracks plan board.

**Success criteria.** Per `PROJECT_BRIEF.md` §"Success Criteria".

**Scope.** Linux + macOS in v1; Windows in v1.1; mobile out of scope. Solo-dev focus; no team/cloud features.

## 2. Tech Stack

### Engine layer

| Layer | Technology | Version | Rationale |
|---|---|---|---|
| Agent harness | [Pi](https://pi.dev) (`@earendil-works/pi-coding-agent`) | latest npm | Minimal extensible coding agent; multi-provider via `pi-ai`; sessions JSONL-with-branching; explicitly designed to be extended with custom skills/extensions. Drop-in replacement for the bash kit's "Claude Code as the harness" model. |
| Pi extension language | TypeScript 5.8 | latest | Pi's extension API is TypeScript-native. |
| Pi runtime | Node.js | 22 LTS | Pi's runtime. Bundled with the Tauri shell to reduce install surface. |
| Codex transport | [`codex-codes`](https://docs.rs/codex-codes) Rust crate | latest | Typed JSON-RPC bindings for Codex's app-server protocol. More stable than `codex mcp-server` (which is experimental). Wrapped via N-API for Pi extension consumption. |
| Claude transport | `claude --print --output-format stream-json` (CLI) | Claude Code 2.x | No Rust SDK from Anthropic; headless mode is canonical. JSONL events parsed in TypeScript. |
| MCP client/server | [`rmcp`](https://docs.rs/rmcp) | 1.6.0 | Official Anthropic Rust MCP SDK. 9.4M downloads. Apache-2.0. Used both client-side (calling context-mode) and server-side (kit-as-MCP-server). |
| Storage | rusqlite | 0.31+ (bundled SQLite + FTS5 + WAL) | Per ELVES DECISIONS.md rationale: bundled = no system dep, FTS5 always available. |
| Code Maps | [tree-sitter](https://github.com/tree-sitter/tree-sitter) `binding_rust` | latest | Repo-Prompt-style structural summaries. 200+ language grammars on crates.io. |
| Context efficiency | [context-mode](https://github.com/mksglu/context-mode) | latest npm | MCP server, sandboxes tool output (98% reduction). MIT. Managed sidecar. |

### Shell layer

| Layer | Technology | Version | Rationale |
|---|---|---|---|
| Desktop framework | [Tauri](https://v2.tauri.app/) | 2.11.x | Stable since Oct 2024. Cross-platform. Apache-2.0/MIT. Small bundles via system WebView. MSRV 1.77.2. |
| Frontend | React | 19 | Mature, biggest ecosystem. ELVES uses it. |
| Frontend lang | TypeScript | 5.8 (strict) | Per ELVES strict config: `noImplicitReturns`, `noUncheckedIndexedAccess`, `noUnusedLocals`, `noUnusedParameters`. |
| Build tool | Vite | 7 | Required by Tauri 2 templates. Fast HMR. |
| Styling | Tailwind CSS | v4 (CSS-first config) | Per ELVES rationale: `@theme` tokens, no `tailwind.config.js`, simpler. |
| State | Zustand | latest | Slice-based; subscribe per-component, no re-render storm. |
| Diff viewer | CodeMirror 6 | latest | The Apply Mode surface needs a real code editor with diff lensing. |
| Embedded PTY | xterm.js + portable-pty | latest | Optional inline PTY for users who prefer the live agent stream. |
| Frontend tests | Vitest + jsdom + @testing-library/react | latest | Per ELVES DECISIONS.md: native Vite integration; faster than Jest. |
| Backend tests | `cargo test` | — | Standard. |
| IPC | Tauri events + Tauri commands | 2.x | Enough for v1; revisit WebSocket if browser-mode lands later. |

### Cross-cutting

| Concern | Choice | Notes |
|---|---|---|
| License | MIT | Pi-package + Tauri shell + Rust crates all MIT. |
| Distribution (engine) | npm: `@korallis/workflow-skills` | Pi-package, semver-tagged. |
| Distribution (shell) | GitHub Releases | Signed Tauri bundles for macOS (.dmg, .app), AppImage + .deb for Linux. Windows MSI in v1.1. |
| Auth | Detect-and-guide only | No token storage. Trusts `claude login`, `codex login`, env vars. |
| Telemetry | None | No phone-home; everything local. (`ctx_stats` is local-only.) |
| Updates | Tauri auto-updater plugin | Opt-in; signed update channel. |

### Version strategy

Pin major versions in `Cargo.toml` and `package.json`. Track upstream patch releases monthly. Codex MCP server interface is "experimental" — abstract behind a `CodexTransport` trait so we can swap when it stabilises.

## 3. Artefact model

```text
~/workflow-app/                                    (this repo)
├── packages/
│   └── workflow-skills/                           [npm: @korallis/workflow-skills]
│       ├── package.json                           pi-package manifest
│       ├── prompts/*.md                           prompt templates → /project-init etc.
│       ├── skills/*/SKILL.md                      Anthropic Agent Skills
│       └── extensions/                            TypeScript Pi extensions
│           ├── kit-engine/
│           ├── codex-bridge/
│           ├── claude-bridge/
│           └── context-mode-bridge/
├── crates/
│   ├── session-store/                             rusqlite + FTS5
│   ├── code-maps/                                 tree-sitter
│   ├── kit-mcp-server/                            optional rmcp server (kit-as-MCP)
│   └── kit-engine-native/                         N-API bindings for Pi extensions
├── apps/
│   └── gui-shell/                                 Tauri 2 desktop app
│       ├── src-tauri/                             Rust backend
│       ├── src/                                   React frontend
│       └── tauri.conf.json
├── specs/                                         the kit's own spec hierarchy
│   ├── PROJECT_BRIEF.md
│   ├── RESEARCH.md
│   ├── MODULES.md
│   ├── MASTER_BLUEPRINT.md                        (this file)
│   ├── ROADMAP.md
│   └── modules/<name>/{SPEC,CLAUDE}.md, parallel.yaml
├── .claude/                                       the kit (bash) installed for dogfood
└── package.json                                   workspace root
```

User-side state (not in repo):

```text
~/.kit-workflow-app/
├── global.db                                      cross-project queries
├── pi/                                            embedded Pi runtime
└── context-mode/                                  context-mode SQLite content store

<user-project>/
├── .kit-workflow-app/
│   ├── state.db                                   per-project tracks + learnings
│   ├── parallel/                                  worktree registry, locks
│   └── tracks/<TS>-<module>/                      per-track logs (gitignored)
└── specs/                                         user's spec hierarchy
```

## 4. Surface Area (Commands + Tools)

### User-facing slash commands (via Pi)

All 12 kit commands available via Pi's `/`-prefixed prompt templates:

| Command | Pi mechanism | Notes |
|---|---|---|
| `/project-init` | Prompt template + skill | Full 7-phase setup, identical to bash kit. |
| `/project-research` | Skill | Exa + Ref via `pi-ai` web tools. |
| `/project-blueprint` | Skill | — |
| `/project-spec` | Skill | — |
| `/project-module` | Skill | Single-harness via `claude-bridge` extension. |
| `/project-execute` | Skill | Dual-harness via `codex-bridge` extension. Schema-validated reports. |
| `/project-tracks` | Skill + native tool | `plan` + `start` from v1; `status`, `review`, `merge`, `cleanup` in v1.1. |
| `/project-review` | Skill | Updates SQLite-backed learnings (FTS5 indexed). |
| `/project-security-review` | Skill | Isolated review subagent via Pi spawn. |
| `/project-status` | Skill + native tool | Reads from SQLite. |
| `/project-deploy` | Skill | — |
| `/project-test` | Skill | — |

### Pi extension tools (machine-facing)

Exposed by Pi extensions to the agent:

| Tool | Module | Purpose |
|---|---|---|
| `kit_spec_load` / `kit_spec_save` | kit-engine | Spec file CRUD with schema validation |
| `kit_track_plan` / `kit_track_start` | kit-engine | Parallel tracks orchestration |
| `kit_learnings_search` | kit-engine | FTS5 query against learnings |
| `kit_codex_execute` | codex-bridge | Run Codex with schema-validated report |
| `kit_claude_execute` | claude-bridge | Run Claude in print mode |
| `kit_code_map` | code-maps | Generate tree-sitter Code Map |
| `ctx_*` (8 tools) | context-mode-bridge | Sandboxed execution + indexed retrieval |

### Tauri commands (GUI ↔ Rust backend)

Standard Tauri command pattern. GUI subscribes to Pi event stream via Tauri events.

| Command | Purpose |
|---|---|
| `pi_rpc_call(method, params)` | Forward an RPC call to Pi |
| `pi_event_subscribe()` | Subscribe to Pi event stream |
| `auth_status()` | Detect Claude/Codex/API-key state |
| `code_map_for_file(path)` | Direct tree-sitter call (cheaper than round-tripping through Pi) |
| `tracks_list()` | Read SQLite directly (read-only) |

## 5. Process model

```text
                     ┌──────────────────────────┐
                     │  gui-shell (Tauri app)   │
                     │  • React frontend        │
                     │  • Rust backend          │
                     │  • Tauri events + cmds   │
                     └──┬─────────────┬─────────┘
                        │             │
                Tauri command       Tauri spawn
                (sync calls)      (long-lived child)
                        │             │
                        ▼             ▼
            ┌──────────────────┐  ┌─────────────────────┐
            │  Rust backend    │  │  Pi (Node)          │
            │  (in-proc)       │  │  • interactive RPC  │
            │  • session-store │  │  • workflow-skills  │
            │  • code-maps     │  │    pi-package       │
            │  • auth detect   │  │  • extensions:      │
            │  • kit-mcp-      │  │    kit-engine,      │
            │    server (opt)  │  │    codex-bridge,    │
            │                  │  │    claude-bridge,   │
            │                  │  │    context-mode-    │
            │                  │  │    bridge           │
            └──────────────────┘  └──┬─────────┬────────┘
                                     │         │
                              spawn  │         │  spawn
                                     ▼         ▼
                             ┌─────────┐  ┌──────────────┐
                             │ codex   │  │ claude       │
                             │ (CLI    │  │ --print      │
                             │  app-   │  │ stream-json  │
                             │  server)│  │              │
                             └─────────┘  └──────────────┘
                                     │         │
                              ┌──────▼─────────▼─────┐
                              │ context-mode (MCP    │
                              │ server, also spawned │
                              │ by Pi extension)     │
                              └──────────────────────┘
```

**Lifecycle.**
1. App starts → Tauri spawns Pi as a long-lived child via `pi --mode rpc`.
2. Pi starts → loads `@korallis/workflow-skills` from bundled `node_modules`; extensions register tools.
3. context-mode-bridge extension spawns context-mode subprocess + registers its tools with Pi.
4. User opens a project → Tauri reads `<project>/.kit-workflow-app/state.db` (creates if absent), Pi `cd`s to project.
5. User triggers `/project-execute auth` → Pi runs the skill → skill calls `kit_codex_execute` → codex-bridge spawns Codex via codex-codes → events stream back to Pi → Pi forwards over RPC → Tauri event → React UI updates.
6. App closes → Pi receives RPC close → Pi shuts down gracefully (saves session JSONL) → Tauri waits up to 5s, then SIGTERM, then SIGKILL.

## 6. Spec Hierarchy invariants

The bash kit's three-tier hierarchy is unchanged:

1. **Blueprint (this file).** System design.
2. **Module Spec** (per-module SPEC.md). Detailed contracts.
3. **Code.** Implementation.

Plus a fourth on-disk artefact unique to the bash kit and ported here: `parallel.yaml` per module declares track shape (`version: 1`, `touches:`, `shared:`, `ports:`, `migrations:`).

Compound learning is unchanged in shape: per-track fragments at `.claude/parallel/learnings/<module>.md`; integrator merges into root `LEARNINGS.md` after track merge. **New in workflow-app:** `LEARNINGS.md` is also indexed into SQLite FTS5 so the GUI can offer search.

## 7. Security & Privacy

| Concern | Mitigation |
|---|---|
| Token theft | App never stores tokens. Detection-only auth; relies on OS keychain via Claude/Codex CLIs. |
| Sandbox escape (Codex) | `KIT_CODEX_SANDBOX=workspace-write` default; orchestrator-commits pattern (Codex doesn't touch `.git/`). Per-track `parallel.yaml` can opt to `danger-full-access` for explicit cases. |
| Tool output containing secrets | Read-path scrubbing via the kit's `scrub-secrets.sh` (ported as a Rust crate or bundled binary). All `-report.json` / `.jsonl` / `.log` artefacts pass through scrubber before re-entering Pi/agent context. |
| `.git/index.lock` write blocked by Codex sandbox | Orchestrator-commits canonical (per the kit's bash retro). GUI shows the proposed commit list; user clicks "Apply" to commit on the main thread. |
| Self-modifying engine | Pi extensions can be installed via `pi install npm:...` — review-the-source warning shown in the GUI. |
| Compaction-induced amnesia | context-mode + session-store + `LEARNINGS` FTS5 give multiple recovery paths. |
| Update channel compromise | Tauri auto-updater is opt-in; signed updates only; release channel separate from `main`. |

UK GDPR / healthcare / OWASP concerns apply only to *projects built with the kit*, not to the kit itself; `/project-security-review` skill addresses them.

## 8. Open Questions

These didn't have to be resolved for the blueprint to ratify, but should be answered before module specs are frozen:

- [ ] **Bundle Node 22 with the Tauri app?** Pros: zero-install. Cons: ~50MB bundle weight. Alt: detect existing Node ≥22 and prompt to install if missing.
- [ ] **Bundle Pi or fetch on first run?** Bundling is reproducible; fetching keeps Pi current. Probably bundle a pinned version, surface "update Pi" in Settings.
- [ ] **Bundle context-mode or fetch?** Same calculus; context-mode v1.0.x is small.
- [ ] **Single workspace or true monorepo?** This repo would have `packages/`, `crates/`, `apps/` at top level — npm workspaces + Cargo workspace. Cleaner but heavier tooling.
- [ ] **First-class `pi --mode rpc` or fork to expose richer protocol?** Pi's RPC mode is documented but limited; we may need to PR a richer protocol upstream.
- [ ] **N-API vs separate process for Rust crates from Pi?** N-API is faster but couples versions; subprocess is decoupled but slower.

## 9. References

Same set as RESEARCH.md §References. Most-load-bearing for the blueprint:

- Pi: https://pi.dev / https://github.com/earendil-works/pi
- Tauri 2: https://v2.tauri.app
- rmcp: https://docs.rs/rmcp
- codex-codes: https://docs.rs/codex-codes
- context-mode: https://github.com/mksglu/context-mode
- Tree-sitter: https://github.com/tree-sitter/tree-sitter
- AI Project Kit (this kit, the foundation): https://github.com/korallis/workflow
