# Spec Revision — 2026-05-08

> Records the pivot from "Pi-package + Tauri shell" to "pure Rust core + Tauri shell, no Pi". This document is the audit trail; the actual specs are updated in place.

## Trigger

Operator flagged: "PI issue is it doesnt use Claude code as an MCP so the claude usage is paid via the extra usage."

## Research findings (Exa + Ref, 2026-05-08)

### 1. Pi DOES support OAuth subscription billing — but it's a Terms-of-Service trap for Anthropic

Per [`badlogic/pi-mono` providers docs](https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/providers.md), Pi has built-in OAuth flows for both ChatGPT Plus/Pro (Codex) AND Claude Pro/Max:

```
pi /login openai-codex   # ChatGPT Plus/Pro subscription
pi /login anthropic       # Claude Pro/Max subscription (technically supported)
```

**However**, Anthropic published an explicit policy at [code.claude.com/docs/en/legal-and-compliance#authentication-and-credential-use](https://code.claude.com/docs/en/legal-and-compliance#authentication-and-credential-use):

> "OAuth authentication (used with Free, Pro, and Max plans) is intended exclusively for Claude Code and Claude.ai. Using OAuth tokens obtained through Claude Free, Pro, or Max accounts in any other product, tool, or service — including the Agent SDK — is not permitted and constitutes a violation of the Consumer Terms of Service."

This means:
- **Codex via Pi:** ✅ ToS-compliant. ChatGPT Plus subscription is the legitimate billing path.
- **Claude via Pi:** ⚠️ Pi technically supports it but using OAuth tokens *outside* of Claude Code itself violates Anthropic's consumer ToS.

The ToS-clean path for Claude-via-subscription is to **invoke Claude Code itself** (not steal its OAuth token for use elsewhere). Confirmed by the ecosystem of community wrappers that take exactly this approach:

- [`hampsterx/claude-mcp-bridge`](https://github.com/hampsterx/claude-mcp-bridge): "Subscription (default): `claude login` (uses your Pro/Max plan, no API credits needed)"
- [`khalilgharbaoui/opencode-claude-code-plugin`](https://github.com/khalilgharbaoui/opencode-claude-code-plugin): "wraps the Claude Code CLI (`claude`) and routes model traffic through it instead of the Anthropic HTTP API"
- [`csbrandt/cc-mcp`](https://github.com/csbrandt/cc-mcp): supports `systemPrompt` / `appendSystemPrompt` overrides — addresses the "we don't want Claude Code's system prompt" concern via `--append-system-prompt-file`.
- [`joesobo/claude-max-api-proxy`](https://github.com/joesobo/claude-max-api-proxy): exposes Claude as an OpenAI-compatible HTTP endpoint by wrapping the CLI.

### 2. `claude mcp serve` is NOT what we wanted

Per [Anthropic docs](https://docs.claude.com/en/docs/claude-code/mcp.md):

> "this MCP server is simply exposing **Claude Code's tools** to your MCP client" — Read, Edit, Write, Bash, etc.

It exposes Claude Code's **local file/shell tools**, not the agent loop, and not Claude-the-model. Useless for our purpose: we'd still need our own agent loop AND would still need to call Claude-the-model from somewhere else.

### 3. Repo Prompt is itself an MCP server, not a Claude wrapper

Per [danielraffel.me/til/2026/03/24](https://danielraffel.me/til/2026/03/24/how-to-install-repoprompt-globally-in-claude-code/) and [Repo Prompt's blog](https://repoprompt.com/blog/building-repo-bench): Repo Prompt is a Swift native macOS app that **acts as an MCP server** (`/Applications/Repo Prompt.app/Contents/MacOS/repoprompt-mcp`). Claude Code is the CLIENT; Repo Prompt is the SERVER. Repo Prompt's tools (`context_builder`, `file_search`, `apply_edits`, `get_file_tree`) don't make LLM calls — they're pure context-engineering primitives. Inference happens inside Claude Code (billed via your `claude login`).

## Architectural pivot

### Before (now obsolete)

```
Tauri+React shell ──RPC──► Pi (Node.js, embedded)
                            ├── workflow-skills (npm pi-package)
                            ├── kit-engine (Pi extension)
                            ├── claude-bridge (Pi extension; pi-ai Anthropic provider)  ← ToS-grey
                            ├── codex-bridge (Pi extension; codex-codes via N-API)
                            └── context-mode-bridge (Pi extension)
```

### After (this revision)

```
Tauri+React shell ──Tauri commands──► workflow-core (Rust)
                                      ├── spec-engine
                                      ├── track-engine
                                      ├── skill-runner (interprets bundled markdown skills)
                                      ├── claude-bridge ──spawn──► `claude --print --bare --append-system-prompt-file <ours>`
                                      │                                    └─ uses claude login → Max plan ✅ ToS-clean
                                      ├── codex-bridge ──spawn──► `codex exec` (or codex-codes Rust crate)
                                      │                                    └─ uses codex login → ChatGPT Plus ✅ ToS-clean
                                      ├── context-mode-manager ──spawn──► context-mode (MCP server sidecar)
                                      ├── session-store (rusqlite + FTS5)
                                      └── code-maps (tree-sitter)
```

Workflow skills (`/project-init` etc.) are markdown content bundled with the app and interpreted by `skill-runner`. They're not code modules.

## Why this is better than the Pi-based plan

| Concern | Pi plan | Rust-only plan |
|---|---|---|
| ToS compliance for Claude-via-Max | ⚠️ Grey area (pi-ai Anthropic provider violates Anthropic consumer ToS) | ✅ Clean (we invoke Claude Code itself, exactly the path Anthropic carves out) |
| User's "we don't want Claude Code's system prompt" | Hard — Pi controls the prompt | Easy — `claude --bare --append-system-prompt-file <ours>` strips defaults |
| Distribution | npm + Tauri = two install paths | Single Tauri bundle |
| Runtime dependencies | Tauri + Node 22 + Pi binary + npm package | Tauri only (claude + codex CLIs detected on user's PATH) |
| Subprocess count at runtime | Tauri + Node + Pi + extension subprocesses | Tauri + per-call subprocesses (claude/codex/context-mode) |
| Debugging | Multi-runtime (Rust ↔ Node ↔ Pi extensions) | Single runtime (Rust) |
| Multi-provider future-proofing | Pi gives 20+ providers for free | We add providers ourselves (next: Gemini via gemini CLI) |
| Workflow ownership | Pi controls agent loop; we override prompts | We own the loop end-to-end |

## What stays the same

- Spec hierarchy unchanged: `PROJECT_BRIEF.md` → `MASTER_BLUEPRINT.md` → `MODULES.md` → per-module `SPEC.md`/`CLAUDE.md`/`parallel.yaml`.
- Compound-learning model unchanged.
- Parallel-tracks model unchanged (spec, semantics, all CodeRabbit-vetted invariants from the bash kit).
- GUI scope unchanged: plan board, file picker, Code Maps, Apply Mode, onboarding.
- License (MIT), platform scope (Linux+macOS v1, Windows v1.1), auth detection model (no token storage).
- Tauri 2.11, React 19 + Vite + Tailwind v4, rusqlite + FTS5, tree-sitter binding_rust, rmcp 1.6.0.

## What changes (file-by-file)

| File | Change |
|---|---|
| `PROJECT_BRIEF.md` | Drop Pi mentions; add "ToS compliance for Claude billing" to constraints. |
| `RESEARCH.md` | Add §"Anthropic OAuth ToS finding" + community wrapper inventory. Mark Pi's Anthropic provider as "ToS-grey, not used." |
| `MASTER_BLUEPRINT.md` | Replace §2 (Tech Stack) — remove Pi, Node, npm distribution. Replace §5 (Process model) — Rust core spawns CLIs directly. |
| `MODULES.md` | Replace 8-module Pi-centric list with 9 Rust crates: session-store, code-maps, spec-engine, track-engine, claude-bridge, codex-bridge, context-mode-manager, skill-runner, gui-shell. |
| `ROADMAP.md` | Sprint 1.1–1.5 rewritten as Rust-only; no pi-package step. |
| `modules/workflow-skills/` | DELETED — was a pi-package shape. Skill markdown lives inside `crates/skill-runner/skills/`. |
| `modules/kit-engine/` | SPLIT into `modules/spec-engine/` + `modules/track-engine/` + `modules/skill-runner/`. |
| `modules/claude-bridge/` | Rewritten — Rust crate, not Pi extension. Adds `--bare --append-system-prompt-file` invariants. |
| `modules/codex-bridge/` | Rewritten — Rust crate, uses codex-codes; subprocess pattern matches claude-bridge for consistency. |
| `modules/context-mode-bridge/` | Renamed `context-mode-manager`; supervises subprocess + connects via rmcp client. |
| `modules/gui-shell/` | Mostly unchanged; drop Pi RPC, replace with direct Tauri commands. |

## Decisions ratified in this revision

| # | Question | Decision |
|---|---|---|
| 1 | Drop Pi? | **Yes**, entirely. |
| 2 | Claude routing? | Subprocess wrapper around `claude --print --bare --append-system-prompt-file`. Uses `claude login` → Max. |
| 3 | Codex routing? | Subprocess wrapper. Uses `codex login` → ChatGPT Plus. Same shape as Claude for consistency. |
| 4 | Multi-provider abstraction? | None in v1. Both bridges are bespoke. Future Gemini bridge follows the same subprocess pattern. |
| 5 | Where do workflow skills live? | Bundled markdown files inside `crates/skill-runner/skills/`. Not a separate npm package. |
| 6 | Distribution? | Single Tauri bundle (signed for macOS, AppImage for Linux). No npm publishing. |
