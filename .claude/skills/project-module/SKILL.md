---
name: project-module
description: "Implement a module that already has an approved spec — reads all specs, enters plan mode, then builds incrementally with tests and commits. Use when ready to write code for a specific module, or when someone says 'build the X module', 'implement X', or 'start coding X'. The module MUST have a spec before implementation begins."
---

# Project Module: Implement a Fully-Specified Module

This skill builds a complete module from an approved specification. It reads all relevant documentation, creates a detailed implementation plan, then executes incrementally with testing and version control.

## When to Use

- **Starting implementation**: When a module spec is approved and ready to be built
- **Resuming work**: When returning to an in-progress module
- **Keyword triggers**: "build the X module", "implement X", "start coding X", "build X"

## Prerequisites

The module **MUST** have:
- An approved `specs/modules/$ARGUMENTS/SPEC.md`
- An implementation guide at `specs/modules/$ARGUMENTS/CLAUDE.md`

If either is missing, stop and run `/project-spec $ARGUMENTS` first.

## Process

### Step 1: Read All Foundation Documents

Before writing any code, read these in order:

1. **`specs/MASTER_BLUEPRINT.md`** — Understand the overall architecture, technology stack, and module relationships
2. **`specs/modules/$ARGUMENTS/SPEC.md`** — Read the complete specification for this module
3. **`specs/modules/$ARGUMENTS/CLAUDE.md`** — Read the implementation guide for module-specific patterns and conventions
4. **`specs/LEARNINGS.md`** — Check for patterns, mistakes, and lessons from previous modules
5. **`root/CLAUDE.md`** — Review root-level architecture, coding standards, and system-wide conventions

If any of these files are missing or incomplete, stop and note what needs to be created or updated.

### Step 2: Create Implementation Plan

Based on the spec, create a detailed plan with the following structure:

**Frontend Implementation Guidelines** (if UI included):
- Consult the `web-artifacts-builder` skill for React/Tailwind/shadcn UI patterns
- Use `ref_search_documentation` to research component library docs and framework patterns
- Use `get_code_context_exa` to find real code examples of similar implementations
- For complex UI flows, create a standalone prototype first before integrating
- Plan UI screens and components in order of dependency

**Testing Strategy**:
- Write unit tests after each logical chunk of code
- For UI modules, use Playwright for visual verification:
  - `browser_navigate` to each route
  - `browser_snapshot` to capture accessibility tree
  - `browser_take_screenshot` for visual evidence
  - Verify element structure and interactions
  - Test forms, buttons, and user flows
  - Capture console messages via `browser_console_messages` and report errors
- Create evidence (screenshots) for each completed screen
- Run full test suite before each commit

**Implementation Order**:
- Start with data models and types
- Build API endpoints / server actions
- Implement business logic and validation
- Create UI components and screens
- Write integration tests
- Final review against spec

**Commit Strategy**:
- Format: `feat([module-name]): [what was done]`
- Commit after each substantial piece of work that passes tests
- Include test evidence in commit messages

### Step 3: Present Plan for Approval

Output your implementation plan in a clear, readable format:

- List all planned work items in logical order
- Identify dependencies and blockers
- Estimate effort for each phase
- Ask: "Does this plan align with the spec? Any changes before I start coding?"

Wait for approval before proceeding.

### Step 4: Implement Incrementally

Once approved, build the module in phases:

#### Phase 1: Data Model & Types
- Create TypeScript interfaces for all entities
- Add validation schemas (Zod, etc.)
- Create type exports from `types/index.ts`

#### Phase 2: API Endpoints / Server Actions
- Implement each endpoint from the spec
- Add request/response validation
- Handle all error cases
- Write unit tests for each endpoint
- Verify with API testing tool

#### Phase 3: Business Logic & Utilities
- Implement calculation functions
- Add state transition logic
- Create helper functions
- Write comprehensive tests

#### Phase 4: Frontend Components (if applicable)
- Start with utility components (forms, buttons, inputs)
- Build page-level components
- Implement loading and error states
- Test with Playwright:
  - Navigate to route
  - Take accessibility snapshot
  - Verify DOM structure
  - Test interactions (form submission, button clicks)
  - Take screenshots of each state
  - Check console for errors
- Verify responsive design on mobile and desktop

#### Phase 5: Integration & Full Testing
- Test end-to-end workflows
- Verify integration with other modules
- Run full test suite
- Check for console errors and warnings

### Step 5: Quality Checks Before Completion

Before marking the module as complete:

- [ ] All acceptance criteria from spec are met
- [ ] All tests pass (unit, integration, and visual)
- [ ] No console errors or warnings
- [ ] Code matches conventions in CLAUDE.md
- [ ] Code is consistent with LEARNINGS.md patterns
- [ ] Module boundaries are respected (see CLAUDE.md)
- [ ] All spec requirements are implemented (not just partially)

### Step 6: Update Specs if Implementation Deviated

If implementation revealed issues or changes:

- Update `specs/modules/$ARGUMENTS/SPEC.md` to match actual implementation
- Update `specs/modules/$ARGUMENTS/CLAUDE.md` with new patterns discovered
- Add entry to `specs/LEARNINGS.md` with the lesson
- Keep specs and code in sync

### Step 7: Run Project Review

Once complete, run the `/project-review` skill to:

- Capture what was done and what wasn't
- Document learnings and patterns
- Update root-level CLAUDE.md if patterns should be shared
- Identify blockers or issues for the next session
- Get recommendation for the next task

## Module Argument

`$ARGUMENTS` = the module name (e.g., "auth", "dashboard", "analytics")

This is used to locate the spec and implementation guide.

## Implementation Checklist

- [ ] Read MASTER_BLUEPRINT.md
- [ ] Read module SPEC.md
- [ ] Read module CLAUDE.md
- [ ] Read LEARNINGS.md
- [ ] Read root CLAUDE.md
- [ ] Create and present implementation plan
- [ ] Get approval for plan
- [ ] Implement Phase 1: Data Model & Types
- [ ] Implement Phase 2: API Endpoints / Server Actions
- [ ] Implement Phase 3: Business Logic & Utilities
- [ ] Implement Phase 4: Frontend Components (if applicable)
  - [ ] Build components incrementally
  - [ ] Test with Playwright after each screen
  - [ ] Verify responsive design
  - [ ] Check console for errors
- [ ] Implement Phase 5: Integration & Full Testing
- [ ] Verify all acceptance criteria are met
- [ ] Update specs if implementation changed them
- [ ] Run `/project-review`
- [ ] Commit final changes

## Key Guidelines

**Incrementalism**: Don't try to build everything at once. Complete one phase, test it, commit it, then move to the next phase.

**Testing as you go**: Write tests after each chunk of code. Don't defer testing to the end.

**Visual verification**: For UI modules, take screenshots and accessibility snapshots after each screen is complete. This provides evidence and catches layout issues early.

**Respecting boundaries**: Check CLAUDE.md before calling functions or reading data from other modules. Respect module ownership.

**Consistency**: Reference LEARNINGS.md and CLAUDE.md frequently. Use the same patterns and conventions as the rest of the system.

**Specification fidelity**: The spec is the source of truth. If something seems unclear, check the spec first. If implementation requires changes to the spec, update it and document why.

## Troubleshooting

**Missing spec**: Run `/project-spec $ARGUMENTS` first.

**Unclear requirements**: Check the spec's "Open Questions" section. If still unclear, ask before implementing.

**Test failures**: Use `browser_console_messages` to check for JavaScript errors. Review test output carefully.

**Integration issues**: Verify module boundaries in CLAUDE.md. Check that you're not reading or writing data you shouldn't.

**Performance issues**: Reference the "Performance Considerations" section in CLAUDE.md.

## Next Steps After Completion

Once a module is complete:

1. Run `/project-review` to capture learnings
2. Review ROADMAP.md to identify the next module
3. Check for newly unlocked dependencies
4. Start the next module with `/project-spec` if it doesn't have a spec yet

---

**Reference**: See the SPEC.md and CLAUDE.md for this module for detailed requirements and patterns.
