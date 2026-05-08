# workflow-skills — SPEC

> **Status:** Phase 5 of /project-init
> **Layer:** Engine (TypeScript pi-package, npm: `@korallis/workflow-skills`)
> **Last updated:** 2026-05-08

## 1. Purpose

The user-visible kit surface as a **pi-package**: 12 slash commands (prompt templates) + matching skills (`SKILL.md` files in Anthropic Agent Skills format). Drop-in port of `.claude/skills/*` from the bash kit. Distributed via npm; loaded by Pi via `pi install npm:@korallis/workflow-skills`.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| WS-1 | Solo dev with the app installed | Type `/project-init "build me a CRM"` in the GUI prompt | Pi fans out the kit's full 7-phase init flow |
| WS-2 | Power user | Skill content is markdown I can edit per-project under `.kit-workflow-app/skills/` | Custom workflow tweaks don't require fork+rebuild |
| WS-3 | New user | A `/project-status` command shows me what specs exist | Onboarding to an in-progress project is one command |
| WS-4 | Anyone | Skill files are versioned + semver-tagged on npm | I can pin to a known kit version per project |
| WS-5 | Future Claude Code user | The same skill files work in `.claude/skills/` | Cross-harness portability is preserved |

## 3. Skill catalogue

| Slash command | Skill | Notes vs bash kit |
|---|---|---|
| `/project-init` | `project-init/SKILL.md` | Same 7-phase flow. Calls `kit_spec_save` for outputs. |
| `/project-research` | `project-research/SKILL.md` | Uses Pi's web tools + `ctx_fetch_and_index`. |
| `/project-blueprint` | `project-blueprint/SKILL.md` | — |
| `/project-spec` | `project-spec/SKILL.md` | Validates `parallel.yaml` via `kit_spec_validate_parallel_yaml`. |
| `/project-module` | `project-module/SKILL.md` | Single-harness via `kit_claude_execute`. |
| `/project-execute` | `project-execute/SKILL.md` | Dual-harness via `kit_codex_execute`. Uses `codex-report-schema.json`. |
| `/project-tracks` | `project-tracks/SKILL.md` | Multi-subcommand: `plan`, `start`, `status`, `review`, `merge`, `cleanup`. v1 ships `plan` + `start`. |
| `/project-review` | `project-review/SKILL.md` | Appends to `LEARNINGS.md` AND to SQLite via `kit_learning_append`. |
| `/project-security-review` | `project-security-review/SKILL.md` | Spawns isolated review agent via Pi's spawn API. |
| `/project-status` | `project-status/SKILL.md` | Reads SQLite via `kit_track_status` + `kit_spec_load`. |
| `/project-deploy` | `project-deploy/SKILL.md` | — |
| `/project-test` | `project-test/SKILL.md` | — |

## 4. Package layout

```text
packages/workflow-skills/
├── package.json                       (npm + pi manifest)
├── README.md
├── prompts/                            (slash command → prompt template)
│   ├── project-init.md
│   ├── project-research.md
│   ├── … (one per command)
├── skills/                             (Anthropic Agent Skills)
│   ├── project-init/
│   │   ├── SKILL.md
│   │   ├── blueprint-template.md       (bundled assets)
│   │   ├── module-spec-template.md
│   │   └── claude-module-template.md
│   ├── project-research/SKILL.md
│   ├── project-execute/
│   │   ├── SKILL.md
│   │   ├── dispatch-prompt-template.md
│   │   └── codex-report-schema.json    (referenced by codex-bridge)
│   └── … (one folder per skill)
├── extensions/                         (TypeScript Pi extensions)
│   ├── kit-engine/
│   ├── codex-bridge/
│   ├── claude-bridge/
│   └── context-mode-bridge/
└── themes/                             (optional kit theme)
    └── workflow-app.json
```

`package.json` has `"keywords": ["pi-package"]` and a `pi` key declaring extensions/skills/prompts.

## 5. Business rules

- **1:1 port from the bash kit.** Each `SKILL.md` mirrors `korallis/workflow/.claude/skills/<name>/SKILL.md` with surgical edits where Pi vs Claude Code context differs.
- **Per-project overrides.** If `<project>/.kit-workflow-app/skills/<name>/SKILL.md` exists, it takes precedence over the npm-installed version.
- **Schema files** (`codex-report-schema.json`, `parallel.yaml.schema.json`) live in this package; consumed by `codex-bridge` and `kit-engine`.
- **Carry forward all CodeRabbit-vetted invariants** from the bash kit's six review rounds (Explicit Negatives section, orchestrator-commits, slug-boundary regex, etc.).
- **British English** in user-facing prose (matches the kit's existing voice).
- **Agent Skills standard.** Each skill's frontmatter has `name`, `description`, `effort` (Claude-Code-extension; Pi ignores unknown frontmatter). Progressive disclosure preserved.

## 6. Integration points

| Module | Relationship | Notes |
|---|---|---|
| kit-engine | Calls into | All `kit_*` tools |
| codex-bridge | Calls into | `kit_codex_execute` |
| claude-bridge | Calls into | `kit_claude_execute` |
| context-mode-bridge | Indirect | Tool outputs route through transparently |
| gui-shell | Provides UI for | `/project-init` etc. surfaced as buttons too |

## 7. Acceptance criteria

- [ ] `npm install -g @korallis/workflow-skills && pi` shows all 12 commands available.
- [ ] Each slash command produces output equivalent to its bash-kit counterpart on a fixture project.
- [ ] Per-project override (`<project>/.kit-workflow-app/skills/project-init/SKILL.md`) takes precedence.
- [ ] Schema files load + validate correctly (`codex-report-schema.json`, `parallel.yaml.schema.json`).
- [ ] Package builds + publishes via `npm publish` (no errors).
- [ ] README.md documents all 12 commands.
- [ ] `pi-package` keyword + `pi` key in `package.json` are correct (verified by Pi's package loader).
- [ ] Cross-harness portability: same SKILL.md files work when symlinked into `.claude/skills/` for Claude Code.

## 8. Out of scope

- New kit commands beyond the existing 12. Future commands ship as separate skills.
- Custom kit themes for terminals (use Pi defaults).
- Internationalisation (English-only v1).

## 9. Open questions

- [ ] **Bundle templates inside the npm package** or fetch on first use? Bundle is reproducible; size is negligible (<100KB).
- [ ] **Skill auto-update on package update.** When user upgrades the npm package, do per-project overrides surface a "stale override" warning?
- [ ] **Effort frontmatter compatibility.** Pi may interpret `effort` differently than Claude Code; verify behavior, document.
