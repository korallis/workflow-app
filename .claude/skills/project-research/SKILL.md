---
name: project-research
description: "Conduct deep, structured research on any domain, technology, competitor, regulation, or technical topic for this project. Use whenever the team needs to understand something before making architectural or product decisions — market landscapes, compliance requirements, API capabilities, framework comparisons, or user workflows."
---

# Project Research Skill

This skill guides structured, multi-source research using available MCP tools. Research findings inform product decisions, architecture, and technical strategy.

The research topic is provided via `$ARGUMENTS`.

## Research Methodology

Follow this structured approach for all research tasks. Use the tools specified below for each research category.

### 1. Market & Domain Research

**Goal:** Understand the competitive landscape, business models, user needs, and market trends.

**Tools to Use:**
- `web_search_exa` with `category: "company"` for competitor analysis, product positioning, pricing, and business models
- `web_search_exa` with `category: "research paper"` for academic studies, industry reports, and technical papers
- `web_search_exa` (no category) for recent news, trends, adoption rates, and emerging patterns
- `web_search_exa` with `category: "people"` to find domain experts, thought leaders, and influencers

**Search Queries to Try:**
- "[Domain] market landscape 2024/2025"
- "[Product category] competitor analysis"
- "[Use case] best practices research"
- "[Industry] trends and forecasts"
- "[Topic] thought leaders experts"

**What to Document:**
- Market size and growth rates
- Key competitors and their positioning
- Common pain points and solutions
- Pricing models and revenue strategies
- User demographics and behaviors
- Emerging trends and disruptions
- Expert opinions and recommendations

**Example Workflow:**
1. Search: "[domain] market landscape" → identify top 5 competitors
2. Search: "[competitor] pricing model" → document each competitor's approach
3. Search: "[domain] research paper" → find academic validation
4. Search: "[domain] experts" → identify thought leaders to follow

---

### 2. Technical Research

**Goal:** Understand technology options, implementation patterns, APIs, and technical best practices.

**Tools to Use:**
- `ref_search_documentation` to find official documentation for frameworks, libraries, and APIs
- `ref_read_url` to read full documentation pages and deep technical guides
- `get_code_context_exa` for production code examples, GitHub repos, and Stack Overflow patterns
- `web_search_exa` (no category) for comparison articles, blog posts, and case studies

**Search Queries to Try (vary by technology type):**

**For Frameworks/Libraries:**
- "{{framework}} official documentation"
- "{{framework}} {{version}} changelog"
- "{{framework}} {{feature}} example"
- "{{framework}} performance benchmarks"
- "{{framework}} best practices 2024"

**For API Research:**
- "{{service}} API documentation"
- "{{service}} REST endpoints reference"
- "{{service}} authentication methods"
- "{{service}} rate limits and quotas"
- "{{service}} SDK examples {{language}}"

**For Architecture Patterns:**
- "{{pattern}} {{language}} example"
- "{{pattern}} production architecture"
- "{{pattern}} gotchas and pitfalls"
- "{{pattern}} vs {{alternative}} comparison"

**For Code Examples:**
- Use `get_code_context_exa` with: "{{framework}} {{pattern}} example GitHub"
- "{{stack}} production code sample"
- "{{problem}} {{language}} solution Stack Overflow"

**What to Document:**
- Latest stable versions of technologies
- Key features and capabilities
- API design patterns and best practices
- Common implementation patterns
- Performance characteristics
- Limitations and gotchas
- Migration paths and upgrading strategies
- Community size and maturity

**Example Workflow:**
1. `ref_search_documentation`: "React 19 documentation" → find official docs
2. `ref_read_url`: Read React hooks API page in full
3. `get_code_context_exa`: "React hooks custom hook example" → find implementation patterns
4. `web_search_exa`: "React performance optimization best practices" → find comparison article

---

### 3. Regulatory & Compliance Research

**Goal:** Understand legal, regulatory, and compliance requirements for the project.

**Tools to Use:**
- `web_search_exa` (no category) for regulations, compliance guides, and legal frameworks
- Always cross-reference multiple sources (minimum 2)
- Look for official regulatory body sources first

**Search Queries:**
- "{{regulation}} compliance requirements 2024"
- "{{regulation}} software requirements"
- "{{regulation}} audit and testing"
- "{{jurisdiction}} data privacy laws"
- "{{industry}} compliance checklist"
- "{{regulation}} fines and penalties"

**What to Document:**
- Applicable regulations and laws
- Compliance requirements by feature/module
- Data handling and privacy rules
- Audit and testing requirements
- Certification or licensing needs
- Penalties for non-compliance
- Implementation timeline and cost

**Verification Requirement:**
- Always cite at least 2 independent sources
- Prefer official regulatory body sources
- Note any conflicting information with explanations

**Example Workflow:**
1. Search: "GDPR software compliance requirements" → understand scope
2. Search: "GDPR data processing agreement" → find specific requirements
3. Search: "GDPR fines and penalties 2024" → understand enforcement
4. Document: "Applies if: EU users; Requirements: data consent, right to delete, DPA"

---

### 4. User Research & Workflows

**Goal:** Understand how users will interact with the product and what problems they're solving.

**Tools to Use:**
- `web_search_exa` (no category) for user reviews, feedback, and case studies
- `web_search_exa` with `category: "research paper"` for user behavior studies
- `web_search_exa` with `category: "people"` for user interviews and testimonials

**Search Queries:**
- "{{product}} user reviews feedback"
- "{{use case}} user workflows pain points"
- "{{product}} case study results"
- "{{domain}} user research study"
- "how do users {{action}}"

**What to Document:**
- Common user workflows and journeys
- Primary pain points and frustrations
- Feature usage and adoption rates
- User demographics and segments
- Success stories and case studies
- Common workarounds and hacks
- Unmet needs and feature requests

---

### 5. Competitive & Alternative Solutions Research

**Goal:** Understand what competitors are doing and what alternatives exist.

**Tools to Use:**
- `web_search_exa` with `category: "company"` for competitor websites, products, and positioning
- `get_code_context_exa` for open-source alternatives and reference implementations
- `web_search_exa` (no category) for comparison articles and reviews

**Search Queries:**
- "{{product category}} competitors 2024"
- "{{problem}} solution comparison"
- "open source {{solution}} alternatives"
- "{{product}} vs {{competitor}} comparison"
- "{{use case}} tools and solutions"

**What to Document:**
- Competitor feature matrix
- Pricing comparison
- Strengths and weaknesses of each
- Market positioning and differentiation
- Open source alternatives
- Emerging solutions and startups
- Recommended positioning for your solution

---

## Research Output Structure

Organize research findings in a consistent format. Save to `specs/research/[topic-slug].md`.

### Template

```markdown
# Research: {{TOPIC}}

> **Conducted:** {{DATE}}
> **Researcher:** {{YOUR_NAME}}
> **Status:** Complete / In Progress
> **Next Update:** {{DATE}}

## Executive Summary

{{1-2 paragraph overview of key findings}}

## Research Questions

- {{Question 1}}
- {{Question 2}}
- {{Question 3}}

## Current Landscape

### {{Subtopic 1}}
{{Findings with sources}}

### {{Subtopic 2}}
{{Findings with sources}}

## Key Findings for This Project

1. **{{Finding 1}}**
   - Impact: {{How this affects the project}}
   - Source: [Title](URL)

2. **{{Finding 2}}**
   - Impact: {{How this affects the project}}
   - Source: [Title](URL)

## Recommended Approach

### Option 1: {{Approach Name}}
- {{Advantage 1}}
- {{Advantage 2}}
- Trade-off: {{Trade-off}}
- Cost/Effort: {{Estimate}}

### Option 2: {{Approach Name}}
- {{Advantage 1}}
- {{Advantage 2}}
- Trade-off: {{Trade-off}}
- Cost/Effort: {{Estimate}}

**Recommendation:** {{Selected option}} because {{reasoning}}

## Open Questions

| Question | Priority | Status | Owner |
|----------|----------|--------|-------|
| {{Q1}} | {{Priority}} | {{Status}} | {{Owner}} |
| {{Q2}} | {{Priority}} | {{Status}} | {{Owner}} |

## Sources Consulted

### Market & Domain
- [Source Title 1](URL) — {{Type: Company Website / Blog / News}}
- [Source Title 2](URL) — {{Type: Research Paper / Industry Report}}

### Technical
- [Source Title 3](URL) — {{Type: Official Documentation / Blog}}
- [Source Title 4](URL) — {{Type: GitHub Repo / Stack Overflow}}

### Regulatory
- [Source Title 5](URL) — {{Type: Official Regulation / Compliance Guide}}
- [Source Title 6](URL) — {{Type: Legal Analysis}}

**Note:** Cross-referenced regulatory information across 2+ sources

## Conflicting Information

If sources disagree, document the conflict:
- **Source A says:** {{Claim}}
- **Source B says:** {{Conflicting claim}}
- **Resolution:** {{Our understanding}}
```

---

## Integration with Project Specifications

After completing research, integrate findings into the broader project specification:

1. **Update PROJECT_BRIEF.md** with market context and success criteria
2. **Inform MASTER_BLUEPRINT.md** technology decisions with technical research
3. **Feed module specs** (SPEC.md files) with specific requirements from domain research
4. **Create CLAUDE.md** patterns based on architectural best practices found
5. **Reference in ROADMAP.md** to inform priority and risk assessments

---

## Source Tracking & Citation

**For every finding, record:**
- URL or source identifier
- Tool used (web_search_exa, ref_search_documentation, get_code_context_exa, etc.)
- Access date
- Brief quote or summary
- Relevance to the project

**Example:**
```
- "Market grew 45% YoY" — [Gartner 2024 Report](https://gartner.com/...) (web_search_exa, category: research paper, 2025-03-19)
- "React 19 stable" — [React Official Docs](https://react.dev) (ref_search_documentation, 2025-03-19)
- "Next.js production patterns" — [GitHub: vercel/next.js examples](https://github.com/...) (get_code_context_exa, 2025-03-19)
```

---

## Quality Checklist

Before considering research complete:

- [ ] Primary research question answered
- [ ] All search queries executed (market, technical, regulatory, user, competitive)
- [ ] Minimum 3 sources per major finding
- [ ] Regulatory claims cross-referenced (2+ sources minimum)
- [ ] Conflicting information documented and resolved
- [ ] Findings include actionable recommendations
- [ ] All sources cited with URLs
- [ ] Output saved to `specs/research/[topic-slug].md`
- [ ] LEARNINGS.md updated with key insights
- [ ] Shared with team for feedback
- [ ] Open questions assigned to owners

---

## Common Research Topics

Use this skill for:

- **Market Research:** "Competitor landscape for {{product type}}"
- **Technical Feasibility:** "Can we use {{technology}} for {{requirement}}"
- **API Capabilities:** "What can {{service}} API do"
- **Compliance:** "GDPR requirements for {{feature}}"
- **User Workflows:** "How do {{users}} currently {{action}}"
- **Technology Comparison:** "{{Framework A}} vs {{Framework B}} for {{use case}}"
- **Emerging Patterns:** "Best practices for {{architecture pattern}}"
- **Performance:** "Optimization patterns for {{problem}}"
- **Security:** "Security considerations for {{feature}}"
- **Cost Analysis:** "Pricing comparison of {{services}}"

---

## Tips for Effective Research

1. **Start Broad, Narrow Down:** Research market context first, then technology options, then specific implementations
2. **Use Multiple Sources:** Don't rely on a single article; triangulate findings across sources
3. **Check Publication Dates:** Prioritize recent sources (within 1-2 years for fast-moving topics like AI/frameworks)
4. **Distinguish Opinion from Fact:** Note whether a source is an official guide, blog opinion, or academic study
5. **Document Trade-offs:** For every recommendation, clearly state what's being sacrificed
6. **Share Early:** Don't wait for "perfect" research; share findings periodically for team feedback
7. **Update Regularly:** Technology moves fast; revisit research quarterly for active projects
8. **Link to Decisions:** Explicitly connect research findings to architectural and product decisions

---

## Next Steps After Research

Once research is complete:

1. **Present Findings** to the team with 2-3 key recommendations
2. **Facilitate Discussion** on options and trade-offs
3. **Document Decision** in an Architecture Decision Record (ADR)
4. **Update Specifications** (MASTER_BLUEPRINT.md, module specs, roadmap)
5. **Monitor Progress** — If research assumptions change, repeat research for that topic
