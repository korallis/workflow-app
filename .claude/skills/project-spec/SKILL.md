---
name: project-spec
description: "Create or update a detailed module specification — data model, API endpoints, UI screens, business logic, and acceptance criteria. Use when planning a new module, refining requirements before implementation, or updating specs after implementation revealed changes. Trigger on any mention of 'spec', 'specification', 'plan this module', or 'define the requirements for'."
---

# Project Spec: Generate or Update Module Specification

This skill creates or updates a complete module specification that serves as the foundation for implementation.

## When to Use

- **Planning a new module**: Start here before any code is written
- **Refining requirements**: When a module needs clearer requirements before implementation begins
- **Updating specs**: When implementation reveals gaps or changes to the original spec
- **Keyword triggers**: "spec", "specification", "plan this module", "define the requirements for", "outline the module for"

## Process

### Step 1: Read Foundation Documents

Start by understanding the overall architecture and any existing context:

- Read `specs/MASTER_BLUEPRINT.md` to understand the overall system architecture, module dependencies, and design patterns
- If updating an existing spec, read `specs/modules/$ARGUMENTS/SPEC.md` to see what already exists
- Read any related module specs (check MASTER_BLUEPRINT.md for dependency relationships) to ensure consistency
- Read `specs/LEARNINGS.md` to apply patterns and lessons from previous modules

### Step 2: Research Framework Patterns

Before designing the module, verify implementation patterns:

- Use `ref_search_documentation` to research API patterns and best practices for the chosen framework
- Use `get_code_context_exa` to find real examples of similar module implementations in the codebase
- Check `root/CLAUDE.md` for architectural guidelines and decisions already made

### Step 3: Access Templates

Reference the templates bundled with this skill:

- `../project-init/module-spec-template.md` — The standard MODULE_SPEC template with all required sections
- `../project-init/claude-module-template.md` — The implementation guide template for module-specific conventions and patterns

### Step 4: Draft the Specification

Create a comprehensive spec that covers all 10 required sections:

1. **Purpose** — What is this module, who uses it, what problem does it solve?
2. **User Stories** — 3-8 stories in table format (ID, As a..., I want to..., So that...)
3. **Data Model** — TypeScript interfaces with field types and descriptions
4. **API / Server Actions** — For each endpoint: METHOD PATH, purpose, auth, request/response schemas, errors
5. **UI Screens** — For each route: purpose, key components, user flows, empty/loading/error states
6. **Business Logic & Rules** — Validation rules, calculations, edge cases, constraints
7. **Integration Points** — Which other modules this depends on or is used by (table format)
8. **Acceptance Criteria** — Checkbox list of testable conditions for completion
9. **Out of Scope** — Explicit exclusions and non-goals
10. **Open Questions** — Unresolved decisions marked with checkboxes

### Step 5: Present for Approval

Output both the `SPEC.md` and `CLAUDE.md` files in a readable format, then ask for approval:

- Show a summary of the spec structure
- Highlight any decisions or assumptions made
- Ask: "Does this spec match your intent? Any changes before I save it?"

### Step 6: Save on Approval

Once approved:

- Save `specs/modules/$ARGUMENTS/SPEC.md` with the full MODULE_SPEC
- Save `specs/modules/$ARGUMENTS/CLAUDE.md` with the implementation guide
- Confirm both files are saved and ready for implementation

## Module Argument

`$ARGUMENTS` = the module name (e.g., "auth", "dashboard", "analytics")

Use this to locate or create the spec directory and files.

## Key Guidelines

- **Data Model**: Use TypeScript interfaces; include field types, validation rules, and comments
- **API Endpoints**: Include full JSON request/response examples; list all error codes
- **UI Screens**: Describe layouts, interactions, and state variations (empty, loading, error)
- **Business Logic**: Be explicit about rules, constraints, and edge cases
- **Integration Points**: Show clearly what this module depends on and what depends on it
- **Acceptance Criteria**: Make these testable and concrete — they will be used to verify completion

## Next Steps After Spec Approval

Once a spec is approved, use the `/project-module $ARGUMENTS` skill to begin implementation. The implementation skill will read this spec and build the module incrementally with tests and commits.

---

**Templates**: See `../project-init/module-spec-template.md` and `../project-init/claude-module-template.md` for the structure and examples.
