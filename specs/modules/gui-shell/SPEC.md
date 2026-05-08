# gui-shell — SPEC

> **Layer:** Shell (Tauri 2 + React 19)
> **Last updated:** 2026-05-08 (Pi-removal pivot — no Pi RPC; direct Tauri commands into Rust core)
> **Depends on:** all core Rust crates (session-store, code-maps, spec-engine, track-engine, claude-bridge, codex-bridge, context-mode-manager, skill-runner)

## 1. Purpose

The user-visible **desktop application**. A thin Tauri 2 + React shell wrapping Pi (which runs the engine). Owns four primary surfaces:

1. **Onboarding / auth detection** — first-run UX; detects `claude` / `codex` / API keys; guides through gaps.
2. **Plan board** — parallel-tracks dashboard; live state; click into a track for its scrubbed report + log.
3. **Visual file picker + Code Maps** — Repo-Prompt-style: select files visually, see compressed structural summaries + token estimates.
4. **Apply Mode diff viewer** — review `proposed_commits` from agent reports; accept/reject per-hunk; commit on user click.

Plus secondary surfaces: Projects landing, Learnings browser (FTS5-backed), Settings.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| GUI-1 | First-run user | Open the app and see what's missing for setup (e.g. "Codex CLI not installed") with one-click install guides | I'm running my first /project-init in <5 min |
| GUI-2 | User mid-flow | See live status of every parallel track in one pane | I don't switch between terminals |
| GUI-3 | User assembling a prompt | Select files visually, see Code Maps, watch token count | I send signal, not bytes |
| GUI-4 | User reviewing an agent run | See proposed commits as reviewable diffs; accept/reject per-hunk | The orchestrator-commits pattern is one click |
| GUI-5 | User looking up past learnings | Search "tauri-2" across all my projects | Compound learning is queryable |
| GUI-6 | User who prefers terminals | Run the engine without the GUI | TUI-only mode via `pi` directly |

## 3. Screen inventory

| Screen | Path | Notes |
|---|---|---|
| Projects landing | `/` | List of recent projects + "New project" + "Open existing" |
| Onboarding / auth | `/onboarding` | First-run only; detect+guide; skip if all green |
| Plan board | `/project/:id/plan` | Live tracks; per-track expansion; sentinel/abort controls |
| File picker + Code Maps | `/project/:id/context` | File tree with token estimates; selection state; Code Maps preview |
| Apply Mode | `/project/:id/apply/:run_id` | Diff viewer (CodeMirror 6); accept/reject per-hunk; commit button |
| Learnings | `/project/:id/learnings` | FTS5 search; filter by module/tag |
| Settings | `/settings` | Pi version, context-mode toggle, auth detect re-run, theme |
| Spec viewer | `/project/:id/specs/:path` | Read-only viewer for spec hierarchy |

## 4. Tauri commands (Rust ↔ frontend)

No Pi RPC. Each Tauri command is a thin wrapper around a Rust core crate API:

```rust
// Skill execution
#[tauri::command]
async fn run_skill(skill: String, args: Value, app: AppHandle, state: State<'_, AppState>) -> Result<SkillResult, String>;

#[tauri::command]
async fn list_skills(state: State<'_, AppState>) -> Result<Vec<SkillMeta>, String>;

#[tauri::command]
async fn auth_status() -> AuthStatus;

#[tauri::command]
async fn code_map_for_file(path: PathBuf) -> Result<CodeMap, String>;

#[tauri::command]
async fn code_map_for_directory(root: PathBuf, opts: ScanOpts) -> Result<Vec<CodeMap>, String>;

#[tauri::command]
async fn tracks_list(project: String, filter: Option<TrackFilter>) -> Result<Vec<Track>, String>;

#[tauri::command]
async fn learnings_search(q: String, opts: SearchOpts) -> Result<Vec<LearningHit>, String>;

#[tauri::command]
async fn open_project(path: PathBuf) -> Result<Project, String>;

#[tauri::command]
async fn project_list() -> Result<Vec<Project>, String>;

#[tauri::command]
async fn apply_proposed_commits(run_id: String, accepted: Vec<CommitDecision>) -> Result<ApplyResult, String>;
```

Pi events flow as Tauri events on the channel `pi://event` with payload typed per the bridges' `ClaudeEvent` / `CodexEvent` shapes.

## 5. Frontend architecture

- **Routing:** React Router v7.
- **State:** Zustand. One slice per domain: `useAppStore` (Pi connection + auth), `useProjectStore` (current project + specs), `useTracksStore` (live tracks), `useUiStore` (theme, modals).
- **Data fetching:** TanStack Query for Tauri commands; Pi events feed into Zustand.
- **Styling:** Tailwind v4 with `@theme` tokens. Catppuccin Mocha as the v1 theme.
- **Components:** shadcn/ui-style primitives (Radix + Tailwind); custom for plan board + diff viewer.
- **Diff viewer:** CodeMirror 6 with `@codemirror/merge`.
- **Code Maps viewer:** custom; tree-list of items with click-to-jump-to-line.

## 6. Process model

- Single Tauri process. **No Pi, no Node.** Rust core lives in-process as crates.
- Skill runs spawn per-call: claude-bridge / codex-bridge / context-mode-manager each manage their own subprocesses with proper cleanup on app shutdown.
- Tauri shared state (`State<AppState>`): `Arc<SkillRunner>`, `Arc<TrackEngine>`, `Arc<SpecEngine>`, etc. Frontend talks to these via Tauri commands; events flow back via Tauri events.
- Shutdown: `WindowEvent::CloseRequested` → app calls `shutdown_all()` which gracefully stops context-mode subprocess + reaps any in-flight claude/codex children + flushes SQLite WAL.

## 7. Acceptance criteria

- [ ] App opens, detects Pi, spawns it, displays Plan board for the last-opened project in <2s on a warm machine.
- [ ] Auth detection on a clean machine flags missing CLIs with one-click install guides.
- [ ] Plan board updates live (within 1s) when a track's status changes.
- [ ] File picker on a 1000-file repo loads in <2s; Code Maps render lazily on selection.
- [ ] Apply Mode renders a 50-file diff in <1s; per-hunk accept/reject works.
- [ ] Learnings search returns top-10 hits in <100ms (UI overhead).
- [ ] App quits cleanly: Pi subprocess + context-mode subprocess both reaped.
- [ ] All 4 primary surfaces (onboarding, plan, context, apply) keyboard-navigable.
- [ ] WCAG AA contrast for the Catppuccin Mocha theme.
- [ ] Tauri bundle: <30 MB on macOS aarch64; <40 MB on Linux x86_64 AppImage.

## 8. Out of scope

- Mobile (iOS/Android), even though Tauri 2 supports them.
- Browser-only mode (no Tauri shell). Possible v2.
- Plugins / theming SDK (use built-in themes only in v1).
- Voice input, agent personalities, animated avatars (the ELVES anti-pattern).
- Multi-window UI (single window v1; revisit if users push).
- Chat UI for direct Pi chatter (the slash commands are the surface; raw chat is Pi's TUI).

## 9. Open questions

- [ ] **Single window or multi-window?** Multi-window helps power users (track + diff side-by-side). v1: single + tabs; v1.1 multi-window.
- [ ] **Pi's interactive TUI inside the app** as a debug pane? Useful but adds an embedded PTY surface.
- [ ] **Auto-update channel** (Tauri auto-updater)? Yes, opt-in; signed updates; release branch separate from main.
- [ ] **Code signing** for macOS (.app, .dmg)? Required for distribution outside dev mode. Apple Developer cert needed.
