# Project Learnings

This file is updated at the end of every session. It captures mistakes, discoveries, patterns that work, and patterns that don't. Reading this at the start of an implementation session prevents repeating mistakes.

> **Maintained by:** Claude Code (auto-updated via `/project-review`)
> **Format:** Newest entries at the top.

---

## Patterns That Work

### Orchestrator-commits split (`/project-execute`)

Codex returns a single bundled `proposed_commits` entry; the orchestrator can and should split it where boundaries are clearer. For session-store the split was:

1. `chore` — `.gitignore` (must land FIRST or `target/` enters history when staging the workspace).
2. `fix` — kit infra change unblocking the run (schema strict-mode fix).
3. `feat` — the actual module work, with `Co-Authored-By: Codex CLI gpt-5.5 (medium)`.

**Why:** Pre-staging the gitignore before any Cargo file is the only way to keep `target/` out of git on a greenfield Rust project; bundling it with the feat commit means the very first `git add crates/` accidentally tracks build output.

**Where:** Every dual-harness scaffold of a new ecosystem (Rust workspace, Node project, Python venv) — land the ignore commit first.

### Ground-truth verification catches what schema validation can't

Codex's `--output-schema` report passed validation but contained:

- 4 `tests_run` entries referencing `omx state write …` and ultrawork session IDs from a totally different project (parent-environment bleed-through Codex picked up from its sandbox).
- A `cargo test` "failed" entry with a network-blocked excuse — but the real failure (a partial-move test bug) only surfaced when the orchestrator ran tests itself.

**Why:** Schema enforces shape, not truth. The orchestrator MUST cross-check `phases_completed` against `git diff`, run the test commands itself, and read every modified file before applying commits.

**Where:** Every `/project-execute` post-run. Don't trust report fields; verify each against repo state.

---

## Mistakes & Fixes

### Mistake: kit's report schema was OpenAI-strict-mode incompatible

`.claude/skills/project-execute/codex-report-schema.json` declared `additionalProperties: false` but listed `notes` and `body` only in `properties`, not `required`. OpenAI Responses API rejected the schema at turn 0 with `invalid_json_schema`, killing every dispatch before any work happened.

- **Symptom:** Dispatch exits, log shows `"required" is required to be supplied and to be an array including every key in properties. Missing 'notes'.`
- **Root cause:** OpenAI strict mode (which `--output-schema` uses) demands every property listed in `properties` also appear in `required` whenever `additionalProperties: false`.
- **Fix:** Add the optional properties to `required` and widen their type to `["string", "null"]` so callers can still pass null. (Commit `a3b2bfe`.)
- **Prevention:** When designing report schemas for Codex `--output-schema`, treat OpenAI strict mode as the contract — every property listed must be required-but-nullable, never bare-optional.

### Mistake: Codex test didn't compile because Codex couldn't `cargo test`

Codex generated `assert!(matches!(err, …DuplicateTrackModule { module } …))` then `assert!(err.to_string()…)` on the next line — the `matches!` macro partially moves `module` out of `err`, breaking the subsequent borrow. Wouldn't have shipped if Codex had compiled the test, but its `workspace-write` sandbox blocked dependency fetch.

- **Symptom:** `error[E0382]: borrow of partially moved value: 'err'` at `tests/session_store.rs:58`.
- **Root cause:** Codex's sandbox lacks crates.io network access AND has no populated local registry cache, so `cargo test --offline` can't resolve `chrono`, `rusqlite`, etc. Codex therefore reported "tested" only via `cargo fmt --check` and `cargo metadata`.
- **Fix:** Match against `&err` (one-character change). The orchestrator caught it on the smoke test.
- **Prevention:** Orchestrator MUST run the full test suite outside the sandbox before committing — never accept Codex's `tests_run` outcomes at face value when the report includes "could not resolve dependencies" or similar. The smoke test is the gate.

---

## Stack-Specific Notes

### Codex CLI 0.128 + ChatGPT auth + `gpt-5.5`

`codex login` (ChatGPT auth) is the path to `gpt-5.5`. `OPENAI_API_KEY` requires Tier 1+ on the OpenAI org, and the kit's preflight will surface a clear "model not available" error if the tier is missing.

### Codex sandbox + Cargo

`workspace-write` sandbox blocks the network and the `~/.cargo/registry/cache` is per-user — Codex (running as a subprocess of the orchestrator) inherits the host's `$HOME` and so the cache *is* shared. But fresh dependencies still need a download, and that's blocked. **Implication:** spec a Cargo workspace AHEAD of time so Codex only needs to add files (not new dependencies on first run), or pre-populate the cache via `cargo fetch` before dispatch.

### rusqlite 0.31 + `bundled` feature on Linux

Compiles cleanly without system SQLite; FTS5 is included. Don't switch to `bundled-sqlcipher` unless we add encryption — the deps blow up by an order of magnitude.

### FTS5 content-tied tables (`content='learnings'`)

Trigger-maintained sync via `learnings_ai/ad/au` works as documented. The "delete" trick uses `INSERT INTO learnings_fts(learnings_fts, rowid, body) VALUES('delete', …)` — the magic `'delete'` first column is FTS5's command channel, not a real INSERT. Application code never INSERTs into the FTS table directly.

### Tauri 2.x rusqlite version pinning (forward-looking)

The blueprint warns Tauri may pin rusqlite to a specific version; we picked 0.31 to match what's typical in 2.11.x's tree. Revisit when `gui-shell` lands — match Tauri's transitive pin or accept duplicate compilation of rusqlite.

---

## Open Questions

### Should `learnings_fts` queries return snippet/highlight extracts?

Current `learnings_search` returns the full row + bm25 score. SQLite FTS5 can also produce `snippet()` / `highlight()` extracts for UI rendering. Defer until `gui-shell` learnings-browser is specced.

### How should the global DB at `~/.kit-workflow-app/global.db` be populated?

The per-project DB is created by `SessionStore::open(<project>/.kit-workflow-app/state.db)`. The cross-project DB at `~/.kit-workflow-app/global.db` is mentioned in SPEC §1 but no API distinguishes "local-only learning" vs "promote to global." Open question for the next module that consumes session-store (probably `kit-engine` per MASTER_BLUEPRINT §2).

### Cargo.lock policy

Committed Cargo.lock for this workspace because the blueprint plans `apps/gui-shell/` (a binary) downstream. If we end up library-only for longer than expected, revisit.
