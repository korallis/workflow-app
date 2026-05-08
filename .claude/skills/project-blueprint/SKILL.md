---
name: project-blueprint
description: "Generate or regenerate the master architecture document (specs/MASTER_BLUEPRINT.md) — the single source of truth for tech stack, data model, API patterns, auth, UI conventions, and module relationships. Use when starting architecture, changing stack decisions, or after significant research that affects the technical approach."
effort: high
---

# Project Blueprint Skill

This skill creates or updates the Master Architecture Blueprint, the foundational architecture document for your project. Use it when:
- Starting a new project (Phase 4 of project-init)
- Making significant technology stack changes
- After completing domain or technical research that changes approach
- Quarterly architecture reviews

The skill reads existing project context, verifies technology versions, finds production examples, and generates a comprehensive blueprint.

## Pre-Blueprint Review

Before generating the blueprint, gather existing project context:

1. **Read existing specs:**
   - `specs/PROJECT_BRIEF.md` (if exists) — project vision and constraints
   - `specs/RESEARCH.md` (if exists) — technology recommendations
   - `specs/MODULES.md` (if exists) — module decomposition
   - `CLAUDE.md` (if exists) — project coding patterns
   - Any architecture decision records (ADRs)

2. **Identify decision blockers:**
   - Is the tech stack already decided?
   - Have key frameworks/databases been chosen?
   - Are there hard constraints (budget, compliance, performance)?

3. **Determine research needs:**
   - If tech stack is undefined, proceed to "Stack Detection" section below
   - If partial stack exists, identify gaps (frontend, backend, database, deployment)
   - If stack is defined, proceed directly to "Technology Verification" section

---

## Stack Detection (if not yet decided)

If the technology stack is not yet defined, answer these 4 core questions to guide choices:

### Question 1: What Frontend Framework Aligns with UI Complexity and Team Skills?

**Options to Research:**

**React**
- Use if: Complex UI state, component reusability, large ecosystem, team familiarity
- Search: "React 2024 latest version" + `ref_search_documentation`
- Considerations: Large bundle size, steep learning curve, ecosystem fragmentation

**Vue**
- Use if: Progressive enhancement, gentle learning curve, good balance
- Search: "Vue 3 official documentation" + `ref_search_documentation`
- Considerations: Smaller ecosystem than React, focused on single-file components

**Svelte**
- Use if: Minimal JavaScript, performance critical, smaller app
- Search: "Svelte documentation" + `ref_search_documentation`
- Considerations: Smaller ecosystem, less community content

**Solid or Astro (for content-heavy sites)**
- Use if: Static content dominance, island architecture, fast load times
- Search: "Astro vs Next.js comparison" + `web_search_exa`
- Considerations: Different paradigm, less suitable for complex SPAs

**Decision Criteria:**
- Team skill level and learning curve tolerance
- Application complexity and state management needs
- Performance requirements and bundle size
- Community size and available resources
- Deployment target (Vercel, traditional server, edge)

### Question 2: What Backend Stack? (Language + Framework + Runtime)

**JavaScript/Node.js Options:**
- **Next.js** (React meta-framework): Full-stack React, API routes, edge functions
  - Search: "Next.js 15 latest version" + `ref_search_documentation`
- **Express.js**: Minimal, flexible, requires more setup
  - Search: "Express.js best practices 2024" + `web_search_exa`
- **Fastify**: High-performance, modern, TypeScript-first
  - Search: "Fastify documentation" + `ref_search_documentation`

**Python Options:**
- **Django**: Feature-rich, batteries-included, ORM, admin panel
  - Search: "Django 5.0 official documentation" + `ref_search_documentation`
- **FastAPI**: Modern, async, automatic API docs, very fast
  - Search: "FastAPI documentation" + `ref_search_documentation`

**Other Runtimes:**
- **Go (Echo/Gin)**: High performance, compiled, good for microservices
  - Search: "Go web framework comparison" + `web_search_exa`
- **Rust (Actix/Axum)**: Safety and performance, steep learning curve
  - Search: "Rust web framework 2024" + `web_search_exa`

**Decision Criteria:**
- Team language expertise
- Performance requirements
- Development speed vs. production optimization
- Available libraries and frameworks
- Hosting and deployment options

### Question 3: What Database? (Primary Data Store)

**Relational (SQL):**
- **PostgreSQL**: Mature, feature-rich, free, excellent JSON support
  - Search: "PostgreSQL latest version documentation" + `ref_search_documentation`
- **MySQL/MariaDB**: Widely hosted, simpler than PostgreSQL
- **SQL Server**: Enterprise, Windows-centric

**Document (NoSQL):**
- **MongoDB**: Flexible schema, scales horizontally, document-oriented
  - Search: "MongoDB Atlas documentation" + `ref_search_documentation`

**Graph:**
- **Neo4j**: Relationship-heavy data, social networks, recommendations
  - Search: "Neo4j documentation" + `ref_search_documentation`

**Vector (for AI/ML):**
- **Pinecone/Weaviate**: Vector embeddings, AI search
  - Search: "Pinecone vs Weaviate comparison" + `web_search_exa`

**Cache/Sessions:**
- **Redis**: In-memory, fast, sessions, caching, real-time features
  - Search: "Redis latest version" + `ref_search_documentation`

**Decision Criteria:**
- Data structure: Highly relational → SQL; flexible → MongoDB; relationships → Graph
- Scale: Billions of records → consider distributed options
- Consistency: ACID requirements → SQL; eventual consistency ok → NoSQL
- Query patterns: Complex joins → SQL; simple lookups → NoSQL

### Question 4: What Deployment Target? (Infrastructure)

**Serverless/Edge:**
- **Vercel**: Next.js native, edge functions, preview deployments
  - Search: "Vercel documentation" + `ref_search_documentation`
- **AWS Lambda**: Scalable, pay-per-use, complex to manage
- **Cloudflare Workers**: Edge execution, global distribution

**Container-Based:**
- **Docker + Kubernetes**: Full control, operational complexity
- **Docker + Docker Compose**: Simpler for small teams

**Traditional Servers:**
- **AWS EC2, DigitalOcean, Linode**: Full control, manage updates
- **Heroku**: PaaS, simpler but less control, higher cost

**Decision Criteria:**
- Operational complexity tolerance
- Scaling needs: Sudden spikes → serverless; steady → traditional servers
- Cost model: Sporadic usage → serverless; always-on → traditional
- Control requirements: Full → traditional/k8s; happy with managed → serverless/Vercel
- Team DevOps experience

---

## Technology Verification

Once stack options are identified or decided, verify current versions and best practices:

### For Each Chosen Technology:

1. **Find Official Documentation:**
   - Use `ref_search_documentation` with query: "{{framework}} official documentation"
   - Record the current stable version (not beta, not EOL)
   - Note any deprecations or major API changes

2. **Verify Production Patterns:**
   - Use `get_code_context_exa` with query: "{{framework}} {{pattern}} production example"
   - Look for: Real-world repos, large projects, proven patterns
   - Record: URL to example repo, key takeaways

3. **Check Version Compatibility:**
   - Frontend + Backend: Are they compatible? (e.g., React 19 + Next.js 15)
   - Database drivers: Are they updated for the database version?
   - Build tools: Updated recently? Any major breaking changes?

4. **Document Rationale:**
   - For each technology, record:
     - Version number
     - Why chosen (from stack detection Q1-Q4)
     - Trade-offs vs. alternatives
     - Source URL

---

## Master Blueprint Generation

### Step 1: Gather Information

Collect the following before filling the blueprint:

**From Project Brief:**
- Problem statement and solution overview
- Target users and success criteria
- Scope boundaries and constraints

**From Research:**
- Market landscape context
- Competitor positioning
- Technical recommendations
- Regulatory/compliance requirements

**From Stack Detection:**
- Frontend framework choice and rationale
- Backend language, framework, runtime
- Primary database choice
- Deployment target
- Secondary technologies (cache, auth, CDN, etc.)

**From Tech Verification:**
- Current stable versions for each technology
- Documentation URLs
- Production example repos
- Known gotchas or migration paths

### Step 2: Fill Blueprint Template

Use the template at `../project-init/blueprint-template.md`.

Fill each section systematically:

1. **1. Project Overview:** Copy from PROJECT_BRIEF.md
2. **2. Tech Stack:** List technologies with versions from stack detection + verification
3. **3. Data Model:** Define core entities as TypeScript interfaces
4. **4. API Design Patterns:** Choose REST/GraphQL/tRPC; define error format, auth method
5. **5. Shared UI Patterns:** Design system basics, navigation pattern, accessibility level
6. **6. Modules:** List from MODULES.md with dependencies
7. **7. Infrastructure & Deployment:** Hosting, CI/CD, monitoring, backups
8. **8. Security & Compliance:** Auth method, encryption, compliance requirements
9. **9. Open Questions:** Known unknowns to resolve before implementation

### Step 3: Create Example Data Model

Define the core entities that represent your domain:

**Example: E-commerce**
```typescript
interface User {
  id: string;
  email: string;
  passwordHash: string;
  role: "customer" | "admin";
  createdAt: Date;
  updatedAt: Date;
}

interface Product {
  id: string;
  name: string;
  description: string;
  price: number;
  inventory: number;
  categoryId: string;
  createdAt: Date;
  updatedAt: Date;
}

interface Order {
  id: string;
  userId: string;
  items: OrderItem[];
  status: "pending" | "processing" | "shipped" | "delivered";
  totalPrice: number;
  createdAt: Date;
  updatedAt: Date;
}
```

---

## Blueprint Review & Approval

### Prepare Summary for Team Review

Create a 1-page summary covering:

**Key Technology Decisions:**
- Frontend: {{Framework}} v{{Version}} — chosen because {{Rationale}}
- Backend: {{Language/Framework}} v{{Version}} — chosen because {{Rationale}}
- Database: {{Technology}} — chosen because {{Rationale}}
- Deployment: {{Platform}} — chosen because {{Rationale}}

**Architectural Highlights:**
- Data Model: {{Key entities and relationships}}
- API Style: {{REST/GraphQL/etc}} — {{Key design decision}}
- Authentication: {{Method}} — {{Implementation approach}}

**Known Constraints & Trade-offs:**
- {{Constraint 1}} — Impact: {{Impact}}
- {{Constraint 2}} — Impact: {{Impact}}

**Open Questions Requiring Decision:**
1. {{Question 1}} — Decision needed by {{Date}}
2. {{Question 2}} — Decision needed by {{Date}}

### Present & Discuss

1. **Share Summary** with the team
2. **Walk through** the 9 sections of the blueprint
3. **Discuss trade-offs** — for each major decision, explain alternatives and why this won
4. **Identify Risks** — what could go wrong with this architecture?
5. **Gather Feedback** — are there concerns? Missing considerations?

### Approval Gates

Before finalizing, confirm:

- [ ] Tech stack agreed by team (frontend, backend, database, deployment)
- [ ] Data model reviewed and no structural issues identified
- [ ] API design patterns clear and feasible
- [ ] Deployment strategy supported by team DevOps capability
- [ ] Compliance/security requirements are met
- [ ] Open questions have owners and target resolution dates
- [ ] Team confident in the approach

---

## Save & Version

Once approved:

1. **Save to:** `specs/MASTER_BLUEPRINT.md`
2. **Version Control:** Commit with message: "feat: create master blueprint for {{project}}"
3. **Reference Specs:** Link from PROJECT_BRIEF.md and module specs
4. **Update LEARNINGS.md:** Document key architectural decisions and rationale

---

## Common Blueprint Patterns

### Pattern: Monolithic + API

**When to Use:** Startup MVP, single team, simple domain
```
Frontend (React) → Backend (Next.js) → Database (PostgreSQL)
                ↓
              Cache (Redis)
```

### Pattern: Full-Stack Frameworks

**When to Use:** Web apps with tight frontend-backend coupling
```
Next.js (frontend + backend) → Postgres + Redis
                              ↓
                         Vercel (deploy)
```

### Pattern: Microservices

**When to Use:** Scale, multiple teams, independent deployments
```
Frontend (React) → API Gateway → Service 1 (Node.js) → DB1
                              → Service 2 (Python) → DB2
                              → Service 3 (Go)     → DB3
```

### Pattern: Async Workers

**When to Use:** Long-running jobs, background processing
```
Frontend → Backend API → Queue (Redis/Bull) → Workers
                      ↓
                   Database
```

Choose the pattern that matches your scale, team size, and complexity.

---

## Handling Changes to the Blueprint

If circumstances change during development:

1. **Minor Changes** (new field in data model): Update blueprint, create ADR if significant
2. **Moderate Changes** (database technology swap): Re-run tech verification, update blueprint, get team approval
3. **Major Changes** (backend framework swap): Repeat full blueprint process with research

Always maintain the blueprint as current source of truth.

---

## Template & Resources

- **Template:** `../project-init/blueprint-template.md`
- **Example Blueprints:** Reference blueprints in `docs/examples/`
- **Technology Docs:** Use `ref_search_documentation` to find official guides
- **Production Examples:** Use `get_code_context_exa` for real-world implementations

---

## Next Steps

After blueprint approval:

1. **Create Module Specs** — Each module gets detailed SPEC.md using `../project-init/module-spec-template.md`
2. **Create Module CLAUDE Guides** — Each module gets CLAUDE.md for implementation patterns
3. **Create Implementation Roadmap** — Priority and sequencing of work
4. **Set Up Infrastructure** — Deploy databases, auth system, CI/CD
5. **Begin Implementation** — Modules in priority order
