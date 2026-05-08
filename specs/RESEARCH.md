# RESEARCH — workflow-app

> **Status:** Phase 2 of /project-init
> **Last updated:** 2026-05-08
> **Method:** Exa web_search_exa + Ref ref_search_documentation. All findings cited with URLs and dates. Research conducted prior to /project-init invocation; this document is the audit trail.

## 1. Market Landscape

The "AI-coding-agent-orchestration" category is **active and crowded**, but the kit's specific shape — *spec-first / dual-harness / compound-learning / parallel-tracks* — is **not occupied** by any current product. Existing apps either:
- ship multi-agent dashboards without a workflow model (beehive, TUICommander, racc)
- wrap a single agent with personality (ELVES)
- are pure context-engineering tools (Repo Prompt) without orchestration
- are kanban-coordination platforms (Routa)
- are minimal extensible harnesses (Pi)

There is room for a product that treats the kit's workflow as the *primary product* and uses these existing pieces as *infrastructure*.

## 2. Competitor / Reference Analysis

| Tool | Author | License | Stars | Stack | What it does | Why we don't fork it |
|---|---|---|---|---|---|---|
| **[Pi](https://pi.dev/)** ([earendil-works/pi](https://github.com/earendil-works/pi)) | Mario Zechner | MIT-style | — | TypeScript (npm) | Minimal terminal coding harness; 4 modes (interactive/print/RPC/SDK); extensible via TypeScript extensions, skills, prompts, themes; multi-provider via `pi-ai`; sessions in JSONL with branching. | **We embed it.** Pi is the engine, not a competitor. |
| **[pi_agent_rust](https://github.com/Dicklesworthstone/pi_agent_rust)** | Dicklesworthstone | "Other" | 859 | Rust | From-scratch Rust port of Pi (with Mario's blessing). Single binary, instant startup, custom HTTP/TLS/SSE on `asupersync`. | **Reference & possible bundling.** Single-binary distribution is attractive; we may bundle the Rust port for users who don't have Node. |
| **[ELVES](https://github.com/mvmcode/elves)** | Manikandan VM | MIT | 10 | Tauri 2 + React + Rust (rusqlite + portable-pty) | "Whimsical, viral, personality-driven multi-agent system" with named elf characters, animated avatars, sound design. "Not just for coding." | **Explicitly rejected as a fork base.** Their VISION.md is incompatible with disciplined-engineering UX. We cite their `DECISIONS.md` (Tauri 2 + rusqlite v0.31 + Vitest + `claude --print --output-format json`) for stack picks. |
| **[Repo Prompt](https://repoprompt.com/)** | Eric Provencher | Closed (Swift macOS) | — | Swift macOS native | Visual context engineering: file picker, Code Maps (compressed structural summaries via tree-sitter-style analysis), Apply Mode (XML-format model output → reviewable diffs). Also exposes itself as an MCP server for Cursor/Claude Code. | **UX reference, not infrastructure.** We replicate its file-picker / Code Maps / Apply Mode shape via tree-sitter + a custom diff parser. Closed source so we can't reuse code. |
| **[context-mode](https://github.com/mksglu/context-mode)** | mksglu | MIT | — | TypeScript (MCP server) | Sandboxes MCP tool output: 315 KB → 5.4 KB (98% reduction) via `ctx_execute`/`ctx_index`/`ctx_search` over SQLite FTS5 + BM25. Plugin-installable into Claude Code via `/plugin marketplace add`. | **We embed it as a managed sidecar.** Pi launches context-mode as an MCP server on app start; tool outputs route through it. |
| **[beehive](https://github.com/storozhenko98/beehive)** | storozhenko98 | MIT | 54 | Rust+Tauri+TypeScript / Ratatui+Crossterm TUI | Multi-repo, isolated worktrees ("combs"), persistent terminals, agent panes (Claude Code or any CLI agent). macOS GUI + standalone TUI. | **Infrastructure reference.** Their PTY+worktree+layout-persistence patterns are well-engineered; we'll reference the patterns, not the code. |
| **[TUICommander](https://github.com/sstraus/tuicommander)** | sstraus | Apache 2.0 | 44 | Rust+Tauri+SolidJS | Auto-detects 10 AI agents (Claude, Codex, Aider, Gemini, Amp, Cursor Agent, OpenCode, Warp Oz, Droid, Goose). Real-time observability, MCP proxy hub, plugin SDK, autonomous agent loop. | Closest existing tool but generic-agent-runner; we want kit-workflow-specific. |
| **[Routa](https://github.com/LyonVallence/routa)** | LyonVallence | Various | — | Tauri+Next.js+Axum | Workspace-first multi-agent platform. Kanban orchestration: Backlog→Todo→Dev→Review→Done with specialized agents per column. Supports Claude Code, Codex, Gemini, Kimi, Augment, Copilot, OpenCode via ACP. | Different shape — kanban-driven, not spec-first. |
| **[racc](https://github.com/liu1700/racc)** | liu1700 | MIT | 1 | Three-crate Rust workspace + Tauri + React | Desktop control plane for AI coding agents. Multi-session, task board, git worktree isolation. Headless server (axum) for browser access. | Early stage, single contributor. Reference. |

**Conclusion:** Greenfield + Pi as engine + cite/reference the others.

## 3. Technical Foundation Audit

### Pi (the engine)

- **Authoritative source:** [earendil-works/pi](https://github.com/earendil-works/pi) — TypeScript monorepo with `pi-coding-agent` (CLI), `pi-agent-core` (runtime), `pi-ai` (multi-provider LLM API), `pi-tui` (TUI library), `pi-web-ui`.
- **Distribution:** `npm install -g @earendil-works/pi-coding-agent` or `curl -fsSL https://pi.dev/install.sh | sh`.
- **Models:** Configurable via `pi-ai`. Auto-fallback chain (Anthropic Claude → OpenAI Codex → Google Gemini).
- **Execution modes:** `interactive` (TUI), `print` (single-shot), `json` (JSONL events), `rpc` (LF-delimited JSONL over stdin/stdout), `SDK` (Node import).
- **Extension API:** TypeScript module exporting a factory; can register tools, slash commands, keybindings, event handlers, custom TUI widgets.
- **Skills:** Markdown `SKILL.md` files (Anthropic Agent Skills standard) — progressive disclosure, prompt-cache friendly.
- **Sessions:** JSONL with branching at `~/.pi/sessions/`. Resume via `--continue` / `--resume`.
- **Settings precedence:** CLI flags > env vars > project `.pi/settings.json` > global `~/.pi/agent/settings.json` > defaults.
- **Why this is right:** "Pi isn't a sealed product. If you need a command, tool, provider, workflow, or UI tweak, just ask Pi to build it." This *is* the kit's philosophy — bring your own workflow.

### Tauri 2

- **Source:** [tauri-apps/tauri](https://github.com/tauri-apps/tauri) — 106K stars, 14M downloads. License: Apache-2.0 OR MIT.
- **Status:** Stable since Oct 2024 (v2.0). Latest at audit time: **v2.11.1** (May 2026). MSRV: 1.77.2.
- **Frontend-agnostic** (we use React 19 + Vite + Tailwind v4 to match ELVES's tested stack).
- **Cross-platform** desktop: Linux, macOS, Windows. Mobile (iOS, Android) supported but out of scope.
- **Plugins:** auto-update, log, window-state, shell, dialog, tray-icon all official.
- **Bundle sizes:** as small as 600KB (uses system WebView).

### Codex transport

- **`codex mcp-server`:** [openai/codex docs](https://github.com/openai/codex/blob/main/codex-rs/docs/codex_mcp_interface.md) — JSON-RPC 2.0 over stdio. Two tools: `codex` and `codex-reply`. Approval callbacks (`applyPatchApproval`, `execCommandApproval`) sent server→client. **Marked experimental — subject to change without notice.**
- **`codex-codes` Rust crate:** [docs.rs/codex-codes](https://docs.rs/codex-codes) — typed JSON-RPC bindings for the underlying app-server protocol. `AsyncClient` / `SyncClient`, full thread+turn lifecycle, approval handling, streaming notifications.
- **`codex-app-server-sdk`:** [thehumanworks/codex-sdk-rs](https://github.com/thehumanworks/codex-sdk-rs) — alternative Tokio Rust SDK with stdio + WebSocket transports.
- **`codex-wrapper`:** [crates.io/crates/codex-wrapper](https://crates.io/crates/codex-wrapper) — type-safe builder pattern wrapper for the `codex` CLI subcommands; useful for `mcp add`/`mcp list` config.
- **Decision:** Use `codex-codes` for primary transport (typed, well-tested). Fall back to `codex mcp-server` via `rmcp` if it stabilises and offers benefits.

### Claude Code transport

- **No Rust SDK** ships from Anthropic. Officially Python and TypeScript only ([Agent SDK](https://docs.anthropic.com/en/docs/claude-code/sdk/sdk-typescript)).
- **Headless mode:** `claude -p "prompt" --output-format stream-json --include-partial-messages` produces JSONL events. Documented at [code.claude.com/docs/en/headless](https://code.claude.com/docs/en/headless).
- **Schema:** [stable schema request issue #53516](https://github.com/anthropics/claude-code/issues/53516) — schema is currently *not* explicitly versioned. Pin Claude Code via `pathToClaudeCodeExecutable` or detect version at startup.
- **Session JSONL:** at `~/.claude/projects/<encoded-cwd>/<uuid>.jsonl`. Append-only. Tool calls visible as `tool_use` blocks.
- **Decision:** Spawn `claude --print --output-format stream-json` from a Pi extension, parse JSONL events, expose them through Pi's tool/event surface.

### Model Context Protocol

- **`rmcp` v1.6.0** ([docs.rs/rmcp](https://docs.rs/rmcp)) — official Rust MCP SDK. 9.4M downloads, 1,040 reverse-deps. License: Apache-2.0. Tokio-based. Supports both server and client modes.
- **Decision:** Use `rmcp` for the optional kit-as-MCP-server surface (Module 8) — exposes the kit's tools (track status, learnings query, spec retrieval) so external clients (Claude Code, Cursor) can drive the workflow.

### Tree-sitter (Code Maps)

- **`tree-sitter` core:** [tree-sitter/tree-sitter](https://github.com/tree-sitter/tree-sitter). C library with first-party Rust bindings (`binding_rust`).
- **Languages:** 200+ pre-built grammars on crates.io (`tree-sitter-rust`, `tree-sitter-typescript`, `tree-sitter-python`, etc.).
- **Use:** Generate compressed structural summaries (class/function signatures only) — Repo Prompt's "Code Maps" pattern. Compresses 7M-token codebases to ~85K tokens.
- **Decision:** Rust crate (`code-maps`) using tree-sitter for the heavy lift; expose via N-API to Pi extensions and via Tauri commands to the GUI.

### context-mode

- **Source:** [mksglu/context-mode](https://github.com/mksglu/context-mode). MIT license. v1.0.89 at audit time. 12-14 platform integrations. SQLite FTS5 + BM25.
- **Tools:** `ctx_execute`, `ctx_execute_file`, `ctx_index`, `ctx_search`, `ctx_fetch_and_index`, `ctx_batch_execute`, plus `ctx_stats` / `ctx_doctor` / `ctx_upgrade` / `ctx_purge`.
- **Hooks:** PreToolUse hook auto-routes outputs; subagents learn `batch_execute` as primary tool.
- **Decision:** Launch as a managed MCP server on app start; route tool outputs through it. Document `KIT_CONTEXT_MODE_DISABLE=1` for users who don't want it.

### Storage

- **rusqlite v0.31+ with bundled SQLite + FTS5 + WAL.** Per ELVES `DECISIONS.md` rationale: bundled means no system dep; FTS5 always available. Tauri v2 dependency tree pins compatible versions.
- **Schema versioning:** schema_version table + versioned Rust migration functions. Idempotent.
- **Decision:** Adopt the same pattern.

## 4. Key Trends (2026-05)

1. **Worktree-isolated parallel agents** is now table stakes (every reference tool implements it).
2. **MCP is the standardisation point** — Anthropic's open-source protocol now has 1,040+ Rust crates and an official SDK in every major language.
3. **Output-side context optimisation** is the next frontier (context-mode is the canonical example; Cloudflare's Code Mode addressed the input side).
4. **Schema-validated agent outputs** (JSON Schema enforcement on final messages) is becoming standard, supported by both Codex (`--output-schema`) and Claude (via tool-use response shapes).
5. **Single-binary distribution** is winning over npm-installed wrappers for desktop UX (pi_agent_rust, beehive's standalone TUI).
6. **Tauri 2 over Electron** for cross-platform Rust+web desktop apps (smaller bundles, better security model, native performance).

## 5. Regulatory Considerations

None applicable — this is a developer tool, not a consumer product. UK GDPR / healthcare etc. only apply to *projects built with the kit*, not the kit itself. The `/project-security-review` skill (carried over from the bash kit) handles those concerns at the project level.

## 6. Recommendations (informing Phase 4 Blueprint)

1. **Engine: Pi (TypeScript) via npm.** Don't reimplement the agent loop.
2. **GUI: Tauri 2.11 + React 19 + Vite + Tailwind v4.** Stable, well-supported, matches ELVES's tested stack.
3. **Codex: `codex-codes` Rust crate.** Typed transport with version-insulation.
4. **Claude: `claude --print --output-format stream-json`.** No Rust SDK option; headless mode is canonical.
5. **MCP: `rmcp` 1.6.0** for both client (calling context-mode) and server (kit-as-MCP-surface).
6. **Storage: rusqlite + FTS5 (bundled).** Per-project SQLite at `~/.kit-workflow-app/<project>.db`.
7. **Code Maps: Rust crate via tree-sitter `binding_rust`.** Wrapped via N-API for the Pi extension and via Tauri commands for the GUI.
8. **context-mode: managed sidecar MCP server.** Launched on first /project-execute or /project-tracks invocation; supervised by the GUI shell.
9. **Distribution: dual.** Pi-package on npm (`@korallis/workflow-skills`), Tauri shell on GitHub Releases (signed for macOS, AppImage for Linux).
10. **Build the project using itself.** First non-trivial /project-execute should be `kit-engine` from inside an early dogfood version — proves the workflow shape survives.

---

## References

- Pi: https://pi.dev / https://github.com/earendil-works/pi
- pi_agent_rust: https://github.com/Dicklesworthstone/pi_agent_rust
- ELVES (rejected fork): https://github.com/mvmcode/elves — VISION.md, DECISIONS.md
- Repo Prompt: https://repoprompt.com / https://toolalt.com/repo-prompt/
- context-mode: https://github.com/mksglu/context-mode
- Tauri 2: https://v2.tauri.app / https://crates.io/crates/tauri
- rmcp: https://docs.rs/rmcp / https://github.com/modelcontextprotocol/rust-sdk
- Codex MCP: https://github.com/openai/codex/blob/main/codex-rs/docs/codex_mcp_interface.md
- codex-codes: https://docs.rs/codex-codes
- codex-app-server-sdk: https://github.com/thehumanworks/codex-sdk-rs
- Claude Code headless: https://code.claude.com/docs/en/headless
- Claude Agent SDK (TS): https://docs.anthropic.com/en/docs/claude-code/sdk/sdk-typescript
- Tree-sitter: https://github.com/tree-sitter/tree-sitter
- beehive: https://github.com/storozhenko98/beehive
- TUICommander: https://github.com/sstraus/tuicommander
- Routa: https://github.com/LyonVallence/routa
- racc: https://github.com/liu1700/racc
