# skill-runner — Implementation guide

## Patterns

- **Skills as embedded markdown** via `include_str!("skills/project-execute.md")`. One `lazy_static!` map of bundled skills loaded at compile time.
- **Per-project override check** at runtime: `<project>/.kit-workflow-app/skills/<name>.md` first, then the bundled one.
- **Frontmatter parsing** via `gray_matter` crate.
- **Prompt assembly** via `tera` (Jinja2-like) templates in `crates/kit-skill-runner/templates/`.
- **One `tokio::spawn` per skill run**; cancellation token threaded through to bridges.

## Conventions

- Crate name: `kit-skill-runner`.
- Skill names match the bash kit's slash commands: `project-init`, `project-research`, etc.
- System-prompt template: `crates/kit-skill-runner/templates/system-prompt.md`.
- Per-skill bodies bundled in `crates/kit-skill-runner/skills/`.

## Module boundaries

This module **owns**: skill content, prompt assembly, dispatch routing, report cross-check.

This module **must NOT**: bypass the bridges; talk directly to subprocesses; render UI.

## Known gotchas

- **Bash kit's heredoc-backtick bug.** TypeScript template literals had this. Rust `format!` doesn't, but `tera` templates do — use `{% raw %}` for code fences inside templates.
- **System prompt MUST be passed via file**, not CLI arg — `--append-system-prompt-file` not `--append-system-prompt` (the latter has shell-quoting fragility).
- **Per-project skill overrides** must be validated as paths relative to the project root; never absolute, never `..`-bearing.
- **Schema enforcement on Claude.** Claude doesn't natively support `--output-schema`; we ask for JSON in the prompt + validate after. Some prompts will produce free text first; the validator must look for the JSON block (extract from ```json fence).

## Test patterns

- Mock `ClaudeBridge` and `CodexBridge` via traits; assert correct prompt content on each.
- Snapshot-test the assembled system prompt for the 12 skills.
- Per-project override test: create override file, assert it overrides the bundled.
- Schema cross-check: feed a known-bad report (e.g. `phases_completed: ["OMX autopilot"]`); assert mismatch flagged.

## Cargo dependencies

- `tokio`, `serde`, `serde_yaml`, `serde_json`, `gray_matter`, `tera`, `jsonschema`, `tracing`
- Sister crates: `kit-spec-engine`, `kit-track-engine`, `kit-claude-bridge`, `kit-codex-bridge`, `kit-context-mode-manager`, `kit-code-maps`, `kit-session-store`
