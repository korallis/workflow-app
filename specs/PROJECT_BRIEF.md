# PROJECT_BRIEF — workflow-app

> **Status:** Draft (Phase 1 of /project-init)
> **Owner:** Lee Barry
> **Last updated:** 2026-05-08

## Overview

`workflow-app` is a **solo-dev coding workflow tool**. It packages the `korallis/workflow` AI Project Kit — spec-first / dual-harness / parallel-tracks development — into a single-install Tauri+Rust desktop application.

The user installs one app, signs into Claude Code (`claude login` → Max plan) and Codex CLI (`codex login` → ChatGPT Plus), and gets the full kit workflow without touching slash commands or shell scripts: `init → research → blueprint → module specs → roadmap → dual-harness implementation → isolated review → compound learning → parallel tracks across worktrees`.

The Tauri+React shell provides a Repo-Prompt-style **visual context-engineering** surface (file picker, tree-sitter Code Maps, Apply Mode diff viewer) and a parallel-tracks dashboard. The engine is a Rust core (`workflow-core`) that spawns Claude and Codex CLIs as subprocesses — uses **the user's existing subscriptions**, never API keys (when configured correctly).

> **Architectural pivot 2026-05-08:** Earlier drafts proposed a Pi-based engine (TypeScript pi-package). Dropped after Anthropic clarified that OAuth tokens for Pro/Max plans are "intended exclusively for Claude Code and Claude.ai. Using OAuth tokens obtained through Claude Free, Pro, or Max accounts in any other product, tool, or service — including the Agent SDK — is not permitted." Pi's Anthropic provider would have violated this. We now invoke Claude Code itself via `claude --print --bare`, which is the explicitly-permitted path. See `SPEC_REVISION_2026-05-08.md`.

## Problem Statement

The bash-skill version of the kit (`korallis/workflow`) works but has hard floors:
- **Discoverability is zero.** Users must know the slash commands exist.
- **Context engineering is invisible.** Files get stuffed in prompts; tokens are wasted; there's no Repo-Prompt-style visual selection.
- **Parallel tracks rely on tmux.** Cross-platform, cross-machine, and headless-CI usage is awkward.
- **Auth is folklore.** Users have to know about `claude login`, `codex login`, `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `gtimeout`, etc.
- **Compound learning is filesystem grep.** No queryable surface, no UI.

A desktop app driven by Pi solves all of these without abandoning the kit's disciplined workflow shape.

## Target Users

**Primary:** Solo developers who already use Claude Code and/or Codex CLI for serious engineering work, are comfortable with a terminal, and want stronger structure than ad-hoc agent sessions.

**Secondary:** Two-to-three person teams who want shared spec hierarchy + parallel module implementation.

**Not the target:** Newcomers to AI-assisted coding, vibe-coders, large enterprise teams (they have Cursor / Cline / Continue / managed offerings).

## Success Criteria

1. **One-install onboarding.** From `npm install -g` (or download Tauri bundle) to "running my first /project-init" in under five minutes for a user who already has Claude+Codex CLIs installed.
2. **Cross-platform.** Linux + macOS in v1; Windows in v1.1.
3. **No regression vs the bash kit.** Every kit slash command (`/project-init`, `/project-spec`, …, `/project-tracks`) is reachable in the app, with at least the bash kit's capabilities.
4. **Visual context engineering at parity with Repo Prompt** for the file-selection + Code Maps + Apply Mode flows.
5. **Compound learning is queryable.** SQLite-backed `LEARNINGS` with FTS5 full-text search; UI to browse/filter past learnings.
6. **Context efficiency.** Embed [context-mode](https://github.com/mksglu/context-mode) as a managed sidecar; default sessions extend from ~30 min to ~3 hr (per their published benchmarks).
7. **Open source, MIT.** Pi-package on npm; Tauri shell binaries on GitHub Releases.
8. **Dogfood.** This very project (`workflow-app`) is built using `workflow-app` itself once it's bootstrapped — proves the workflow shape survives translation from bash to Pi.

## Constraints

| Constraint | Implication |
|---|---|
| Anthropic ToS on OAuth | OAuth tokens for Free/Pro/Max plans may NOT be used outside Claude Code/Claude.ai. We invoke `claude --print --bare` (Claude Code itself); we never ingest the token. ToS-clean. |
| Anthropic ships no Rust SDK | Claude integration is subprocess + JSONL parsing. `--bare` mode strips Claude Code's defaults so our own system prompt has clean control. |
| Codex `mcp-server` is experimental | v1 uses `codex exec` subprocess for parity with claude-bridge; behind a `CodexTransport` trait for v1.1 swap to `codex-codes` (typed JSON-RPC). |
| `ANTHROPIC_API_KEY` env var | If set, Claude Code uses API billing instead of Max. Onboarding detects and warns. |
| Solo-developer focus | No team features in v1 (no shared cloud state, no multi-user auth). |
| Cross-platform | Linux + macOS in v1; Windows in v1.1. Avoid platform-specific APIs in shared code. |
| Auth detection only | App never stores tokens; trusts `claude login` / `codex login` / env vars. Reduces security surface. |

## Out of scope (v1)

- Cloud sync / team collaboration / shared spec repositories.
- Hosted backend. The app is fully local.
- Custom LLM provider integrations (Pi already supports 15+ via `pi-ai`).
- Mobile (iOS/Android), even though Tauri 2 supports them.
- Voice input, agent personalities, animated avatars (the ELVES anti-pattern).
- Auto-fixing CI failures, GitHub PR auto-creation. Stay aligned with the bash kit: orchestrator-commits, user pushes.
- A new "agent loop". Pi's loop is canonical.
- Production support for arbitrary multi-repo monorepos in v1 (single git repo per project).

## Non-goals (forever)

- Becoming an IDE replacement. Cursor / VS Code / Zed are IDEs; this is the *spec-and-orchestrate* layer that drives them.
- Adding "fun" features at the cost of disciplined engineering UX. The personality-driven model (e.g. ELVES) is explicitly rejected.
