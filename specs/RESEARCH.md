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

## 5.4 OpenAI Codex OAuth pattern (added 2026-05-08, Round 2)

OpenAI **explicitly endorses** third-party OAuth use via the [Codex for Open Source](https://developers.openai.com/community/codex-for-oss) program. The page lists "Pi, OpenCoworkAI, Cline, OpenClaw" alongside Codex itself as supported tools. Unlike Anthropic's restrictive consumer ToS, OpenAI's policy permits any tool to authenticate users via the same OAuth client and call the Codex Responses API directly.

### Protocol constants (from Pi's open-source implementation)

Source: [`badlogic/pi-mono` `packages/ai/src/utils/oauth/openai-codex.ts`](https://github.com/badlogic/pi-mono/blob/main/packages/ai/src/utils/oauth/openai-codex.ts)

| Constant | Value |
|---|---|
| `CLIENT_ID` | `app_EMoamEEZ73f0CkXaXp7hrann` (public) |
| Authorize URL | `https://auth.openai.com/oauth/authorize` |
| Token URL | `https://auth.openai.com/oauth/token` |
| Redirect URI | `http://localhost:1455/auth/callback` (OpenAI-whitelisted) |
| Scope | `openid profile email offline_access` |
| Codex API URL | `https://chatgpt.com/backend-api/codex/responses` |
| JWT claim path | `https://api.openai.com/auth.chatgpt_account_id` |

### Required request shape

Per Codex CLI's wire format ([`codex-rs/core/src/client.rs`](https://github.com/openai/codex)):

- `store: false` — REQUIRED (ChatGPT backend rejects `true` for OAuth auth)
- `stream: true` — REQUIRED
- `instructions: <our system prompt>` — REQUIRED
- Strip message IDs from prior messages (stateless mode)
- Include `reasoning.encrypted_content` from prior turns for context continuity

### Implementation refs

- **TypeScript:** Pi's `openai-codex-responses.ts` provider (canonical reference)
- **Python:** [`codex-auth`](https://pypi.org/project/codex-auth/) — drop-in OpenAI SDK patch using OAuth
- **Python (LangChain):** [`langchain-codex-oauth`](https://pypi.org/project/langchain-codex-oauth/)
- **Swift:** [`steipete/CodexBar`](https://github.com/steipete/CodexBar/blob/v0.20/docs/codex-oauth.md) — auth headers + usage endpoint
- **TypeScript (architecture):** [`ndycode/oc-codex-multi-auth`](https://github.com/ndycode/oc-codex-multi-auth/blob/main/docs/development/ARCHITECTURE.md) — covers `store: false`, message-ID stripping, reasoning encrypted_content

We port Pi's TypeScript implementation to Rust verbatim. No Rust crate currently exists for this protocol (the `codex-codes` crate wraps the Codex CLI, not the underlying API).

### Asymmetric Claude vs Codex routing

| Provider | Path | Why |
|---|---|---|
| Claude (Anthropic) | Subprocess wrapper around `claude --print --bare` | ToS forbids third-party OAuth use |
| Codex (OpenAI) | Direct OAuth + Responses API | OpenAI explicitly endorses third-party OAuth use |

This asymmetry is mandated by the providers, not a design choice.

## 5.5 Anthropic OAuth ToS finding (added 2026-05-08)

After the initial draft of this document, follow-up research surfaced a critical constraint that overturns the Pi-as-engine recommendation. Anthropic published an explicit policy at [code.claude.com/docs/en/legal-and-compliance#authentication-and-credential-use](https://code.claude.com/docs/en/legal-and-compliance#authentication-and-credential-use):

> "OAuth authentication (used with Free, Pro, and Max plans) is intended exclusively for Claude Code and Claude.ai. Using OAuth tokens obtained through Claude Free, Pro, or Max accounts in any other product, tool, or service — including the Agent SDK — is not permitted and constitutes a violation of the Consumer Terms of Service."

Pi's [`pi-ai`](https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/providers.md) supports `pi /login anthropic` (writing an `ANTHROPIC_OAUTH_TOKEN`-style credential to `~/.pi/agent/auth.json`), but using it routes traffic through pi-ai → Anthropic API with the Max-plan OAuth token. **This violates Anthropic's consumer ToS.**

The compliant path for "use Claude via Max subscription" is to invoke Claude Code itself. Anthropic explicitly carves out `claude --print` (the headless CLI) — when `claude login` has Max credentials and `ANTHROPIC_API_KEY` is unset, the subprocess uses Max billing via OAuth, ToS-cleanly. Confirmed via:

- [hampsterx/claude-mcp-bridge](https://github.com/hampsterx/claude-mcp-bridge): "Subscription (default): `claude login` (uses your Pro/Max plan, no API credits needed)" — wraps `claude` CLI as an MCP server.
- [khalilgharbaoui/opencode-claude-code-plugin](https://github.com/khalilgharbaoui/opencode-claude-code-plugin): "wraps the Claude Code CLI (`claude`) and routes model traffic through it instead of the Anthropic HTTP API."
- [csbrandt/cc-mcp](https://github.com/csbrandt/cc-mcp): supports `systemPrompt` + `appendSystemPrompt` overrides — addresses "we don't want Claude Code's system prompt" via `claude --bare --append-system-prompt-file`.
- [joesobo/claude-max-api-proxy](https://github.com/joesobo/claude-max-api-proxy): exposes Claude as an OpenAI-compatible HTTP endpoint by wrapping the CLI.

### `claude mcp serve` is NOT the answer

Per [Anthropic docs](https://docs.claude.com/en/docs/claude-code/mcp.md): "this MCP server is simply exposing Claude Code's tools to your MCP client" — Read, Edit, Write, Bash. It exposes Claude Code's local file/shell tools, NOT the agent loop, NOT Claude-the-model. Useless for our purpose.

### Repo Prompt is itself an MCP server, not a Claude wrapper

Per [danielraffel.me/til/2026/03/24](https://danielraffel.me/til/2026/03/24/how-to-install-repoprompt-globally-in-claude-code/): Repo Prompt is a Swift native macOS app that **acts as an MCP server**. Claude Code is the CLIENT; Repo Prompt is the SERVER. Repo Prompt's tools (`context_builder`, `file_search`, `apply_edits`, `get_file_tree`) don't make LLM calls — they're pure context-engineering primitives. Inference happens inside Claude Code (billed via your `claude login`).

### Architectural implication

We **drop Pi entirely**. Our Rust core (`workflow-core`) IS the agent harness. It spawns:
- `claude --print --bare --append-system-prompt-file <ours>` for Claude (uses Max via `claude login`)
- `codex exec` (v1) or via `codex-codes` (v1.1) for Codex (uses ChatGPT Plus via `codex login`)

This is documented in `SPEC_REVISION_2026-05-08.md` and reflected throughout the rest of this RESEARCH.md.

## 6. Recommendations (informing Phase 4 Blueprint)

1. **Engine: own it. No Pi, no Node.** Rust core (`workflow-core`) spawns Claude and Codex CLIs as subprocesses. ToS-clean for both. Single-binary distribution.
2. **GUI: Tauri 2.11 + React 19 + Vite + Tailwind v4.** Stable, well-supported, matches ELVES's tested stack.
3. **Claude: `claude --print --bare --append-system-prompt-file <ours>` (subprocess).** Strips Claude Code's defaults; uses our system prompt; uses Max billing via `claude login`. ToS-clean.
4. **Codex: direct OAuth + Codex Responses API.** Replicate Pi's `openai-codex-responses` provider in Rust. PKCE flow against `auth.openai.com`, JWT account-id extraction, HTTPS calls to `chatgpt.com/backend-api/codex/responses`. No subprocess. No `codex-codes` crate. ToS-clean per OpenAI's Codex for OSS endorsement.
5. **MCP: `rmcp` 1.6.0** for client (calling context-mode); optional server surface (expose kit tools to Claude Code/Cursor) in v1.1.
6. **Storage: rusqlite + FTS5 (bundled).** Per-project SQLite at `<project>/.kit-workflow-app/state.db`; cross-project at `~/.kit-workflow-app/global.db`.
7. **Code Maps: Rust crate via tree-sitter `binding_rust`.** Direct call from skill-runner (no N-API needed since it's all Rust).
8. **context-mode: managed sidecar MCP server.** Spawned by `kit-context-mode-manager` Rust crate; connected via `rmcp` MCP client.
9. **Distribution: single Tauri bundle.** No npm publishing. Signed `.dmg`/`.app` for macOS, AppImage + `.deb` for Linux. Windows MSI in v1.1.
10. **Build the project using itself.** First non-trivial /project-execute should be `track-engine` from inside an early dogfood version — proves the workflow shape survives translation from bash to Rust.

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
