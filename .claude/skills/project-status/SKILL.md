---
name: project-status
description: "Show a complete project status dashboard — what specs exist, what's been implemented, what's next, and overall health. Use when someone asks 'where are we', 'what's the status', 'what should I work on next', or at the start of any new session to orient yourself."
---

# Project Status Dashboard

I'll generate a comprehensive status report of your AI Project Kit, checking spec completeness, code implementation, learnings, and recommended next steps.

## Steps

### 1. Read Project State

First, I'll check the existence and status of all specification and planning documents:

- **specs/RESEARCH.md** — research and discovery phase
- **specs/MASTER_BLUEPRINT.md** — system architecture and technical decisions
- **specs/ROADMAP.md** — planned milestones and work items
- **specs/modules/*/SPEC.md** — individual module specifications

For each, I'll note:
- Whether it exists
- Key summary (scope, status, key decisions)
- Any blockers or open questions

### 2. Check Implementation Status

I'll examine the source code directories to determine which modules have:
- **Spec only** — design document exists, no code yet
- **In Progress** — both spec and partial code
- **Complete** — spec and production code
- **Not started** — no spec, no code

### 3. Review Learnings

I'll read LEARNINGS.md (if it exists) to:
- Count total learning entries
- Identify open questions still needing answers
- Note any recurring patterns or blockers

### 4. Generate Status Dashboard

I'll produce a clean, actionable status table with:
- Research, Blueprint, Roadmap completion status
- Module-by-module status (spec + code)
- Learning entry count and open questions
- A recommended next task based on project state

### 5. Recommend Next Steps

Using the project state, I'll suggest:
- Which module to work on next (using `/project-module`)
- Whether to focus on specs, code, tests, or deployment
- Any pre-requisite work needed

## Output Format

```
PROJECT STATUS — [date]
================================

SPECS:
  Research:     ✅ Complete / ❌ Missing / 🔄 In progress
  Blueprint:    ✅ Complete / ❌ Missing / 🔄 In progress
  Roadmap:      ✅ Complete / ❌ Missing / 🔄 In progress

MODULES:
  module-name   ✅ Spec  ✅ Code  |  Summary of status
  ...

LEARNINGS: [X] entries ([Y] open questions)

HEALTH CHECK:
  Tests:        ✅ Passing / ⚠️ Warnings / ❌ Failing
  Deploy:       ✅ Ready / 🔄 In progress / ❌ Blocked
  Quality:      ✅ Good / ⚠️ Needs work

NEXT RECOMMENDED TASK:
  For module focus: /project-module module-name
  For testing:      /project-test
  For deployment:   /project-deploy
  For deep dive:    /project-research
```

## Key Commands

Once you have status, use these related commands:
- `/project-module [name]` — deep dive into a specific module
- `/project-test` — run comprehensive test pass
- `/project-deploy` — deploy to preview or production
- `/project-research [question]` — research and document findings
