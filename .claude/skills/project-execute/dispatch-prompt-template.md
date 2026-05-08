# project-execute Dispatch Prompt Template

Use this template when assembling `.kit-orchestration/exec-<module>-<timestamp>-prompt.md`.
Keep the section order stable so Codex receives project-wide rules before module-specific instructions.

---

## 1. Dispatch Header

<!-- placeholder: model, reasoning effort, module name, timestamp, repository root -->

---

## 2. Root Operating Rules

Source: `CLAUDE.md`

<!-- placeholder: full root CLAUDE.md content -->

---

## 3. Master Blueprint

Source: `specs/MASTER_BLUEPRINT.md`

<!-- placeholder: full master blueprint content -->

---

## 4. Module Specification

Source: `specs/modules/<module>/SPEC.md`

<!-- placeholder: full module SPEC.md content -->

---

## 5. Module Conventions

Source: `specs/modules/<module>/CLAUDE.md`

<!-- placeholder: full module CLAUDE.md content -->

---

## 6. Executor Instruction Block

<!-- placeholder: explicit dual-harness implementation constraints from SKILL.md -->

---

## 6.5. Explicit Negatives

This section reiterates "do NOT" requirements that appeared in §4 (Module Spec) or §5 (Module Conventions) so they get the same emphasis as the constraints block in §6. Spec deviations cluster around requirements that live only in spec body prose — repeat them here.

Required entries every run (kit-wide negatives):

- Do NOT commit. The orchestrator commits after smoke-testing. Record proposed commits in your final report.
- Do NOT `git push` or open PRs.
- Do NOT modify `.git/` directly. Do NOT modify CI config (`.github/`, etc.).
- Do NOT modify `.claude/lib/dispatch.sh` while a dispatcher is running unless the spec specifically requires it (the dispatcher self-relocates to a temp copy, but live edits still confuse downstream tooling).
- Do NOT modify `.claude/hooks/pre-compact.sh` or `.claude/lib/scrub-secrets.sh` unless explicitly in scope.

Spec-derived entries (extracted from §4/§5 of THIS prompt's spec):

<!-- placeholder: bullet list of "no X", "trust Y only", "don't add Z heuristic" requirements lifted verbatim from the spec body. The skill author is responsible for populating this when assembling the prompt; an empty list is acceptable but suspicious. -->

---

## 7. Final Report Schema

Source: `.claude/skills/project-execute/codex-report-schema.json`

Your final agent message MUST be a JSON object conforming to this schema. The dispatcher enforces this via `codex exec --output-schema`. Free-form claims of completion ("X is now done", "ready to ship") will not be accepted in any field — the orchestrator verifies every field against repository ground truth (`git status`, test output, file presence) before committing or summarising to the user.

<!-- placeholder: the schema is loaded from disk by the dispatcher; do NOT inline it here, just reference the path. -->
