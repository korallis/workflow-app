---
name: project-tracks
description: Plan and start isolated parallel module implementation tracks from specs/MODULES.md and per-module parallel.yaml declarations.
effort: high
---

# /project-tracks

Parallel module implementation command. Stage 1 supports:

- `/project-tracks plan [modules...] [--harness=codex|claude]`
- `/project-tracks start [modules...] [--harness=codex|claude]`

Use the helper script for deterministic parsing and side effects:

```bash
bash .claude/lib/project-tracks.sh plan "$ARGUMENTS"
bash .claude/lib/project-tracks.sh start "$ARGUMENTS"
```

## Plan

Run:

```bash
bash .claude/lib/project-tracks.sh plan [modules...] [--harness=codex|claude]
```

Requirements enforced by the helper:

- `specs/MODULES.md` must exist, otherwise print clear guidance.
- Each selected module must have `specs/modules/<module>/parallel.yaml`.
- `parallel.yaml` must declare `version: 1`.
- Selected modules must not have dependency edges between each other in `specs/MODULES.md`.
- Selected modules must not declare the same `shared:` path.
- Total selected modules must be `<= KIT_PARALLEL_MAX` (default 4).
- Missing `parallel.yaml` prints `add parallel.yaml or run sequentially`.

`plan` is read-only. It prints a proposal and does not create worktrees,
registry entries, locks, panes, logs, or branches.

## Start

Run:

```bash
bash .claude/lib/project-tracks.sh start [modules...] [--harness=codex|claude]
```

`start` reuses the same validation as `plan`, then for each selected module:

1. Creates `.claude/worktrees/track-<module>` on branch `track/<module>`.
2. Copies files listed in `.worktreeinclude` when they exist.
3. Reserves `KIT_PARALLEL_PORT_BASE + index` (default `3000 + index`).
4. Appends a running entry to `.claude/parallel/tracks.json` under a mkdir lock.
5. Launches the selected harness in the worktree.

All tracks in one invocation use the same harness. Default harness is `codex`.

**Stage 1 supports only `--harness=codex`.** `--harness=claude` is reserved for a future stage; the current launcher fails fast rather than silently bypass the per-track lock, JSON-mode capture, and schema validation that the codex path gets through `dispatch.sh`.

For Codex tracks, the helper launches `.claude/lib/dispatch.sh` with:

```bash
KIT_DISPATCH_TS=<per-track-ts>
KIT_PARALLEL_TRACK=<module>
KIT_PARALLEL_PORT=<reserved-port>
PORT=<reserved-port>
```

`KIT_PARALLEL_TRACK` already namespaces the dispatcher's lock by module, so `KIT_ALLOW_CONCURRENT=1` is **not** set — it would disable that lock and let a second launch of the same module race in the same worktree. `KIT_DISPATCH_TS` is the per-track timestamp; the dispatcher reuses it for `.log`/`.jsonl`/`-report.json` filenames so they share the `<TS>-<module>` identifier with the per-track prompt directory.
