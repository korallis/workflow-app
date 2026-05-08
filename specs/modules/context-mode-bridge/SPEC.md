# context-mode-bridge — SPEC

> **Status:** Phase 5 of /project-init
> **Layer:** Engine (TypeScript Pi extension)
> **Last updated:** 2026-05-08

## 1. Purpose

Embed [mksglu/context-mode](https://github.com/mksglu/context-mode) as a **managed sidecar MCP server** so all tool outputs route through it for sandboxing and 98% context-window reduction. Spawn at Pi startup, supervise health, expose its 8 `ctx_*` tools to the Pi agent, register a PreToolUse hook, surface `ctx_stats` telemetry to the GUI.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| CMB-1 | Pi extension running a `bash` tool | Have its output route through `ctx_execute` automatically | A 56KB Playwright snapshot becomes a 299B summary |
| CMB-2 | `/project-research` skill | Use `ctx_fetch_and_index` to load Exa results into FTS5 | Future calls retrieve via BM25, not re-fetch |
| CMB-3 | GUI status bar | See live "context saved this session" telemetry | The user understands the value being delivered |
| CMB-4 | Power user | Disable context-mode for a session via env var | I can debug raw outputs when something looks wrong |

## 3. Lifecycle

```text
Pi startup
   └─ context-mode-bridge.activate(ctx)
         ├─ check `KIT_CONTEXT_MODE_DISABLE` (skip if set)
         ├─ resolve context-mode binary (bundled or user-installed)
         ├─ spawn as MCP server subprocess
         ├─ MCP handshake (stdio JSON-RPC)
         ├─ register all 8 ctx_* tools with Pi
         ├─ register PreToolUse hook (route bash/web/file outputs through ctx_execute)
         ├─ start health check (poll every 30s)
         └─ expose ctx_stats query to the GUI

Pi shutdown
   └─ context-mode-bridge.deactivate(ctx)
         ├─ flush pending writes
         ├─ SIGTERM subprocess (10s grace)
         └─ SIGKILL on timeout
```

## 4. Public Pi tool surface

The 8 ctx_* tools are registered with Pi by this extension. They route to the context-mode subprocess via MCP:

- `ctx_batch_execute` — run multiple commands + searches in one call (986 KB → 62 KB)
- `ctx_execute` — run code in 11 languages; only stdout enters context (56 KB → 299 B)
- `ctx_execute_file` — process files in sandbox; raw content never leaves (45 KB → 155 B)
- `ctx_index` — chunk markdown into FTS5 with BM25 (60 KB → 40 B)
- `ctx_search` — query indexed content
- `ctx_fetch_and_index` — fetch URL + chunk + index; 24h TTL cache
- `ctx_stats` — savings + counts (telemetry surface)
- `ctx_purge` — wipe indexed content

Plus a Pi-bridge-internal tool: `kit_context_mode_status()` returning `{ healthy: boolean; pid: number; uptime_seconds: number; subprocess_log_path: string }` for the GUI.

## 5. Business rules

- **Optional, not required.** `KIT_CONTEXT_MODE_DISABLE=1` skips all of this. Pi works without context-mode.
- **PreToolUse hook routes selectively.** Bash, WebFetch, WebSearch, file-read tools route through `ctx_execute` / `ctx_index`. Lightweight tools (Glob, Grep) bypass.
- **Health check.** Poll context-mode every 30s; restart on dead subprocess (max 3 restarts/session before giving up + warning).
- **Path:** `~/.kit-workflow-app/context-mode/` for SQLite content store (matches their default).
- **Telemetry surfacing.** `ctx_stats` cached for 5s (avoid hammering); GUI polls.
- **No automatic upgrades.** User upgrades context-mode explicitly via `/plugin upgrade` or our packaging.
- **Subprocess crash handling.** Log + warn, attempt restart, fall back to bypass mode (tools route directly to Pi without context-mode) if 3 restarts fail.

## 6. Integration points

| Module | Relationship | Notes |
|---|---|---|
| Pi | Extends | Standard Pi extension lifecycle |
| context-mode (external) | Spawns + supervises | Bundled or user-installed |
| kit-engine | Adjacent | Doesn't call into context-mode-bridge directly; benefits via the PreToolUse hook |
| codex-bridge / claude-bridge | Adjacent | Same — outputs routed through transparently |
| gui-shell | Reads telemetry | `kit_context_mode_status` for status indicator |

## 7. Acceptance criteria

- [ ] Extension loads context-mode subprocess on Pi startup; MCP handshake completes; 8 tools registered.
- [ ] PreToolUse hook routes a bash command's output through `ctx_execute`; result is the summary, not the raw bytes.
- [ ] `KIT_CONTEXT_MODE_DISABLE=1` cleanly skips the entire flow with a single info log.
- [ ] Subprocess crash → 1 restart → continue. After 3 restarts → warn + bypass mode.
- [ ] `ctx_stats` query returns valid telemetry; GUI shows "X KB saved this session".
- [ ] `Ctrl+C` on Pi cleanly shuts down context-mode subprocess (SIGTERM + 10s grace).
- [ ] First-run user with no context-mode installed gets a clear install guide (or auto-install if bundled).

## 8. Out of scope

- Modifying context-mode itself (we use upstream as-is).
- Custom routing rules per project (use their config; document `~/.context-mode/config.json`).
- Multi-process context-mode (one per project, not one per track — context-mode itself is single-instance).
- Replacement of context-mode with our own implementation. Use mksglu's; cite + thank.

## 9. Open questions

- [ ] **Bundle or fetch?** Bundle pinned version for reproducibility; surface "update" in Settings.
- [ ] **Per-project or global instance?** Global instance simpler; per-project gives stronger isolation. v1: global with project-tagged events.
- [ ] **Hook conflicts.** If user has their own PreToolUse hooks via Pi extensions, do we chain or override? Probably chain.
