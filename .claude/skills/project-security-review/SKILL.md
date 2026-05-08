---
name: project-security-review
description: Independent security review of pending changes via an isolated Agent context — fresh subagent reads the diff and a security checklist with no implementation bias. Use after a module is implemented and before merge, especially for changes that touch auth, data persistence, PII, audit logging, or anything UK GDPR-sensitive. Triggers on '/project-security-review', 'security review', 'audit this branch'.
effort: high
---

# project-security-review

Runs a security review of pending changes in **fresh Agent context** so the reviewer has no exposure to the implementation reasoning that produced the code. This isolation is the entire point — a reviewer who watched the code being written tends to share its blind spots.

## Process

1. **Capture the diff** (use `mktemp` + cleanup trap so the temp file doesn't leak):

   ```bash
   diff_file="$(mktemp -t security-review-diff.XXXXXX)"
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

2. **Identify the security checklist**: read `.claude/skills/project-security-review/security-review-prompt.md`. This is the canonical instruction set the Agent will follow.

3. **Launch the Agent** with a read-only subagent type so the reviewer can't accidentally modify the working tree:

   ```text
   Agent(
     subagent_type="Explore",
     description="Security review (isolated read-only context)",
     prompt=<contents of security-review-prompt.md, with $diff_file content and any relevant CLAUDE.md security section interpolated into the prompt body>
   )
   ```

   The Explore agent runs with a fresh context window AND no edit/write tools — it has not seen planning or implementation discussion, and it cannot modify code. Its findings reflect the diff alone plus the security checklist.

4. **Receive the Agent's report** and surface it to the user without modification. Add a one-paragraph framing explaining what was reviewed and any user-actionable next steps.

5. **Optional follow-ups**:
   - If the Agent found CRITICAL or HIGH issues, suggest the user address them before merge.
   - Note that this skill does NOT modify code — it produces a report only.

## When to use

- Before merging any PR that touches auth, session management, or token handling.
- After implementing data persistence for PII or healthcare-sensitive data (UK GDPR considerations).
- When the change touches audit logging, access control, or rate limiting.
- When you want a "second set of eyes" with no contextual bias.

## Why isolation matters

A reviewer who participated in implementation tends to validate the assumptions that drove the implementation. An Agent in fresh context only sees the diff — its blind spots are different from the implementer's, which is exactly what a security review needs.
