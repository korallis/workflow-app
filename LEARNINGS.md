# Project Learnings

This file is updated at the end of every session. It captures mistakes, discoveries, patterns that work, and patterns that don't. Reading this at the start of an implementation session prevents repeating mistakes.

> **Maintained by:** Claude Code (auto-updated via `/project-review`)
> **Format:** Newest entries at the top.

---

## Patterns That Work

### Realign stale specs before dispatching, not after

For code-maps the SPEC and CLAUDE still referenced an N-API sibling crate (`kit-code-maps-napi`) and a TypeScript caller (`kit-engine`) — leftovers from before the 2026-05-08 pure-Rust pivot. The pre-dispatch sanity check caught it; if it had reached Codex unmodified, Codex would have built ~200 lines of dead Node-bindings code that the architecture has no consumer for.

**Why:** Per-module SPECs have their own update cadence. A blueprint pivot can leave dependent SPECs lagging for hours or days. Codex follows the spec literally — it is not going to second-guess "but the blueprint says no Node."

**Where to apply:** Before every `/project-execute`, grep the module SPEC + CLAUDE for terms eliminated by recent pivots (here: `N-API`, `napi`, `kit-engine`, `workflow-skills`, `Pi extension`). Realign in a tiny `spec(<module>):` commit BEFORE assembling the dispatch prompt — the prompt embeds the SPEC verbatim, so any staleness goes straight to the executor.

### Pre-dispatch tree must be clean

For code-maps the tree had `MODULES.md` + `ROADMAP.md` dirty from the prior `/project-blueprint` realignment. Committing those first (as a separate `docs:` commit) before dispatching meant the post-Codex `git status` / `git diff --stat` was a clean signal of "what Codex did, full stop."

**Why:** The orchestrator-commits step needs to stage Codex's output by path. If unrelated files are dirty, you have to remember to exclude them, and the verification gate's `git diff` gets noisy.

**Where to apply:** Always commit (or stash) pending edits before kicking off `/project-execute`. If you forget, `git add <module-path>/` lets you scope, but the cleaner habit is to dispatch from a clean tree.

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

### Mistake: single-line JSDoc cleared before consumer line could read it (code-maps)

Codex's JS/TS extractors collected docs into a `Vec<String>` and cleared it after every non-skip line. The skip condition was `line.starts_with('*') || line.starts_with("//") || line.is_empty()`, which catches the *body* of a multi-line JSDoc (`* foo`) but NOT a single-line JSDoc `/** Foo. */` — that line starts with `/`, falls through, and clears the just-collected doc before the next line's function declaration can read it.

- **Symptom:** Two integration test failures (`javascript_fixture_matches_golden`, `typescript_fixture_matches_golden`) where the extractor produced `doc: None` but the golden file had `doc: Some("Starts the app.")`.
- **Root cause:** Skip condition didn't include `/*` prefix. `collect_js_doc` correctly captured the doc; the consuming loop immediately threw it away.
- **Fix:** Add `line.starts_with("/*")` to the skip condition in both extractors.
- **Prevention:** When a doc-comment helper is "collect into buffer / clear after each line," verify the skip-or-clear condition is the inverse of the conditions the helper recognises. Same shape, opposite role.

### Mistake: explicit-path `git add` missed Cargo.lock

When committing the code-maps scaffold, I staged `Cargo.toml` + `crates/code-maps/` explicitly. Cargo.lock got modified by `cargo build` but stayed unstaged, so the feat commit went out without the locked dependencies and a tiny chore follow-up commit had to clean it up.

- **Symptom:** Two commits where one would have done; second one is just `chore: refresh Cargo.lock`.
- **Root cause:** Habit of staging by path skipped the lockfile because it's not under the module dir.
- **Fix/prevention:** When a module adds new deps in its `Cargo.toml`, always also `git add Cargo.lock` (or just stage the lockfile in the same commit as the workspace-members edit).

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

### tree-sitter 0.25 core + mixed grammar versions (0.23–0.25)

The code-maps CLAUDE warned to "pin all together in `Cargo.toml`" because grammars must match the tree-sitter core. In practice tree-sitter 0.25 happily loaded grammars built against 0.23 (`tree-sitter-typescript 0.23`, `tree-sitter-ruby 0.23`) alongside 0.25 grammars (`tree-sitter-rust 0.24`, `tree-sitter-go 0.25`, etc.). Compiles clean, all extractors work. The pin-everything-together rule is overcautious for the 0.23–0.25 window.

**When it matters:** Adding a new grammar. Don't refuse a grammar just because it's a minor version behind the core — try compiling first.

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

### Code Maps `Class` items always have empty `methods` and `fields` in v1

SPEC §3 declares `Item::Class { methods: Vec<Signature>, fields: Vec<Signature>, ... }` but no v1 extractor populates these — every class is emitted with empty vecs. SPEC §7's "fallback for files in unsupported languages: return a Code Map with empty `items`" was Codex's licence to leave class internals empty, but that wasn't the spec's intent for *supported* languages.

**Question:** Is method/field extraction a v1.1 task (incremental enhancement) or a v1 gap that should be filled before code-maps is consumed by `skill-runner` for prompt assembly? If v1.1, document it in SPEC §9 explicitly. If v1, write the next pass before we plug into prompt builders that expect class APIs to surface methods.

### Should `learnings_fts` queries return snippet/highlight extracts?

Current `learnings_search` returns the full row + bm25 score. SQLite FTS5 can also produce `snippet()` / `highlight()` extracts for UI rendering. Defer until `gui-shell` learnings-browser is specced.

### How should the global DB at `~/.kit-workflow-app/global.db` be populated?

The per-project DB is created by `SessionStore::open(<project>/.kit-workflow-app/state.db)`. The cross-project DB at `~/.kit-workflow-app/global.db` is mentioned in SPEC §1 but no API distinguishes "local-only learning" vs "promote to global." Open question for the next module that consumes session-store (probably `kit-engine` per MASTER_BLUEPRINT §2).

### Cargo.lock policy

Committed Cargo.lock for this workspace because the blueprint plans `apps/gui-shell/` (a binary) downstream. If we end up library-only for longer than expected, revisit.
