---
name: project-execute
description: Implement a fully-specced module via dual-harness execution — Claude orchestrates, Codex CLI executes inside a live tmux pane in your attached session. Use when ready to write code for a module that has approved SPEC.md and CLAUDE.md files. Triggers on phrases like 'execute the X module via codex', 'dual-harness build', or '/project-execute X'.
effort: high
---

# project-execute

Hands a fully-specced module to Codex CLI (gpt-5.5) for implementation while Claude Code retains plan/review responsibilities. Live progress streams into a tmux pane that splits into your most-recent attached session.

## Prerequisites (abort if missing)

1. **`$ARGUMENTS` validation**: must match `^[a-z0-9][a-z0-9-]*$` (kebab-case, no slashes, no `..`). Abort if missing or invalid: ask the user "Which module? Existing modules: <list of `specs/modules/*/`>". If invalid, refuse and explain.
2. Read `CLAUDE.md` (root) — operating rules.
3. Read `specs/MASTER_BLUEPRINT.md` — system architecture. **If missing**: stop and tell the user "No `specs/MASTER_BLUEPRINT.md` found. Run `/project-blueprint` first to establish the architecture, then re-run `/project-execute`."
4. Read `specs/modules/$ARGUMENTS/SPEC.md` — module spec. If missing: stop and tell the user "No spec at `specs/modules/$ARGUMENTS/SPEC.md`. Run `/project-spec $ARGUMENTS` first."
5. Read `specs/modules/$ARGUMENTS/CLAUDE.md` — module conventions. If missing: same as above.
6. Read `LEARNINGS.md` (optional) — accumulated patterns.

## Build the dispatch prompt

**First, compute one timestamp token and reuse it for every artefact in this run** (so the prompt file, log file, and last-message file all share the same `<TS>`):

```bash
TS="$(date +%Y%m%d-%H%M%S)-$$"
```

Then concatenate the following into a single prompt file at `.kit-orchestration/exec-$ARGUMENTS-$TS-prompt.md`:

1. A header: model + effort + module name + timestamp.
2. The full text of `CLAUDE.md` (root).
3. The full text of `specs/MASTER_BLUEPRINT.md`.
4. The full text of the module's `SPEC.md`.
5. The full text of the module's `CLAUDE.md`.
6. An explicit instruction block:

   ```text
   You are the executor in a dual-harness orchestration. Implement the module
   above end-to-end. Constraints:

   - Build phase by phase. After each phase, run any tests you can.
   - Do NOT commit. Leave the working tree dirty for the orchestrator to
     inspect, smoke-test, and split into commits. Record proposed commit
     boundaries in the `proposed_commits` field of your final report.
   - Do NOT `git push`. Do NOT touch .git/ directly. Do NOT modify CI
     configuration (.github/, etc.).
   - Do NOT modify files outside the module's directory unless the spec
     requires shared edits. List any such edits in `files_modified`.
   - Re-read any "Explicit negatives" section in the prompt before acting;
     spec body negatives ("don't add X", "trust Y only") are repeated there
     for emphasis and must be honoured.
   - Stop and ask if any spec instruction is ambiguous; record questions
     in the `open_questions` field of your final report.
   - Your final agent message must be a JSON object conforming to the schema
     at .claude/skills/project-execute/codex-report-schema.json (the dispatcher
     enforces this via --output-schema).
   ```

7. The full prompt template at `.claude/skills/project-execute/dispatch-prompt-template.md` provides the canonical assembly order; follow it. **Always include an "Explicit Negatives" section** listing kit-wide and spec-derived prohibitions so they're reiterated in the constraints block, not buried in the spec body.

## Why Codex doesn't commit

Codex's `workspace-write` sandbox blocks `.git/index.lock`, so commits would fail anyway. More importantly, **orchestrator-commits is the canonical pattern by design**:

- Claude reads the scrubbed log + structured report, smoke-tests the working tree, then splits commits at sensible boundaries.
- Co-Authored-By attribution stays on every commit.
- The verification gate catches Codex spec deviations before they land in history.
- No need for `KIT_CODEX_SANDBOX=danger-full-access` (which is what it sounds like).

If you genuinely need Codex to commit (e.g. multi-turn runs that depend on intermediate state), set `KIT_CODEX_SANDBOX=danger-full-access` per-run — but the default flow does not.

## Dispatch via dispatch.sh

Run, as a single Bash tool call (reusing the same `$TS` from the prompt-build step). **Export `KIT_DISPATCH_TS=$TS` first** so the dispatcher reuses your token instead of forking a fresh one — without this, the advertised `.jsonl`/`-report.json`/`.log` paths drift from what you computed and downstream `Read` calls fail:

```bash
KIT_DISPATCH_TS="$TS" bash .claude/lib/dispatch.sh execute "$ARGUMENTS" gpt-5.5 medium \
  ".kit-orchestration/exec-$ARGUMENTS-$TS-prompt.md"
```

The dispatcher handles tmux split (into the most-recent attached session), log capture, lock acquisition, auth/model preflight, timeout enforcement, and exit-code propagation. **Note**: dispatch.sh writes raw logs — secret scrubbing happens later, when this skill reads the log back. If `dispatch.sh` exits non-zero, surface the error and stop.

## After Codex returns

Use the same `$TS` from the prompt-build step. **All of `-report.json`, `-last.md`, and `.log` must be piped through `scrub-secrets.sh` before re-entering Claude's context** — even the schema-validated report can contain credentials Codex echoed during the run.

1. Read the scrubbed structured report (preferred): `bash .claude/lib/scrub-secrets.sh .kit-orchestration/execute-$ARGUMENTS-$TS-report.json`. This is JSON conforming to `codex-report-schema.json` — `phases_completed`, `files_modified`, `files_created`, `tests_run`, `proposed_commits`, `deviations`, `open_questions`. Trust the structure, but verify each field against repo state.
2. **Cross-check the report against ground truth.** Run `git status --short`, `git diff --stat`, and the test commands listed in `tests_run`. If `phases_completed` references work that doesn't appear in the diff, treat it as bleed-through and flag to the user. Schema enforcement gives shape; ground truth gives correctness.
3. Read the scrubbed JSONL events for context if needed: `bash .claude/lib/scrub-secrets.sh .kit-orchestration/execute-$ARGUMENTS-$TS.jsonl | jq -c .`. Or the scrubbed pretty log: `bash .claude/lib/scrub-secrets.sh .kit-orchestration/execute-$ARGUMENTS-$TS.log | tail -200`. Never read raw `.log`/`.jsonl`/`-report.json` directly — every artefact must pass through `scrub-secrets.sh`.
4. **Apply `proposed_commits` from the report** (orchestrator-commits pattern). Stage files, write commits with the proposed subjects + Conventional Commit prefixes + `Co-Authored-By: Codex CLI gpt-5.5 (medium)`. If you disagree with the boundaries, split or merge as you see fit before committing.
5. Summarise to the user: phases completed (verified), commits applied, tests run, deviations from spec, open questions.
6. **Run the review skill in this session**: read `.claude/skills/project-review/SKILL.md` and follow its instructions to update `LEARNINGS.md`. (Skills cannot literally invoke other skills as user actions; this is the Claude-reads-and-follows pattern.) Alternatively, if the user prefers an isolated review, suggest they run `/project-review --isolate` (added in PR3).
7. Do NOT push the branch — that's the user's call.

Note: the `<TS>` placeholders in any spec or log examples elsewhere in the kit refer to the same `$TS` value computed once per `/project-execute` run.
