# ROADMAP — workflow-app

> **Status:** Phase 7 of /project-init (final)
> **Last updated:** 2026-05-08
> **Depends on:** PROJECT_BRIEF.md, RESEARCH.md, MODULES.md, MASTER_BLUEPRINT.md, all module SPECs

## Overview

Three milestone bands: **MVP (TUI-only)** → **Beta (GUI)** → **v1.0 (production)**. Each band is multi-sprint; sprints are 1–2 weeks. Total elapsed estimate: **~14–18 weeks** for one developer using the bash kit's dual-harness model to build itself.

The roadmap deliberately ships value on the way: the MVP is usable as a pi-package replacement for the bash kit; the Beta adds GUI surfaces; v1.0 hardens for public release.

---

## Phase 0 — Infrastructure (Sprint 0, 1 wk)

**Exit criteria:** monorepo scaffolded, CI green, dev workflow documented.

- [ ] Monorepo: npm workspaces + Cargo workspace at root.
- [ ] Top-level `package.json` workspaces: `packages/*`, `apps/*`. Top-level `Cargo.toml` workspace: `crates/*`, `apps/gui-shell/src-tauri`.
- [ ] CI (GitHub Actions): lint, type check, `cargo test`, `vitest`, build matrix (Linux x86_64, macOS aarch64+x86_64).
- [ ] `bash bootstrap.sh` from `korallis/workflow` installed locally → kit drives the rest of the build (dogfood from day one).
- [ ] First `/project-init` complete → THIS document is in `specs/ROADMAP.md`.
- [ ] `LEARNINGS.md` initialised.
- [ ] Developer setup doc: `CONTRIBUTING.md` covering Rust toolchain, Node 22, Tauri prereqs.

---

## Milestone 1 — MVP (TUI-only pi-package), 4–6 wks

The kit, faithfully ported to Pi. No GUI. Users install via `npm install -g @korallis/workflow-skills` and use Pi's interactive TUI.

### Sprint 1.1 — Foundations (1–2 wks)
**Modules:** session-store, code-maps.

- [ ] **session-store crate** (Rust): SQLite + WAL + FTS5; migrations v1; `SessionStore` API; in-memory + file-backed.
- [ ] **code-maps crate** (Rust): tree-sitter for Rust + TypeScript + JavaScript + Python; `generate()` + `render_markdown()` + `budget()`.
- [ ] N-API wrappers for both → `kit-engine-native`.
- [ ] Each crate has its own `cargo test` suite + a smoke test for the N-API binding.

**Exit:** both crates pass `cargo test` and round-trip a fixture from TS via N-API.

### Sprint 1.2 — workflow-skills (1 wk)
**Modules:** workflow-skills (markdown port).

- [ ] Port all 12 SKILL.md files from `korallis/workflow/.claude/skills/*` into `packages/workflow-skills/skills/<name>/SKILL.md`.
- [ ] Port templates (`blueprint-template.md`, `module-spec-template.md`, etc.).
- [ ] Port schemas (`codex-report-schema.json`, `parallel.yaml.schema.json`).
- [ ] Write `package.json` with pi-package metadata.
- [ ] `npm pack` produces a tarball that `pi install` accepts.

**Exit:** `pi install ./workflow-skills-0.1.0.tgz` registers all 12 commands in Pi.

### Sprint 1.3 — Bridges (1–2 wks)
**Modules:** claude-bridge, codex-bridge.

- [ ] **claude-bridge:** `kit_claude_execute` Pi tool. Subprocess + JSONL parser + read-path scrubbing. Vitest fixtures.
- [ ] **codex-bridge:** `kit_codex_execute` Pi tool. `codex-codes` Rust transport via N-API + approval callback hook + schema enforcement. Test against a real Codex auth.
- [ ] Shared `@korallis/kit-scrub` package with the kit's regex set.

**Exit:** A real `/project-execute` against a fixture spec produces a schema-valid report and `proposed_commits`.

### Sprint 1.4 — kit-engine (1–2 wks)
**Module:** kit-engine.

- [ ] Pi extension `kit-engine`. Spec engine tools (`kit_spec_load`, `kit_spec_save`, `kit_spec_validate_parallel_yaml`).
- [ ] Track engine: `kit_track_plan`, `kit_track_start`, `kit_track_status`. Mirror bash kit's logic; carry forward CodeRabbit-vetted invariants.
- [ ] Learnings: `kit_learning_append`, `kit_learnings_search`.
- [ ] Code Maps: `kit_code_map`, `kit_code_map_directory`.
- [ ] Regression suite (port from bash): clean fixture, dep edge, slug boundary, dedupe, brownfield-without-yaml, claude-fails-fast.

**Exit:** `/project-init` end-to-end works: 7-phase setup writes all spec files. `/project-tracks plan auth billing` produces a valid plan.

### Sprint 1.5 — Dogfood + first end-to-end (1 wk)
- [ ] **Bootstrap dogfood:** install MVP into a fresh test repo, run `/project-init`, verify all phases produce expected output.
- [ ] **Self-host:** install MVP into THIS project, use it to spec the next milestone (Beta GUI).
- [ ] First `/project-execute` produces real code commits via the bash kit's orchestrator-commits pattern.
- [ ] LEARNINGS.md captures port deltas (bash → TS gotchas).

**Exit (Milestone 1):** A solo dev with Node 22 + Pi can install the MVP via `npm install -g @korallis/workflow-skills` and run the kit's full workflow end-to-end. Cross-platform (Linux + macOS).

---

## Milestone 2 — Beta GUI (Tauri shell), 5–7 wks

A Tauri-shell-driven UX with Repo Prompt-style visual context engineering and a parallel-tracks dashboard.

### Sprint 2.1 — Tauri scaffold + Pi RPC (1 wk)
- [ ] `apps/gui-shell` initialised via `create-tauri-app v2` (React + TypeScript + Vite + Tailwind v4).
- [ ] `src-tauri/src/pi.rs`: spawn Pi subprocess, RPC channel, lifecycle.
- [ ] `pi_rpc` + `pi_event_subscribe` Tauri commands.
- [ ] Frontend connects to Pi, displays a "connected" indicator.
- [ ] Catppuccin Mocha theme via `@theme` tokens.

**Exit:** App launches; Pi subprocess connected; basic command round-trip works.

### Sprint 2.2 — Onboarding + Auth detection (1 wk)
- [ ] `auth_status` Tauri command: detect `claude`, `codex`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, Node 22, Pi.
- [ ] Onboarding screen with one-click install guides for each missing piece.
- [ ] Skip onboarding if all green; show last-opened project instead.

**Exit:** First-run UX < 5 minutes from open to "ready to /project-init" on a clean machine.

### Sprint 2.3 — Plan board (1–2 wks)
- [ ] `tracks_list` Tauri command (direct SQLite read).
- [ ] Plan board screen: list of tracks with live status; expand for per-track log + report.
- [ ] Pi events route to Plan board (status changes, file_change, agent_message).
- [ ] Sentinel-watcher integration: pid-died tracks show `aborted`; manual "retry" button re-runs the dispatcher.

**Exit:** A `/project-tracks start auth billing` from the GUI shows two live panels with real-time updates.

### Sprint 2.4 — File picker + Code Maps (1–2 wks)
- [ ] File tree with search, multi-select, token-cost annotations per file.
- [ ] Code Maps preview pane: tree-list of items; click-to-jump.
- [ ] `code_map_for_directory` Tauri command + frontend rendering.
- [ ] Token budget UI: visualise % of model context used.

**Exit:** User can select 20 files, see a combined Code Map at <85K tokens, copy as XML to clipboard or pipe to Pi as prompt context.

### Sprint 2.5 — Apply Mode (1 wk)
- [ ] `apply_proposed_commits` Tauri command: read run's report, present as commits.
- [ ] CodeMirror 6 diff viewer with per-hunk accept/reject.
- [ ] Commit button stages + commits with `Co-Authored-By` attribution per the kit's pattern.
- [ ] Edge case: file-only deletion proposed — show as a single-block diff.

**Exit:** A `/project-execute` run completes and the user reviews + commits via Apply Mode without touching `git` directly.

### Sprint 2.6 — context-mode-bridge + Learnings browser (1 wk)
- [ ] **context-mode-bridge** extension: spawn + supervise context-mode; PreToolUse hook; telemetry.
- [ ] Status bar shows "saved this session: X KB / Y%".
- [ ] **Learnings browser** screen: FTS5 search; filter by module/tag; jump to commit.
- [ ] Cross-project queries via the global SQLite DB.

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
- [ ] User documentation site (probably `mdbook` or `Docusaurus`): install, first project, all 12 commands, troubleshooting.
- [ ] Public README with screencap.
- [ ] HN/Reddit/Twitter launch posts.
- [ ] First-month support window: rapid-response to issues.

**Exit (v1.0):** Public release. Linux + macOS + Windows. Documented. Self-hostable. Open source MIT.

---

## Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Codex `codex-codes` API breaks | Medium | High | Abstract behind `CodexTransport` trait; pin version; CI matrix tests against the pinned + latest. |
| Pi extension API changes upstream | Medium | High | Pin Pi version; track upstream releases via dependabot; PR upstream when our needs evolve. |
| Claude `--output-format stream-json` schema changes silently | Medium | Medium | Schema fuzz tests with captured fixtures; detect Claude version on startup; warn on mismatch. |
| Tauri 2 build complexity (signing, notarisation) | Low | Medium | ELVES has solved this; reference their `packaging/`. |
| context-mode upstream lag | Low | Low | We use it as a sidecar; we're insulated from API changes; can pin a version. |
| Solo dev burnout | Medium | High | The kit's compound learning + parallel tracks is the antidote; this product enables itself. |
| Apple Developer cert cost | Certain | Low | $99/yr; budget item. |
| Cross-platform PTY / portable-pty Windows quirks | High | Medium | Defer to v1.1; have a Linux/macOS-first beta. |
| Schema mismatch between Codex `--output-schema` and our `codex-report-schema.json` | Low | Medium | Validate twice (Codex + orchestrator); cross-check fields against `git status`. |

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
- Research (incl. Pi, Tauri, rmcp, codex-codes audits): `specs/RESEARCH.md`
- Module list and dependencies: `specs/MODULES.md`
- Architecture: `specs/MASTER_BLUEPRINT.md`
- Per-module specs: `specs/modules/<name>/SPEC.md`
- Per-module conventions: `specs/modules/<name>/CLAUDE.md`
- Per-module parallel hints: `specs/modules/<name>/parallel.yaml`
