# AI Project Kit

This directory powers spec-first, AI-assisted development for this project.

## What's Here

```
.claude/
  commands/              Slash commands — type these in Claude Code
    project-init.md      /project-init [idea]
    project-research.md  /project-research [topic]
    project-blueprint.md /project-blueprint
    project-spec.md      /project-spec [module]
    project-module.md    /project-module [name]
    project-execute.md   /project-execute [module]
    project-tracks.md    /project-tracks plan|start    (Stage 1; status/review/merge/cleanup planned)
    project-review.md    /project-review
    project-security-review.md    /project-security-review
    project-status.md    /project-status
    project-deploy.md    /project-deploy
    project-test.md      /project-test
  skills/                Skill implementations + bundled templates
    project-init/
      SKILL.md
      blueprint-template.md
      module-spec-template.md
      claude-module-template.md
    project-research/
      SKILL.md
    project-blueprint/
      SKILL.md
    project-spec/
      SKILL.md
    project-module/
      SKILL.md
    project-review/
      SKILL.md
    project-status/
      SKILL.md
    project-deploy/
      SKILL.md
    project-test/
      SKILL.md
  README.md              This file

specs/                   Generated spec documents (committed to git)
  RESEARCH.md           Domain research
  MASTER_BLUEPRINT.md   Architecture source of truth
  ROADMAP.md            Implementation order
  modules/              Per-module specs
    [module]/
      SPEC.md
      CLAUDE.md

CLAUDE.md               Root operating model (read by Claude Code automatically)
LEARNINGS.md            Accumulated project learnings
```

## Quickstart

1. Open Claude Code in this project directory
2. Type: `/project-init [your project idea]`
3. Follow the workflow — research → blueprint → specs → implement

## Adding a Module Mid-Project

```
/project-spec [module-name]    # Creates the spec
/project-module [module-name]  # Begins implementation
```

## After Each Session

```
/project-review    # Captures learnings, updates CLAUDE.md
```

## Commands Reference

| Command | Purpose |
|---------|---------|
| `/project-init [idea]` | Start a new project with full research + blueprint + specs + roadmap |
| `/project-research [topic]` | Deep research on domain, technology, regulation, or competitor |
| `/project-blueprint` | Generate or regenerate master architecture document |
| `/project-spec [module]` | Create or update a module-level specification |
| `/project-module [name]` | Implement a specific module end-to-end |
| `/project-review` | End-of-session: capture learnings, update CLAUDE.md |
| `/project-status` | Show project dashboard — what specs exist, what's built, what's next |
| `/project-deploy` | Deploy to production and verify deployment |
| `/project-test` | Run comprehensive tests — unit, type, lint, visual |
| `/project-execute [name]` | Dual-harness: Claude plans/reviews, Codex CLI implements in tmux |
| `/project-tracks plan\|start` | Parallel module implementation across isolated git worktrees (Stage 1; status/review/merge/cleanup planned) |
| `/project-security-review` | Independent security review of pending changes |
