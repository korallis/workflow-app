# CLAUDE.md — Operating Model for AI Project Kit

This document defines how Claude operates within the AI Project Kit. It serves as the instruction set for every session and guides decision-making at all levels.

---

## The Prime Directive

**Every Claude session operates as a dedicated project specialist.** The goal is not to jump between tasks, but to drive ONE project forward from concept to completion in a single unbroken workflow.

A session succeeds when:
- It captures the full scope of work needed
- It executes each phase in sequence without distraction
- It captures learnings at the end for future sessions
- The project advances measurably (code written, architecture decided, tests passing, deployed)

---

## Trigger Recognition

Sessions are triggered by explicit commands that signal intent:

| Trigger | Meaning |
|---------|---------|
| `/project-init [idea]` | New project or major feature — start from zero |
| `/project-research [topic]` | Deep research mode — explore domain/tech/regulation |
| `/project-blueprint` | Architecture mode — system design or regeneration |
| `/project-spec [module]` | Specification mode — detailed module design |
| `/project-module [name]` | Implementation mode — code and tests |
| `/project-execute [module]` | Dual-harness mode: hand a fully-specced module to Codex CLI for implementation while Claude orchestrates. Live tmux pane in your most-recent attached session. |
| `/project-security-review` | Independent Agent-based security review of pending changes — UK GDPR / healthcare focus. |
| `/project-review` | Wrap-up mode — capture learnings |
| `/project-status` | Dashboard — show current state |
| `/project-deploy` | Deployment mode — deploy and verify |
| `/project-test` | Testing mode — comprehensive test pass |

These are not suggestions—they are **explicit signals** that Claude should enter a specific operational mode.

---

## Workflow

Every Claude session follows a 5-phase sequence, executed in order:

### Phase 1: Plan
- Understand the task
- Identify unknowns
- Map dependencies
- Document assumptions
- Create a todo list if multi-step

**Exit criteria:** Clear scope, no surprises ahead

### Phase 2: Research (if needed)
- Answer unknowns via Exa/Ref documentation search
- Verify current best practices for the stack
- Check competitor implementations (market context)
- Record findings for later reference

**Exit criteria:** All technical questions answered, patterns documented

### Phase 3: Execute
- Build incrementally (small, testable units)
- Verify each piece before continuing
- Keep terminal output clean
- Push incremental commits with clear messages

**Exit criteria:** Code written, feature complete, passing tests

### Phase 4: Verify
- Visually test the change (Playwright if UI)
- Check console for errors
- Verify against acceptance criteria
- Document any gaps or next steps

**Exit criteria:** Change works as intended, no breaking changes introduced

### Phase 5: Capture
- Summarize what was built
- Document unexpected learnings
- Note patterns for reuse
- Update LEARNINGS.md
- Link to relevant PRs/commits

**Exit criteria:** Learnings captured, handoff ready for next session

---

## Dual-Harness Workflow

Some tasks are big enough that you want Claude (Opus 4.7) to plan and review while Codex CLI (gpt-5.5) does the heavy implementation. The kit supports this via `/project-execute`:

- **Plan + Review**: Claude Code (this session). Reads specs, builds the dispatch prompt, reads back the scrubbed structured report + JSONL events, smoke-tests the working tree, **applies commits proposed by Codex** (orchestrator-commits pattern), summarises, runs the review skill.
- **Execute**: Codex CLI (`gpt-5.5`, medium reasoning effort), launched via `.claude/lib/dispatch.sh` with `--json --output-schema`. Runs in a live tmux pane that splits into your most-recent attached session.

Single-harness mode (`/project-module`) keeps everything in Claude. Use dual-harness when the module is large, well-specced, and you want to watch implementation happen in real time.

**Orchestrator-commits is canonical.** Codex does NOT commit. It leaves the working tree dirty and emits a structured final report (schema at `.claude/skills/project-execute/codex-report-schema.json`) listing `proposed_commits`. Claude reviews the diff, smoke-tests, and applies commits with `Co-Authored-By` attribution. This sidesteps Codex's `workspace-write` sandbox `.git` restriction and gives a verification gate that catches spec deviations and training-data bleed-through before they land in history.

**Prerequisites**:

- `npm install -g @openai/codex` (Codex CLI 0.128+ tested).
- Authenticate. Two paths with different model availability:
  - `codex login` — ChatGPT auth. Required for `gpt-5.5` access without API tier requirements.
  - `export OPENAI_API_KEY=…` — API-key auth. `gpt-5.5` requires Tier 1+ on your OpenAI org; if your org lacks the tier, the preflight will fail with a model-availability error.
- A tmux session with at least one attached client. dispatch.sh detects the most-recent attached client via `tmux list-clients` — Claude Code itself doesn't have to be inside tmux as long as one client is attached somewhere. Override with `KIT_TMUX_SESSION=<name>` when you have multiple sessions.

Portability: dispatch.sh works on Linux and macOS. Requires GNU coreutils (`timeout` on Linux, `gtimeout` after `brew install coreutils` on macOS). Lock is mkdir-based (no `flock` dependency).

---

## Available Slash Commands

| Command | Usage | Purpose |
|---------|-------|---------|
| `/project-init` | `/project-init [idea]` | Start a new project or major feature from scratch |
| `/project-research` | `/project-research [topic]` | Deep research on domain, technology, or regulations |
| `/project-blueprint` | `/project-blueprint` | Generate or regenerate master architecture design |
| `/project-spec` | `/project-spec [module]` | Create or update a detailed module specification |
| `/project-module` | `/project-module [name]` | Implement a specific module end-to-end |
| `/project-review` | `/project-review` | End-of-session: capture learnings and progress |
| `/project-status` | `/project-status` | Display project dashboard and current state |
| `/project-deploy` | `/project-deploy` | Deploy to staging/production and verify |
| `/project-test` | `/project-test` | Comprehensive test pass across all modules |
| `/project-execute` | `/project-execute [module]` | Dual-harness mode: hand a fully-specced module to Codex CLI for implementation while Claude orchestrates |
| `/project-security-review` | `/project-security-review` | Independent Agent-based security review of pending changes — UK GDPR / healthcare focus. |

---

## Available Tools & Integrations

Claude has access to specialized tools for research, testing, and deployment. Use these before implementing:

### Research & Documentation
- **Exa Search** (`web_search_exa`) — Web search with category filters (company, research paper, people). Use for market research, competitor analysis, and domain exploration.
- **Exa Code Context** (`get_code_context_exa`) — Find code examples from GitHub, Stack Overflow, and official docs. Use for framework patterns and implementation examples.
- **Ref Documentation** (`ref_search_documentation`) — Search framework and library documentation. Use to verify API patterns, latest versions, and best practices.
- **Ref URL Reader** (`ref_read_url`) — Read full documentation pages. Use after finding relevant docs via search.

### Testing & Verification
- **Playwright** — Browser automation for E2E testing and visual verification. Navigate pages, take screenshots, check accessibility, read console errors.
- **Chrome Automation** — Alternative browser control for testing and verification.

### Deployment
- **Vercel** — Deploy previews and production builds. Monitor build logs, check deployment status.

### Frontend Development
- **web-artifacts-builder skill** — React, Tailwind CSS, and shadcn/ui component patterns. Consult when building frontend modules.

### Hooks
- **PreCompact snapshot hook** (`.claude/hooks/pre-compact.sh`) — Writes recovery snapshots into `specs/sessions/` before conversation compaction. Configured by `.claude/settings.json`.
- **SessionStart compact backup** — On compacted-session resume, prints the latest snapshot path when one exists. Claude Code issue #13572 means PreCompact may not fire reliably for `/compact` on some versions, so treat this as best-effort recovery.

### Rule: Research Before Implementing
**Before implementing any technical pattern you're uncertain about, use Ref or Exa to look up current documentation.** Don't rely on potentially outdated training knowledge for version-specific API details. This ensures consistency with the latest tooling and avoids rework.

---

## Spec Hierarchy

Specifications follow a three-tier hierarchy from abstract to concrete:

### Tier 1: Blueprint (System Design)
- Component relationships
- Data flow
- Integration points
- Risk assessment

**Example:** "User service talks to Auth service via REST; both write to PostgreSQL"

### Tier 2: Module Spec (Detailed Design)
- Function signatures
- Input/output contracts
- Error handling
- Dependencies

**Example:** "POST /users takes { email, password }, returns { id, token } or { error }"

### Tier 3: Code (Implementation)
- Actual working code
- Tests
- Documentation
- Deployment

**Example:** Implemented function with type safety, error handling, and unit tests

Each tier is concrete enough that a developer can execute it without debate. Never skip a tier.

---

## Session Rules

1. **One session = one task.** Don't start a second task until the first is captured and ready for handoff.

2. **Plan mode first.** Before writing code, create a todo list and confirm scope.

3. **Research before building.** Use Ref or Exa to verify technical patterns before implementing.

4. **Commit early, commit often.** Push small, focused commits with clear messages. Easier to review and revert if needed.

5. **Verify as you go.** Don't leave broken code in the branch. Each phase should be testable.

6. **Capture learnings at the end.** Spend 5 minutes summarizing what you learned and what surprised you. Record in LEARNINGS.md.

7. **Link to context.** When you finish, leave breadcrumbs: commit hashes, PR links, file paths. Next session should be able to pick up immediately.

---

## Boundaries

### ✅ Always
- Use Exa/Ref to verify technical details before implementing
- Visually verify UI changes with Playwright when possible
- Commit frequently with clear messages
- Check test output before declaring success
- Ask for clarification if scope is ambiguous
- Use type safety (TypeScript, Pydantic, etc.)
- Document public APIs with examples
- Link learnings to specific code changes

### ⚠️ Ask Before
- Creating major new files or directories
- Changing system architecture
- Adding new dependencies
- Modifying existing APIs
- Deleting code or data
- Deploying to production
- Merging to main branch

### 🚫 Never
- Commit without testing
- Skip error handling
- Leave console warnings in code
- Assume API behavior—read the docs
- Deploy broken branches
- Merge without a clear reason in the commit message
- Ignore accessibility concerns in UI

---

## Stack Detection

Before starting implementation, answer these four questions to understand the project:

1. **Frontend:** React? Vue? Svelte? Plain HTML? (or N/A if backend-only)
2. **Backend:** Node.js? Python? Go? Rust? (or N/A if frontend-only)
3. **Database:** PostgreSQL? MongoDB? Firestore? (or N/A if not applicable)
4. **Deployment:** Vercel? Docker? Lambda? Self-hosted? (or N/A)

These determine which tools and patterns you'll use throughout.

---

## Quality Standards

- **Code:** Passes linting, has no console errors, uses type safety
- **Tests:** Unit tests for logic, E2E tests for user flows, passing locally before push
- **Docs:** Public functions have docstrings/comments, modules have README sections, complex logic is explained
- **UX:** Accessible (WCAG AA minimum), responsive (mobile-first), keyboard-navigable
- **Deployment:** Builds without warnings, passes CI/CD, rolls back cleanly if issues found

---

## Learnings

After every session, update `LEARNINGS.md` with:

1. **What worked well** — patterns, tools, approaches that accelerated progress
2. **What was harder than expected** — gotchas, surprises, missing documentation
3. **What to do differently next time** — concrete changes for future sessions
4. **Links to the work** — commit hashes, PRs, files touched

This creates a knowledge base that future sessions can learn from immediately.

---

## Session Checklist

At the **start** of a session:
- [ ] Read this file (you're doing it now)
- [ ] Check LEARNINGS.md for context
- [ ] Run `/project-status` if this is a continuation
- [ ] Create a todo list
- [ ] Confirm scope with the user

At the **end** of a session:
- [ ] All code tested and committed
- [ ] All todos marked done or moved to next session
- [ ] LEARNINGS.md updated
- [ ] Summary provided with links to work
- [ ] Clear handoff notes for next session

---

**This is the operating model. Follow it.**
