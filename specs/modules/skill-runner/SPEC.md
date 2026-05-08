# skill-runner — SPEC

> **Layer:** Core (Rust crate `kit-skill-runner`)
> **Last updated:** 2026-05-08 (Pi-removal pivot)
> **Depends on:** spec-engine, track-engine, claude-bridge, codex-bridge, context-mode-manager, code-maps, session-store

## 1. Purpose

The **agent loop** of the workflow-app. Interprets the 12 kit slash commands as bundled markdown skills, assembles the right context for each, routes to the right harness (claude-bridge or codex-bridge), processes the structured response, and persists outcomes to session-store. This is what was previously planned to be Pi + workflow-skills + kit-engine sub-domain — collapsed into a single Rust crate.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| SR-1 | gui-shell user | Click "Run /project-execute auth" | The skill runs end-to-end without me typing slash commands |
| SR-2 | skill-runner | Load `skills/project-execute.md` from the embedded asset bundle | Skills ship with the binary; per-project overrides at `<project>/.kit-workflow-app/skills/` take precedence |
| SR-3 | skill-runner | Assemble a prompt with our system prompt + spec hierarchy + Code Maps | Claude receives clean context with our brand, not Claude Code defaults |
| SR-4 | skill-runner | Validate the harness's structured report against the schema | Bleed-through and spec deviations fail loudly |
| SR-5 | skill-runner | Apply `proposed_commits` from the report | Orchestrator-commits is canonical and automated |

## 3. Public API

```rust
pub struct SkillRunner {
    spec: Arc<SpecEngine>,
    tracks: Arc<TrackEngine>,
    claude: Arc<ClaudeBridge>,
    codex: Arc<CodexBridge>,
    context_mode: Option<Arc<ContextModeManager>>,
    code_maps: Arc<CodeMaps>,
    store: Arc<SessionStore>,
}

impl SkillRunner {
    pub async fn run(&self, skill: SkillName, args: SkillArgs) -> Result<SkillResult>;
    pub fn list_skills(&self) -> Vec<SkillMeta>;
    pub fn skill_markdown(&self, name: &str) -> Result<String>;       // for UI display
}

pub enum SkillName {
    ProjectInit, ProjectResearch, ProjectBlueprint, ProjectSpec,
    ProjectModule, ProjectExecute, ProjectTracks, ProjectReview,
    ProjectSecurityReview, ProjectStatus, ProjectDeploy, ProjectTest,
}
```

## 4. Skill markdown format

Bundled at `crates/kit-skill-runner/skills/<name>.md`. Frontmatter:

```yaml
---
name: project-execute
description: Implement a fully-specced module via dual-harness execution
effort: high
harness: codex                 # default harness for this skill
inputs:
  module: { type: string, kebab-case }
---
```

Body is the human-readable instructions for the agent (markdown). skill-runner passes the body verbatim to claude-bridge / codex-bridge as part of the prompt.

## 5. System-prompt assembly

For Claude (via `claude-bridge --append-system-prompt-file`):

```text
You are the agent inside the workflow-app, a disciplined spec-first coding tool.
You have access to file edit tools, bash, glob, grep. You do NOT have web access
unless explicitly granted by the skill.

Project root: {{ROOT}}
Active skill: {{SKILL_NAME}}
Active module (if applicable): {{MODULE}}

## Operating model (from CLAUDE.md, the kit's root operating doc)

{{ROOT_CLAUDE_MD}}

## Spec hierarchy excerpts

{{RELEVANT_SPECS}}

## Skill instructions

{{SKILL_BODY}}

## Output contract

{{SCHEMA_REFERENCE}}     # only for codex (which natively enforces) — for claude we
                          # ask for JSON output and validate after.
```

## 6. Business rules

- **Skills are markdown content, not code.** No TypeScript runtime; no `eval`.
- **Per-project overrides:** if `<project>/.kit-workflow-app/skills/<name>.md` exists, it takes precedence over the bundled one.
- **System prompt strips Claude Code defaults.** `claude --bare --append-system-prompt-file` skips `~/.claude/CLAUDE.md`, hooks, MCP servers, and built-in skills — we own the context.
- **Schema enforcement.** For Codex, pass `--output-schema codex-report-schema.json`. For Claude, request JSON in the prompt and validate after with the same schema.
- **Cross-check final reports** against `git status` / test output before applying `proposed_commits`. Bleed-through detection.
- **All tool outputs route through context-mode-manager** (when enabled) for context efficiency.
- **Read-path scrubbing** — every artefact (`.jsonl`, `-report.json`, `-last.md`) passes through the bundled `kit-scrub` regex set before re-entering Claude Rust string state.

## 7. Integration points

| Module | Relationship |
|---|---|
| spec-engine | Reads spec hierarchy for prompt assembly |
| track-engine | Provides DispatchFn for `/project-tracks start` |
| claude-bridge | Routes Claude skills here |
| codex-bridge | Routes Codex skills here |
| context-mode-manager | Optional sidecar; routes tool outputs |
| code-maps | Generates token-budgeted Code Maps for prompts |
| session-store | Appends learnings, snapshots specs |
| gui-shell | Top-level UI uses `SkillRunner::run` |

## 8. Acceptance criteria

- [ ] All 12 skills bundled as markdown assets via `include_str!`.
- [ ] Per-project override path checked first; bundled fallback works.
- [ ] `/project-execute auth` runs end-to-end on a fixture: spec read → prompt assembled → codex-bridge spawned → schema-valid report → proposed_commits cross-checked vs `git diff` → commits applied with Co-Authored-By.
- [ ] `/project-tracks start auth billing` calls track-engine with the right DispatchFn; tracks visible in tracks.json.
- [ ] System prompt does NOT include "You are Claude Code, Anthropic's official CLI" string when claude-bridge is used.
- [ ] Read-path scrubber catches `sk-ant-...`, `sk-...`, `ghp_...`, `xox[abps]-...`, JWTs.
- [ ] All public APIs are `Send + Sync` for Tauri.

## 9. Out of scope

- Runtime extension of skills (no plugin system in v1; users edit per-project markdown overrides).
- Custom system-prompt generators (the prompt is template-based).
- Multi-LLM voting / consensus.

## 10. Open questions

- [ ] **Where does the system-prompt template live?** `crates/kit-skill-runner/system-prompt.md` as a single template; or per-skill templates? Single template for v1.
- [ ] **Tool calling.** When Claude wants to read a file, does it use its built-in `Read` tool (via the spawned `claude` subprocess) or do we proxy via context-mode? Probably let Claude's tools handle it; context-mode is for outputs.
