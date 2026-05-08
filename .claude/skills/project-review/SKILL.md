---
name: project-review
description: "End-of-session review — captures what was done, learnings, mistakes, and patterns, then updates LEARNINGS.md and CLAUDE.md. Run this after completing any task, at the end of a working session, or when switching to a different area of the project. This is what makes the system compound and get smarter over time."
---

# Project Review: Capture Learnings and Update Documentation

This skill closes out work on a module or feature by documenting what was accomplished, what was learned, what went wrong and how it was fixed, and what patterns should be shared across the project.

Running this skill regularly ensures the project knowledge compounds and future work is guided by past experience.

## When to Use

- **After completing a module**: When `/project-module $ARGUMENTS` finishes
- **End of working session**: When you're done for the day or switching focus
- **After significant learning**: When you discover a pattern, mistake, or gotcha that others should know
- **Before starting new work**: To ensure you're informed by previous lessons

## Optional isolated code-review pass (`--isolate`)

If `$ARGUMENTS` contains `--isolate`, **additionally** run a code-review pass in a fresh Explore-agent context (no edit/write tools, no parent transcript) on top of the normal in-session review. The Agent reviews only the diff for code-quality / correctness issues; the rest of this skill (session learnings, LEARNINGS.md / CLAUDE.md updates, next-task recommendation) still runs in this session because that work needs parent-session memory.

When `--isolate` is detected:

1. Capture the diff:

   ```bash
   diff_file="$(mktemp -t isolated-review-diff.XXXXXX)"
   trap 'rm -f "$diff_file"' EXIT
   if git rev-parse --verify origin/main >/dev/null 2>&1; then
     base="$(git merge-base origin/main HEAD)"
   elif git rev-parse --verify main >/dev/null 2>&1; then
     base="$(git merge-base main HEAD)"
   else
     base="HEAD~1"
   fi
   git diff "$base"..HEAD > "$diff_file"
   ```

2. Launch an Explore agent (read-only):

   ```text
   Agent(
     subagent_type="Explore",
     description="Isolated diff code-review",
     prompt="You are a code reviewer with no prior context. Review the diff below for code-quality issues only — correctness, clarity, naming, error handling, test coverage, simplicity. Output a markdown report with sections: Critical / High / Medium / Low. Quote file:line for each finding. Diff: <contents of $diff_file>"
   )
   ```

3. **Then** continue with the normal in-session steps below. Append the Agent's report to the final review output, clearly labelled "Isolated code-review pass (Explore agent)".

Without `--isolate` (the default), skip the Agent step and proceed with the in-session review only.

## Process

### Step 1: Summarize What Was Done

Document the work completed in this session:

- **What was accomplished**: List completed work items, features implemented, bugs fixed
- **What was NOT done**: List planned work that didn't get completed and why
- **Decisions made**: What technical choices did you make and why? (e.g., chose Framework X over Y, refactored Architecture Z)
- **Blockers encountered**: What got stuck? What took longer than expected?
- **Outstanding issues**: Bugs found but not fixed, tech debt identified, questions left unanswered

Write this in natural language, as a narrative summary. Be specific about what was built, not just generic statements.

**Example:**
> Completed authentication module. Implemented login/logout server actions, login form UI with error handling, and refresh token rotation. Discovered that Next.js needs explicit layout.tsx file for route groups — added to LEARNINGS. Form validation initially used custom logic, but switched to Zod for consistency. One bug with token refresh timing remains in local testing; added to ROADMAP as P2.

### Step 2: Capture Learnings

Add a new section to `specs/LEARNINGS.md` at the top (newest first) documenting:

#### What Patterns Worked

Document approaches and patterns that proved effective:

- **What pattern**: Describe the pattern briefly (e.g., "Using Zod for form validation", "Server-first data fetching in Next.js")
- **Why it worked**: What makes it effective? (e.g., "Catches validation errors before submission, prevents invalid data in DB")
- **Where to use it**: When should other modules use this? (e.g., "Use for all forms that write to the database")
- **Code example** (optional): If helpful, show a 5-10 line example

**Example:**
```markdown
### Pattern: Zod + Form Validation

Zod schemas work well for validating form inputs before submission. Define the schema at the top of the component, use it in the onSubmit handler, and display field-level errors.

Works best for: Forms with multiple fields, complex validation rules, real-time field validation

Example:
const schema = z.object({
  email: z.string().email(),
  password: z.string().min(8),
});
const [errors, setErrors] = useState({});
const onSubmit = async (data) => {
  const result = schema.safeParse(data);
  if (!result.success) {
    setErrors(result.error.flatten().fieldErrors);
    return;
  }
  // Submit...
};
```

#### Mistakes & Fixes

Document errors made and how they were corrected:

- **Mistake**: What went wrong? (e.g., "Tried to share state between modules directly")
- **Symptom**: How did you discover it? (e.g., "State changed in one place didn't update another")
- **Root cause**: Why did it happen? (e.g., "Didn't respect module boundaries")
- **Fix**: What's the right approach? (e.g., "Pass data through props or use a shared store")
- **Prevention**: How to avoid this in the future (e.g., "Review CLAUDE.md before importing from other modules")

**Example:**
```markdown
### Mistake: Direct State Sharing Between Modules

Tried to import state from auth module into dashboard module. Auth state changes weren't reflected in dashboard because they're separate module instances.

Fix: Pass auth state through context or read from a centralized store. Auth module exports hooks like useCurrentUser() that other modules use.

Prevention: Review "Module Boundaries" in CLAUDE.md. If you need data from another module, use its public API (exports from index.ts).
```

#### Stack-Specific Notes

Document learnings specific to the tech stack:

- **Framework/Library**: (e.g., "Next.js", "React Hook Form", "Tailwind CSS")
- **Learning**: What did you discover about how it works? (e.g., "Next.js route groups with parentheses don't appear in URL")
- **When it matters**: When would this affect implementation? (e.g., "When organizing routes, use (group)/ for visual grouping without affecting URL structure")

**Example:**
```markdown
### Next.js: Route Groups Don't Affect URLs

Route groups like (auth)/ organize folder structure but don't appear in the URL. Useful for grouping related routes without affecting the actual URL path.

Matters when: Organizing routes logically (e.g., (auth)/login, (auth)/signup both map to /login, /signup)
```

#### Open Questions

Document questions that came up but weren't fully resolved:

- **Question**: The unanswered question
- **Context**: Where did it come from? Why is it relevant?
- **Next steps**: How should it be resolved? Who should answer it?

**Example:**
```markdown
### Should we cache authentication tokens in localStorage?

Context: Token refresh timing is tight in some scenarios. Caching in localStorage would speed up page reloads.

Concerns: Security implications if token is compromised. Browser storage is accessible to JavaScript.

Next steps: Research Next.js session management best practices. Discuss with team before implementing.
```

### Step 3: Update Root CLAUDE.md and Module CLAUDE.md

If mistakes were repeated more than once or patterns are system-wide, update the CLAUDE.md files:

**Root `CLAUDE.md`**: Update if the pattern/rule applies to all modules
- Add to "Boundaries" section if it's a cross-module rule
- Add to "Patterns" section if it's a system-wide pattern
- Add to "Common Mistakes" if it's something everyone makes

**Module `specs/modules/$ARGUMENTS/CLAUDE.md`**: Update if the pattern is specific to this module
- Add to "Known Gotchas" if it's a surprise
- Add to "Module Boundaries" if the rules changed
- Update "Conventions" if new conventions were established

**Example change to root CLAUDE.md:**
```markdown
## Boundaries

### Cross-Module Communication

- Use public exports (index.ts) to communicate between modules
- Don't import internal files (_private.ts) from other modules
- Prefer hooks (useAuth(), useUser()) over direct state access
- If you need data from another module, that module should export a public API for it
```

### Step 4: Update Specs if Implementation Deviated

If implementation revealed gaps or changes, sync the specs:

- **Update SPEC.md**: Did implementation find different requirements, additional edge cases, or omissions? Update the spec to match what was actually built.
- **Update CLAUDE.md**: Did you discover new patterns or gotchas specific to this module? Add them.
- **Document why**: If spec and code diverged, explain why in a comment or in LEARNINGS.md

Keep specs and code in sync. Specs should be the source of truth.

### Step 5: Recommend Next Task

Based on work completed and the project ROADMAP.md, identify what should be done next:

- **Unlocked dependencies**: What work became possible because of this module?
- **Blockers**: What's waiting on other work?
- **Priority**: What has the highest business value or technical importance?
- **Recommendation**: "Next session should build [module], which unblocks [other modules] and is needed for [feature]."

Write a clear "Next session should:" statement.

**Example:**
```
Next session should:
Build the dashboard module, which is now unblocked by auth completion.
Dashboard depends on auth for user context and will unlock analytics and reporting features.
This is the highest priority per the roadmap.
```

## Review Template

Here's a template to follow:

```markdown
# Review: [Module Name / Feature]

## Work Completed

- [Item 1]: [Description of what was built]
- [Item 2]: [Description of what was built]
- [Blocker]: [What didn't get done and why]

## Decisions Made

- [Decision 1]: [Why]
- [Decision 2]: [Why]

## Key Learnings

### Patterns That Worked
- [Pattern]: [Why and when to use]

### Mistakes & Fixes
- [Mistake]: [How it was fixed, how to prevent]

### Stack Notes
- [Framework]: [Learning about how it works]

### Open Questions
- [Question]: [Context and next steps]

## Spec Updates

- Updated SPEC.md: [What changed and why]
- Updated CLAUDE.md: [Patterns added]

## Next Session Should

Build the [module] module, which is now unblocked and will unlock [other modules].
Priority: [P0/P1/P2]
```

## Example Review

```markdown
# Review: Authentication Module

## Work Completed

- Implemented login/logout server actions with NextAuth
- Built login form with email/password and error handling
- Added token refresh rotation with 15-minute expiry
- Implemented useCurrentUser() hook for client components
- 92% test coverage with unit and integration tests
- Did NOT complete OAuth (Google/GitHub) — scoped to Phase 2

## Decisions Made

- Chose NextAuth over custom JWT: industry standard, handles edge cases, security-focused
- Stored tokens in httpOnly cookies: more secure than localStorage
- Refresh tokens have rolling expiry: ensures old tokens don't persist

## Key Learnings

### Pattern: useCurrentUser() Hook

Created a custom hook that fetches the current user on mount and caches it. Other modules use this hook instead of importing state directly. Works well.

### Mistake: httpOnly Cookies in Dev

Spent 2 hours debugging why cookies weren't persisting in local dev. Root cause: httpOnly not working with localhost in some browsers. Fixed by checking browser dev tools for cookie storage.

### Next.js Note

API routes in /api/ are server-side. I initially tried server-only imports in API routes; they work fine. Just remember: API routes are always server-side in Next.js 14.

### Open Question

Should we implement "Remember Me" functionality? Conflicts with security best practices. Needs team discussion.

## Spec Updates

Updated CLAUDE.md with "Token Refresh Timing" gotcha and added useCurrentUser() pattern example.

## Next Session Should

Build the dashboard module, which needs authenticated user context (now available). Dashboard is P0 and unblocks 3 dependent features.
```

## Running This Skill

This skill doesn't build or deploy anything — it documents. After running it:

1. Read the summary and learnings you've captured
2. Review the recommendations for the next session
3. Consider committing the LEARNINGS.md and CLAUDE.md updates to version control
4. Share learnings with the team if relevant

## Key Guidelines

**Be specific**: Don't write "It was hard." Write "The token refresh timing was complex because Next.js middleware timing is asynchronous and we needed to queue requests while refreshing."

**Focus on why, not just what**: "Used Zod because it catches validation errors before DB writes and prevents cascading issues" is better than "We used Zod."

**Update root CLAUDE.md sparingly**: Only add rules that apply to the whole system. Module-specific patterns go in module CLAUDE.md.

**Keep learnings evergreen**: Update LEARNINGS.md when you discover something new, but also clean up outdated learnings when patterns change.

**Link to code**: If helpful, reference file paths: "See src/modules/auth/hooks/useCurrentUser.ts for the pattern implementation."

## Why This Matters

This skill is how the system gets smarter over time. Each module's lessons compound:
- New modules don't repeat old mistakes
- System-wide patterns become consistent
- Complex gotchas are documented for future developers
- Architecture decisions are captured and explained
- The project becomes easier to navigate and contribute to

Running this at the end of every session ensures knowledge isn't lost.

---

**Reference**: See LEARNINGS.md for existing learnings. Check CLAUDE.md before implementing to benefit from previous discoveries.
