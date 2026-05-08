# kit-engine — SPEC

> **Status:** Phase 5 of /project-init
> **Layer:** Engine (TypeScript Pi extension + native Rust crate)
> **Last updated:** 2026-05-08
> **Depends on:** session-store, code-maps

## 1. Purpose

The **brains** of the workflow-app. Exposes the kit's spec hierarchy and parallel-tracks engine to Pi as tools. Two cohesive subsystems in one Pi extension:

- **Spec engine:** load/save/validate the spec hierarchy (`PROJECT_BRIEF.md`, `RESEARCH.md`, `MASTER_BLUEPRINT.md`, `MODULES.md`, `ROADMAP.md`, `specs/modules/<name>/SPEC.md|CLAUDE.md|parallel.yaml`).
- **Track engine:** port of the bash kit's `project-tracks.sh` — parallel-tracks state machine, worktree management, merge sequencer, sentinel-watcher, learnings-fragment merge.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| KE-1 | Pi running `/project-spec auth` | Read `MASTER_BLUEPRINT.md` + write a new module SPEC + validate `parallel.yaml` schema | Spec authoring is governed by typed contracts, not freeform markdown |
| KE-2 | Pi running `/project-tracks plan auth billing` | Read `MODULES.md` + per-module `parallel.yaml`s, validate dep edges + shared paths, return a topological merge order | Plans are deterministic and rejected when invalid |
| KE-3 | Pi running `/project-tracks start` | Create git worktrees, register tracks in SQLite, dispatch one harness per track, watch sentinels | Multiple modules run in parallel without conflict |
| KE-4 | Compound-learning skill | Append a learning fragment with module + tags + commit hash | The fragment is FTS5-indexed for future queries |
| KE-5 | GUI plan board | Subscribe to track-state changes via Pi events | The dashboard is live |

## 3. Public Pi tools

```typescript
// Spec engine
kit_spec_load(path: string): SpecDoc
kit_spec_save(path: string, body: string): SaveResult
kit_spec_validate_parallel_yaml(yaml: string): ValidationResult

// Track engine
kit_track_plan(modules: string[], harness: 'codex'|'claude'): TrackPlan
kit_track_start(plan: TrackPlan): TrackStartResult
kit_track_status(filter?: TrackFilter): Track[]
kit_track_review(module: string): ReviewResult        // wraps /project-review --isolate
kit_track_merge(): MergeResult                         // dependency-ordered rebase + apply
kit_track_cleanup(modules?: string[]): CleanupResult
kit_track_abort(module: string, reason: string): AbortResult

// Learnings
kit_learning_append(l: LearningInsert): { id: number }
kit_learnings_search(q: string, opts?: SearchOpts): LearningHit[]

// Code maps (delegates to code-maps crate via N-API)
kit_code_map(path: string): CodeMap
kit_code_map_directory(root: string, opts?: ScanOpts): CodeMap[]
```

## 4. Data contracts

```typescript
type Harness = 'codex' | 'claude';
type TrackStatus = 'pending' | 'running' | 'completed' | 'timeout' | 'aborted';

interface ParallelYaml {
  version: 1;
  touches: string[];                                    // glob patterns
  shared?: string[];                                    // paths that may collide with other tracks
  ports?: number[];
  migrations?: boolean;
}

interface TrackPlan {
  modules: string[];                                    // topologically sorted
  harness: Harness;
  shared_paths_warning?: string[];
}

interface Track {
  id: string;
  module: string;
  branch: string;                                       // 'track/<module>'
  worktree: string;                                     // '.claude/worktrees/track-<module>'
  harness: Harness;
  port?: number;
  status: TrackStatus;
  started_at: string;                                   // ISO 8601
  completed_at?: string;
  last_commit?: string;
  pid?: number;
}
```

`parallel.yaml` schema validated via JSON Schema (bundled at `extensions/kit-engine/schemas/parallel.yaml.schema.json`).

## 5. Business rules

Carried forward from the bash kit's CodeRabbit-vetted decisions:

- **`UNIQUE(module)` enforcement** on track inserts (session-store enforces at SQL layer).
- **Slug-boundary regex** for module references in `MODULES.md`: `(^|[^a-z0-9-])<slug>([^a-z0-9-]|$)` case-insensitive. Prevents "auth" matching "oauth".
- **Whole-word dedupe.** `/project-tracks start auth Auth billing` rejects with `duplicate module(s) in selection: auth`.
- **`parallel.yaml` requires `version: 1`.** Files without it are rejected.
- **Brownfield modules without `parallel.yaml` are rejected** — explicit, no heuristic.
- **`KIT_PARALLEL_MAX` defaults to 4** (env var only; not in settings.json).
- **One harness per session** — `harness` is fixed for a `start` invocation; mixed harnesses rejected.
- **Path-traversal guard.** `.worktreeinclude` entries are rejected if absolute or contain `..`; canonicalised via realpath; src+dst verified within their roots.
- **Orchestrator-commits canonical.** Track engine never has the harness commit; it reads `proposed_commits` from the schema-validated report and applies them.
- **Sentinel-watcher** marks pid-died tracks as `aborted`; preserves branch + worktree; cleanup is explicit.
- **Self-relocation pattern** for the dispatcher is no longer needed (we don't shell out to a self-modifiable bash script — Pi extensions are JS).

## 6. Integration points

| Module | Relationship | Notes |
|---|---|---|
| session-store | Depends on (write+read) | All persisted state |
| code-maps | Depends on (read) | Generates Code Maps for prompt assembly |
| codex-bridge | Calls into | `kit_track_start` invokes `kit_codex_execute` per track |
| claude-bridge | Calls into | Single-harness `/project-module` invokes `kit_claude_execute` |
| context-mode-bridge | Adjacent | context-mode runs alongside; not directly called by kit-engine |
| workflow-skills | Used by | Every kit slash command calls one or more `kit_*` tools |
| gui-shell | Used by (read + RPC) | Plan board reads `kit_track_status`; Apply Mode reads proposed_commits |

## 7. Acceptance criteria

- [ ] All 12 kit slash commands (`/project-init` through `/project-test`) invoke the right `kit_*` tools.
- [ ] `kit_track_plan` regression suite (clean fixture, dep edge, slug boundary, dedupe, brownfield-without-parallel-yaml) passes — equivalent to bash kit's regression suite.
- [ ] Sentinel-watcher detects pid-died tracks within 5s.
- [ ] `kit_track_merge` halts cleanly at first conflict with actionable resolution steps.
- [ ] `kit_learnings_search "tauri"` returns top-10 hits in <50ms.
- [ ] `kit_code_map` for a 50-file repo completes in <2s.
- [ ] N-API bindings to session-store + code-maps work without panic on malformed input.
- [ ] Pi extension loads in <500ms on cold start.

## 8. Out of scope

- Multi-track-per-module winner-picking (per the bash kit decision; permanently out).
- Auto-PR / auto-push (orchestrator commits locally; user pushes).
- Auto-merge-conflict resolution (human integrator required).
- Cross-machine track distribution.

## 9. Open questions

- [ ] **N-API or subprocess for the Rust crates?** N-API faster; subprocess decoupled. Default N-API; fallback subprocess if version-coupling pain emerges.
- [ ] **Does the merge sequencer call `git rebase` directly, or shell out?** Direct via `git2` crate is robust but adds a dep; `Command::new("git")` is simpler. Default to `git2` for v1 since we already have lots of Rust.
- [ ] **`/project-tracks status` polling vs Pi events?** GUI subscribes to Pi events; CLI users get polling. Code is the same.
