# workflow-skills — Implementation guide

## Patterns

- **Skill content is markdown.** The bash kit's existing `.claude/skills/*/SKILL.md` files are the source of truth; port verbatim and tweak only where Pi vs Claude Code context differs (e.g. tool names, file paths).
- **Prompt templates** in `prompts/<name>.md` use Pi's `{{variable}}` syntax for slash-command argument expansion.
- **Skill loading is on-demand.** Skills load when their slash command is invoked; not at Pi startup. This is Pi's standard behaviour (progressive disclosure).
- **Schema files** live next to skills that consume them. `codex-report-schema.json` is in `skills/project-execute/` because that skill references it.

## Conventions

- File naming: `project-<verb>` per the bash kit (`project-init`, `project-spec`, etc.).
- Section ordering inside each SKILL.md matches the bash kit's structure: header → invocation → constraints → references.
- All references to other kit commands use slash form (`/project-spec`, not `project-spec`).
- British English throughout.
- Front-matter `effort` field preserved (`high` for `project-init`, `project-blueprint`, `project-execute`, `project-security-review`).

## Module boundaries

This module **owns**: skill markdown content, prompt templates, schema files (assets only), npm package metadata.

This module **must NOT**: contain TypeScript executable code (that's in `extensions/`); manage state (that's in `kit-engine`); spawn subprocesses (that's in bridges).

## Known gotchas

- **The bash kit had a heredoc-backtick bug** when generating prompts. The TypeScript equivalent is template literals — `` `${var}` `` interprets backticks. When generating prompt content programmatically, use array `.join('\n')` rather than template literals if backticks are part of the output.
- **`SKILL.md` frontmatter.** Some harnesses interpret `effort` ; some don't. Don't break by adding required fields beyond `name` + `description`.
- **Per-project overrides** are FS-only; Pi auto-discovers them via the standard skills directory walk.
- **Symlinks for cross-harness portability** require the package to install with no relative-path-into-node_modules tricks. Verify on macOS where symlink semantics can differ.

## Test patterns

- For each skill, a fixture project + assertion that running the slash command produces expected file outputs (PROJECT_BRIEF.md, etc.).
- Verify `pi install` from a tarball produces a working installation.
- Snapshot tests of schema validation (parallel.yaml, codex-report).
- Cross-harness test: symlink the package's `skills/` into `.claude/skills/`, invoke from Claude Code, assert it works.
