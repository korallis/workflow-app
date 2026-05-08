# ROADMAP — workflow-app

> **Status:** Phase 7 of /project-init (final)
> **Last updated:** 2026-05-08
> **Depends on:** PROJECT_BRIEF.md, RESEARCH.md, MODULES.md, MASTER_BLUEPRINT.md, all module SPECs

> **Architectural pivot 2026-05-08:** This roadmap was rewritten to reflect the pure-Rust architecture (no Pi, no Node, no npm). See `SPEC_REVISION_2026-05-08.md` for rationale.

## Overview

Three milestone bands: **MVP (CLI-only)** → **Beta (GUI)** → **v1.0 (production)**. Each band is multi-sprint; sprints are 1–2 weeks. Total elapsed estimate: **~13–17 weeks** for one developer using the bash kit's dual-harness model to build itself, then transitioning to dogfood once the MVP CLI works.

The roadmap deliberately ships value on the way: the MVP is a single-install Rust binary that replaces the bash kit; the Beta adds GUI surfaces; v1.0 hardens for public release.

---

## Phase 0 — Infrastructure (Sprint 0, 1 wk) — **in progress**

**Exit criteria:** Cargo workspace scaffolded, CI green, dev workflow documented.

- [x] First `/project-init` complete → spec hierarchy in `specs/`.
- [x] Cargo workspace at root (`Cargo.toml`, `crates/*`).
- [x] `LEARNINGS.md` initialised; capture session-store + dispatch-infra discoveries.
- [x] `.gitignore` covers Cargo `target/` and dual-harness `.kit-orchestration/` artefacts.
- [ ] CI (GitHub Actions): `cargo fmt`, `cargo clippy`, `cargo test --workspace`; build matrix (Linux x86_64, macOS aarch64+x86_64).
- [ ] Developer setup doc: `CONTRIBUTING.md` covering Rust toolchain, optional Tauri prereqs.
- [ ] `bash bootstrap.sh` from `korallis/workflow` checked in as the bootstrap record (we drove the build from it).

---

## Milestone 1 — MVP (CLI-only `kit` binary), 4–6 wks

The kit, ported to a single Rust binary. No GUI. Users install via `cargo install kit` (or a homebrew tap / curl-bash install script). The binary reads the user's `claude login` and walks them through `codex` OAuth on first run.

### Sprint 1.1 — Foundations (1–2 wks) — **partially done**
**Modules:** session-store, code-maps.

- [x] **session-store crate** (Rust): SQLite + WAL + FTS5; migrations v1; `SessionStore` API; in-memory + file-backed. 7 tests passing.
- [ ] **code-maps crate** (Rust): tree-sitter for Rust + TypeScript + JavaScript + Python; `generate()` + `render_markdown()` + `budget()`.
- [ ] Each crate has its own `cargo test` suite with fixtures.

**Exit:** both crates pass `cargo test` against representative fixtures.

### Sprint 1.2 — Spec layer (1–2 wks)
**Modules:** spec-engine, track-engine.

- [x] **spec-engine crate**: load/save the full spec hierarchy. Snapshot persistence to `session-store`. 11 tests passing. **Follow-up:** `parallel.yaml` validation + parsing are currently hand-rolled (rules match the bundled JSON Schema); swap to `jsonschema` + `serde_yaml` crates once cache is warm — see module CLAUDE.md "Implementation status" and LEARNINGS.md "Codex sandbox + Cargo".
- [ ] **track-engine crate**: parallel-tracks state machine. Worktree create/cleanup. Merge sequencer (dependency-ordered rebase). Sentinel-watcher for operator-killed panes. Learnings-fragment merge.
- [ ] Regression suite (port from bash): clean fixture, dep edge, slug boundary, dedupe, brownfield-without-yaml, claude-fails-fast.

**Exit:** `track-engine` can plan + start two parallel tracks against a fixture spec, with state persisted via `session-store`.

### Sprint 1.3 — Bridges (1–2 wks)
**Modules:** claude-bridge, codex-bridge.

- [ ] **claude-bridge:** Tokio subprocess wrapper for `claude --print --bare --append-system-prompt-file <ours> --output-format stream-json`. JSONL parser + read-path scrubbing. Fixture-based tests.
- [ ] **codex-bridge:** PKCE OAuth flow against `auth.openai.com`. Token storage in `~/.kit-workflow-app/auth.json`. Direct calls to `chatgpt.com/backend-api/codex/responses` with `store: false, stream: true`. Approval callback dispatch. Schema enforcement against `codex-report-schema.json`.
- [ ] Shared `kit-scrub` crate with the kit's regex set for secret scrubbing.

**Exit:** A real `/project-execute` against a fixture spec produces a schema-valid report and `proposed_commits` via `codex-bridge`, and a real `/project-module` runs end-to-end via `claude-bridge`.

### Sprint 1.4 — Skill runner + CLI (1–2 wks)
**Module:** skill-runner. Plus a thin CLI binary `apps/kit` that exposes the 12 commands.

- [ ] **skill-runner crate**: bundle the 12 SKILL.md files as embedded markdown assets. Skill interpreter: parse SKILL.md, assemble prompt (using `code-maps` for context), route to the right bridge, process the result. Apply orchestrator-commits via `git2`.
- [ ] `apps/kit/src/main.rs`: clap-based CLI with subcommands matching the 12 slash commands. Reads project state via `spec-engine` + `track-engine`.
- [ ] Wire `/project-init`, `/project-spec`, `/project-module`, `/project-execute`, `/project-tracks`, `/project-status`, `/project-review`, `/project-blueprint`, `/project-research`, `/project-deploy`, `/project-test`, `/project-security-review`.
- [ ] Crash-recovery: on startup, replay SQLite WAL; resume any interrupted skill from the most recent `spec_snapshots` row.

**Exit:** `kit /project-init` end-to-end works on a fresh repo. `kit /project-tracks plan auth billing` produces a valid plan. `kit /project-execute <module>` walks Codex through a real implementation.

### Sprint 1.5 — Dogfood + first end-to-end (1 wk)
- [ ] **Bootstrap dogfood:** install MVP into a fresh test repo, run `kit /project-init`, verify all phases produce expected output.
- [ ] **Self-host:** install MVP into THIS project, use it to spec the next milestone (Beta GUI). Hand-over from bash-kit dogfood.
- [ ] First `kit /project-execute` produces real code commits via the orchestrator-commits pattern.
- [ ] LEARNINGS.md captures port deltas (bash → Rust gotchas, OAuth flow gotchas, etc.).

**Exit (Milestone 1):** A solo dev with `cargo` (or a downloaded binary) runs `kit /project-init` end-to-end. Cross-platform (Linux + macOS). No Node, no npm, no Pi.

---

## Milestone 2 — Beta GUI (Tauri shell), 5–7 wks

A Tauri-shell-driven UX with Repo-Prompt-style visual context engineering and a parallel-tracks dashboard. Core crates link directly into `src-tauri` — no subprocess, no Pi RPC.

### Sprint 2.1 — Tauri scaffold + core wiring (1 wk)
- [ ] `apps/gui-shell` initialised via `create-tauri-app v2` (React 19 + TypeScript 5.8 strict + Vite 7 + Tailwind v4).
- [ ] `src-tauri/Cargo.toml` declares the 8 core crates as dependencies.
- [ ] Tauri commands wrapping `skill-runner`, `spec-engine`, `track-engine`, `session-store` queries.
- [ ] Tauri event channels for live skill progress (file_change, agent_message, command_execution).
- [ ] Catppuccin Mocha theme via `@theme` tokens (Tailwind v4 CSS-first config).

**Exit:** App launches; first Tauri command round-trip works; live event stream from a fake skill.

### Sprint 2.2 — Onboarding + Auth detection (1 wk)
- [ ] `auth_status` Tauri command: detect `claude` CLI + login state, codex OAuth tokens in `~/.kit-workflow-app/auth.json`, optional `ANTHROPIC_API_KEY` / `OPENAI_API_KEY`.
- [ ] Onboarding screen: detect missing pieces, walk the user through `claude login` and the codex-bridge OAuth flow (browser opens to `auth.openai.com`).
- [ ] Skip onboarding if all green; show last-opened project instead.

**Exit:** First-run UX < 5 minutes from open to "ready to run skills" on a clean machine.

### Sprint 2.3 — Plan board (1–2 wks)
- [ ] `tracks_list` Tauri command (direct call into `track-engine` + `session-store`).
- [ ] Plan board screen: list of tracks with live status; expand for per-track log + report.
- [ ] Tauri events stream from `track-engine` to the Plan board (status changes, file_change, agent_message).
- [ ] Sentinel-watcher integration: pid-died tracks show `aborted`; manual "retry" button re-runs.

**Exit:** A `/project-tracks start auth billing` from the GUI shows two live panels with real-time updates.

### Sprint 2.4 — File picker + Code Maps (1–2 wks)
- [ ] File tree with search, multi-select, token-cost annotations per file.
- [ ] Code Maps preview pane: tree-list of items; click-to-jump.
- [ ] `code_map_for_directory` Tauri command (calls `code-maps` directly).
- [ ] Token budget UI: visualise % of model context used.

**Exit:** User can select 20 files, see a combined Code Map at <85K tokens, copy as XML to clipboard or hand to a skill as prompt context.

### Sprint 2.5 — Apply Mode (1 wk)
- [ ] `apply_proposed_commits` Tauri command: read run's report from `session-store`, present as commits.
- [ ] CodeMirror 6 diff viewer with per-hunk accept/reject.
- [ ] Commit button stages + commits with `Co-Authored-By` attribution per the kit's pattern (via `git2`).
- [ ] Edge case: file-only deletion proposed — show as a single-block diff.

**Exit:** A `/project-execute` run completes and the user reviews + commits via Apply Mode without touching `git` directly.

### Sprint 2.6 — context-mode-manager + Learnings browser (1 wk)
- [ ] **context-mode-manager** crate: spawn + supervise context-mode via `rmcp`; route prompts through `ctx_execute` / `ctx_search`; telemetry.
- [ ] Status bar shows "saved this session: X KB / Y%".
- [ ] **Learnings browser** screen: FTS5 search via `session-store`; filter by module/tag; jump to commit.
- [ ] Cross-project queries via the global SQLite DB at `~/.kit-workflow-app/global.db`.

**Exit (Milestone 2):** Beta release on GitHub Releases. macOS aarch64 + Linux x86_64 AppImage. Bundle size ≤30 MB / 40 MB.

---

## Milestone 3 — v1.0 production, 5 wks

Hardening, distribution, and Windows.

### Sprint 3.1 — Stage 2/3 of /project-tracks (2 wks)
- [ ] `/project-tracks status` (live), `review`, `merge` (dependency-ordered rebase + apply), `cleanup`.
- [ ] Sentinel-watcher robustness: detect operator-killed panes; `aborted` state with branch+worktree preservation.
- [ ] Compound learnings merge: per-track fragments → root LEARNINGS.md (FTS5-indexed).
- [ ] Auto-PR on merge (opt-in, off by default).

### Sprint 3.2 — Windows (1–2 wks)
- [ ] Tauri Windows target. MSI installer.
- [ ] `portable-pty` Windows quirks fixed.
- [ ] PowerShell + bash both supported for shell tooling.
- [ ] Windows-specific path handling audited.

### Sprint 3.3 — Hardening + Telemetry (1 wk)
- [ ] **Crash recovery.** SQLite WAL replay on app start; orphaned worktrees detected + offered for cleanup.
- [ ] **Auto-update channel.** Tauri auto-updater + signed releases.
- [ ] **Local-only telemetry** (no phone-home): `ctx_stats`, dispatch latency, error counts. Settings toggle to display in a panel.
- [ ] **Accessibility audit.** WCAG AA contrast, keyboard navigation, screen reader labels.

### Sprint 3.4 — Documentation + launch (1 wk)
- [ ] User documentation site (`mdbook`): install, first project, all 12 commands, troubleshooting.
- [ ] Public README with screencap.
- [ ] HN/Reddit/Twitter launch posts.
- [ ] First-month support window: rapid-response to issues.

**Exit (v1.0):** Public release. Linux + macOS + Windows. Documented. Self-hostable. Open source MIT.

---

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Codex Responses API breaks | Medium | High | Schema fuzz tests with captured fixtures; pin auth-flow client_id; CI matrix tests against the pinned + latest. |
| Anthropic changes Claude Code's `--print --bare` flag semantics | Medium | High | Pin `claude` minor version in install docs; detect version on startup; warn on mismatch. Schema fuzz tests against captured stream-json fixtures. |
| OAuth flow changes (Codex or future Claude OAuth) | Low | High | Abstract behind a `Transport` trait; pin client_id; track upstream releases. |
| Tauri 2 build complexity (signing, notarisation) | Low | Medium | ELVES has solved this; reference their `packaging/`. |
| context-mode upstream lag | Low | Low | We use it as a sidecar; we're insulated from API changes; can pin a version. |
| Solo dev burnout | Medium | High | The kit's compound learning + parallel tracks is the antidote; this product enables itself. |
| Apple Developer cert cost | Certain | Low | $99/yr; budget item. |
| Cross-platform PTY / portable-pty Windows quirks | High | Medium | Defer to v1.1; have a Linux/macOS-first beta. |

## Success metrics (v1.0)

- **Install success rate** ≥ 95% on a clean machine within 5 minutes.
- **Time from `/project-init` to first commit** ≤ 30 minutes for a small module on a familiar stack.
- **Context efficiency** ≥ 90% reduction (vs raw tool outputs) when context-mode is enabled.
- **Plan board latency** ≤ 1s from track event to UI update.
- **GitHub stars** ≥ 100 in first month (loose signal but useful).
- **Issues resolution median** ≤ 7 days for first month.
- **Dogfood:** the workflow-app team uses workflow-app to ship workflow-app updates.

## Cross-references

- Vision and constraints: `specs/PROJECT_BRIEF.md`
- Research (incl. Pi, Tauri, rmcp, OAuth audits): `specs/RESEARCH.md`
- Architecture pivot rationale: `specs/SPEC_REVISION_2026-05-08.md`
- Module list and dependencies: `specs/MODULES.md`
- Architecture: `specs/MASTER_BLUEPRINT.md`
- Per-module specs: `specs/modules/<name>/SPEC.md`
- Per-module conventions: `specs/modules/<name>/CLAUDE.md`
- Per-module parallel hints: `specs/modules/<name>/parallel.yaml`
