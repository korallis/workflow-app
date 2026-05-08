---
name: project-init
description: "Initialise a new software project with full spec-first workflow — research, architecture, module specs, and roadmap. Use this whenever someone says 'build me a...', 'I want to create...', 'new project', 'start a project', or describes a product idea. Also use for major new feature areas within an existing project."
effort: high
---

# Project Initialization Skill

This skill guides you through a complete 7-phase spec-first project setup. Follow each phase sequentially, save outputs to the `specs/` directory, and maintain clarity throughout.

## Phase 1: Clarify the Vision

Engage with the user to deeply understand their project idea:

- **User Problem:** What pain point or opportunity does this solve?
- **Target Users:** Who are the primary users? Secondary users?
- **Success Criteria:** How will we know this project succeeded?
- **Constraints:** Budget, timeline, compliance, technical, platform constraints?
- **Out of Scope:** What explicitly will NOT be built?

Ask follow-up questions until you have a crystal-clear vision. Create a summary document: `specs/PROJECT_BRIEF.md` with sections: Overview, Problem Statement, Target Users, Success Criteria, Constraints, Out of Scope.

## Phase 2: Domain Research

Conduct structured research on the domain, market, competitors, and technical landscape. Explicitly use available research tools:

**For Market & Competitor Analysis:**
- Use `web_search_exa` with `category: "company"` to find competitor websites, products, and business models.
- Search queries like: "[product type] competitor analysis", "[industry] market landscape", "[use case] solutions".
- Document what competitors do well, weaknesses, pricing, positioning.

**For Academic & Industry Research:**
- Use `web_search_exa` with `category: "research paper"` to find peer-reviewed studies, industry reports, and technical papers relevant to your domain.
- Search queries like: "[domain] best practices", "[problem area] research", "[technology] case studies".

**For General Market Context:**
- Use `web_search_exa` (no category filter) for recent news, trends, adoption rates, and emerging patterns.

**For Technical Foundation (after tech stack decisions):**
- Use `ref_search_documentation` to find official documentation for frameworks, libraries, and APIs you're considering.
- Use `get_code_context_exa` to find production code examples and common patterns.

Record findings in `specs/RESEARCH.md` with sections: Market Landscape, Competitor Analysis, Key Trends, Regulatory Considerations, Technical Considerations, Recommendations.

## Phase 3: Module Identification

Based on the vision and research, decompose the project into logical modules. Each module should represent a cohesive business capability or feature area.

For typical web projects, consider:
- Authentication & User Management
- Core Domain Modules (e.g., Content, Products, Orders)
- Admin & Management Interfaces
- API/Backend Services
- Infrastructure & Deployment
- Observability & Analytics

Ask: "What can be built independently? What has minimal coupling? What represents distinct user workflows?"

Create a rough module list with brief descriptions and dependencies. Save as `specs/MODULES.md`.

## Phase 4: Master Blueprint

Create the comprehensive architecture blueprint by:

1. **Technology Stack Decision:** Choose your core technologies (frontend framework, backend runtime, database, infrastructure, auth, etc.). If not yet decided, answer these 4 questions:
   - What frontend framework aligns with your UI complexity and team skills? (React, Vue, Svelte, etc.)
   - What backend stack? (Node.js, Python, Go, etc. + framework)
   - What database? (Relational, document, graph, key-value?)
   - What deployment target? (Vercel, AWS, Docker, traditional servers?)

2. **Verify Latest Versions:** Use `ref_search_documentation` to check the current stable versions of chosen technologies and their recommended patterns.

3. **Find Production Examples:** Use `get_code_context_exa` with queries like "[framework] [architecture pattern] example", "[stack] production architecture" to find real-world implementations of your planned approach.

4. **Create the Blueprint:** Fill out the blueprint template with all 9 sections. Include version numbers, source URLs, and rationale for each choice.

5. **Review & Approve:** Present a summary of key decisions and architectural constraints. Ask the user for approval before saving to `specs/MASTER_BLUEPRINT.md`.

## Phase 5: Module Specifications

For each module identified in Phase 3, create a detailed module specification:

1. **Read the template:** `module-spec-template.md` (bundled here)
2. **Complete all sections:**
   - Purpose: Why does this module exist?
   - User Stories: Concrete workflows this module enables
   - Data Model: TypeScript interfaces defining the domain
   - API/Server Actions: Endpoints or functions this module exposes
   - UI Screens: Key screens and user flows
   - Business Logic & Rules: Validation, constraints, workflows
   - Integration Points: What other modules depend on this? What does this depend on?
   - Acceptance Criteria: How do we know it's complete?
   - Out of Scope: What's explicitly excluded?
   - Open Questions: Unknowns to resolve

3. **Save each spec:** `specs/modules/[module-name]/SPEC.md`

4. **Link specs together:** Ensure dependencies are clear and cross-module contracts are documented.

## Phase 6: Module CLAUDE.md Files

For each module, create a CLAUDE.md guide that helps future Claude instances understand module conventions and patterns:

1. **Read the template:** `claude-module-template.md` (bundled here)
2. **Document:**
   - Patterns to Follow: Architectural patterns specific to this module (MVC, service layer, repository pattern, etc.)
   - Conventions in This Module: Naming, file structure, error handling, logging conventions
   - Module Boundaries: What this module owns, what it reads from other modules, what it must NEVER do
   - Known Gotchas: Common mistakes, performance traps, threading issues, etc.
   - Test Patterns: Unit test structure, mock patterns, integration test approach

3. **Save:** `specs/modules/[module-name]/CLAUDE.md`

## Phase 7: Implementation Roadmap

Create a prioritized implementation plan:

1. **Identify Phase 0 (Infrastructure):** What must be built first? Database schema, auth system, API scaffolding, deployment pipeline?

2. **Sequence Modules:** Order remaining modules by:
   - User story priority (what delivers user value first?)
   - Dependency graph (what unblocks other work?)
   - Risk (build risky unknowns early)
   - Team capacity (balance parallelizable vs. sequential work)

3. **Create Sprints/Milestones:** Break into 2–4 week chunks with clear deliverables.

4. **Define Exit Criteria:** What does "done" look like for each sprint?

5. **Save:** `specs/ROADMAP.md` with sections: Phase 0 (Infrastructure), Milestones, Sprint Details, Risk Mitigation, Success Metrics.

## Output Structure

After completing all 7 phases, you will have created:

```
specs/
├── PROJECT_BRIEF.md           (Phase 1)
├── RESEARCH.md                (Phase 2)
├── MODULES.md                 (Phase 3)
├── MASTER_BLUEPRINT.md        (Phase 4)
├── ROADMAP.md                 (Phase 7)
└── modules/
    ├── [module-1]/
    │   ├── SPEC.md            (Phase 5)
    │   └── CLAUDE.md          (Phase 6)
    ├── [module-2]/
    │   ├── SPEC.md
    │   └── CLAUDE.md
    └── ...
```

All specs are markdown files stored in version control, reviewed collaboratively, and updated as the project evolves.

## Execution

Start with the user's project idea from `$ARGUMENTS`. Execute each phase in order, asking clarifying questions, saving outputs, and building a comprehensive specification before implementation begins.
