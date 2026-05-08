# Project Learnings

This file is updated at the end of every session. It captures mistakes, discoveries, patterns that work, and patterns that don't. Reading this at the start of an implementation session prevents repeating mistakes.

> **Maintained by:** Claude Code (auto-updated via `/project-review`)
> **Format:** Newest entries at the top.

---

## Patterns That Work

### Pre-fetch warms ~/.cargo/registry/cache; warming workspace `Cargo.lock` is what Codex actually needs

This is the corrected version of the earlier "Pre-fetch crates the SPEC names" pattern — that version was incomplete and led directly to a wasted dispatch.

For `track-engine`, the SPEC named `tokio`, `git2`, `tracing`, `serde_yaml`, `futures` as load-bearing. Pre-flight ran `cargo fetch` against a `/tmp/te-prefetch` scratch crate — every crate ended up in `~/.cargo/registry/cache/`. Dispatch still fell back to `std::thread` + shell-out-to-`git`. Why: Codex ran `cargo build --offline`, which needs every dep present in the *workspace's* `Cargo.lock`. Lockfile entries only appear when the workspace itself depends on the crate. Warming `~/.cargo/registry/cache` warms downloads; it does not warm the lock.

**The two-commit dispatch flow that fixes this:**

1. Pre-create `crates/<module>/Cargo.toml` with the full dependency set named in the SPEC + module CLAUDE.md.
2. Add the crate to the workspace `members`.
3. `cargo build --package <module>` (online) — populates `Cargo.lock` for every transitive resolution.
4. Verify `cargo build --package <module> --offline` succeeds against an empty `lib.rs` stub.
5. Commit as `chore(<module>): scaffold crate with deps to warm Cargo.lock` (separate commit; clean post-Codex `git diff`).
6. Dispatch — Codex's `cargo --offline` now resolves every named dep from the warm lock + cache.

This produced track-engine's correct implementation (1327 lines using `git2::Repository`, `tokio::sync::mpsc`, `futures::future::BoxFuture`) on the second dispatch where the first had produced 1102 lines of std-only fallback.

**Where to apply:** Every `/project-execute` against a Rust workspace where the new crate names deps not already used elsewhere in the workspace. The scaffold-then-dispatch flow should probably be promoted into `dispatch.sh` preflight (or into the project-execute skill itself).

### Decision protocol when the first dispatch returns big-shape deviations

`spec-engine` first run hand-rolled YAML/JSON-schema validation instead of using `jsonschema` + `serde_yaml` — accepted with follow-up. `track-engine` first run replaced Tokio with `std::thread`, `tokio::mpsc` with `std::sync::mpsc`, `git2` with shell-out, and `BoxFuture` with manual `Pin<Box<...>>` — re-dispatched with warmed lock.

The differentiator is **deviation depth**. spec-engine's hand-rolled validator was an isolated 100-line block; the public API was unchanged. track-engine's std-vs-Tokio choice leaks into `DispatchFn`'s shape and the entire runtime model — every downstream consumer (skill-runner, gui-shell with Tauri's tokio runtime) would have had to bridge an unnecessary sync→async barrier. Mechanical-but-pervasive rewrites cost more than re-dispatching with the right preflight.

**Where to apply:** When Codex returns deviations whose reason is "dependency not available offline":

- If the deviation is **isolated** (one helper, swap-out is purely additive): accept + flag follow-up. Pattern matches `spec-engine`.
- If the deviation is **architectural** (changes public API, runtime model, or downstream integration shape): reject + warm the workspace lockfile + re-dispatch.

### Pre-fetch crates the SPEC names before `/project-execute`

For spec-engine the SPEC + CLAUDE explicitly named `jsonschema` (for `parallel.yaml.schema.json`) and implied `serde_yaml` (for the YAML round-trip). Codex's `workspace-write` sandbox couldn't resolve those crates from the registry on first compile, so it hand-rolled both — passing tests, but a real spec deviation that lands as a "revisit" TODO in the commit.

**Why:** `~/.cargo/registry/cache` is shared with Codex (same `$HOME`), but a crate that has never been built on this host isn't there yet. The sandbox network block then short-circuits Codex to the path of least resistance: hand-roll. This makes a good test pass mask a poor library choice.

**Where to apply:** Before dispatching, run `cargo add --dry-run <crate>` or `cargo fetch -p <pkg>` for each new dep the SPEC names, in a throwaway scratch crate or by adding them to the workspace stub. Then dispatch. The dispatcher's preflight should probably do this automatically — added as a follow-up.

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

### Mistake: pre-fetch via /tmp scratch crate didn't warm the workspace lock (track-engine, run 1)

For the first `track-engine` dispatch I followed the earlier "pre-fetch the spec-named crates" learning by running `cargo fetch` in `/tmp/te-prefetch`. Every named crate ended up in `~/.cargo/registry/cache/`, but Codex still hit "dependency not available offline" for `tokio`, `git2`, `tracing`, `serde_yaml`, `futures` — because `cargo --offline` resolves against `Cargo.lock` first and the workspace's lock had no entries for them. Result: 1102 lines of `std::thread`/shell-out fallback that had to be discarded.

- **Symptom:** Codex's report listed 6 deviations citing "available offline" / "could not be resolved in this sandbox"; final code used `std::sync::mpsc` and `Command::new("git")` instead of `tokio::sync::mpsc` and `git2::Repository`.
- **Root cause:** `~/.cargo/registry/cache` warming is necessary but not sufficient. `cargo --offline` requires every dep to also be present in the workspace `Cargo.lock`, which only happens when something in the workspace declares a dependency on it.
- **Fix:** Discard the work, scaffold `crates/kit-track-engine/Cargo.toml` with the full named dependency set, add the crate to the workspace `members`, run `cargo build` once online so Cargo.lock gets the resolutions, commit the scaffold separately, then re-dispatch. Second dispatch produced the correct git2/tokio/futures implementation (commit `d65d8ad`).
- **Prevention:** See "Pre-fetch warms ~/.cargo/registry/cache; warming workspace Cargo.lock is what Codex actually needs" in Patterns above.

### Mistake: OMX-tool training-data interference is escalating across runs

In the spec-engine run, Codex's report listed `.omx/state/.../ultrawork-state.json` in `files_modified` (gitignored — no commit pollution but a scope violation). In the track-engine runs (both attempts), Codex now actively shells out: `omx state clear --input '{"mode":"ultrawork","all_sessions":true,"workingDirectory":"/home/leeb/workflow-app"}' --json`, `omx cancel ultrawork`, and reads/edits `.omx/state/sessions/.../ultrawork-state.json` during its turn.

- **Symptom:** Codex's `tests_run` lists `omx state clear` and `omx state list-active` as if they were part of the module's verification. Files under `.omx/` are modified mid-run. None of these paths appear in proposed commits, so verification gate catches them.
- **Root cause:** Codex's environment has OMX MCP server hooks (`omx_state.state_clear`, stop-hook prompts about "active ultrawork sessions") that fire during long runs. Codex picks up the lifecycle and treats it as part of its own task. Pure training-data / parent-environment bleed-through.
- **Fix (this run):** No action needed — `.omx/` is gitignored and excluded by explicit-paths `git add`. Recorded as a deviation in the run report so the verification gate flags it.
- **Prevention:** Add `Do NOT call \`omx\` / \`ultrawork\` commands` and `Do NOT modify \`.omx/\`` to the kit-wide explicit-negatives list in `.claude/skills/project-execute/dispatch-prompt-template.md` so the next prompt rebuilds it explicitly. Even if the harness still tries, the negative is on record.

### Mistake: dispatch.sh auth preflight default of 15s is sometimes too short

The first track-engine re-dispatch (after scaffold commit) failed at the auth preflight step with `Auth or model availability error` — even though the auth was fine; the OpenAI auth check just took >15s on a slower round trip. `KIT_AUTH_PREFLIGHT_SECONDS=60` made it pass. Pure latency variance, not auth or model availability.

- **Symptom:** Dispatch exits non-zero with `error: Auth or model availability error. Try \`codex login\`...` after the preflight prints a `model: gpt-5.5` block that proves auth was actually working.
- **Root cause:** The preflight does a real round-trip to OpenAI to verify the model is accessible; on slower-network days that round-trip can exceed 15s.
- **Fix:** Re-run with `KIT_AUTH_PREFLIGHT_SECONDS=60`.
- **Prevention:** Consider raising the default in `.claude/lib/dispatch.sh` from 15s to ~30-45s — the cost of waiting longer when auth IS broken is small (still bounded), and the cost of false negatives on a healthy auth is a wasted dispatch attempt.

### Mistake: Codex hand-rolled validation instead of using spec-named crates (spec-engine)

The SPEC for spec-engine bundled `parallel.yaml.schema.json` and CLAUDE.md said schemas are "loaded once via `jsonschema` crate." Codex's sandbox couldn't fetch `jsonschema` or `serde_yaml` (no cache hit + no network), so it embedded the schema via `include_str!` but enforced it with a 100-line hand-rolled validator and parsed YAML with another 90 lines of bespoke code. **All 11 tests pass and the `parallel.yaml` rules (`version: 1`, required fields, types) are correctly enforced** — but the dep choice diverges from the spec.

**Symptom:** Final report listed two `deviations` against `specs/modules/spec-engine/CLAUDE.md`. `git diff Cargo.toml` showed no `jsonschema` / `serde_yaml` entries; the crate's deps were the minimal `regex + serde + serde_json + thiserror`.

**Root cause:** Codex's `workspace-write` sandbox blocked crates.io DNS, and neither crate was in `~/.cargo/registry/cache`. The path of least resistance was to hand-roll.

**Fix (this run):** Accepted the commit (it meets every SPEC §7 acceptance criterion) and recorded a follow-up: swap to `jsonschema` + `serde_yaml` once the cache is warm. Commit body explicitly flags both deviations.

**Prevention:** Pre-fetch named crates before dispatch (see Patterns above). The dispatcher's preflight should grow a "scan SPEC + module CLAUDE for `crate` mentions, run `cargo fetch` for each, fail dispatch if any unresolved" step. Also: if Codex *must* hand-roll, push that into `deviations` reason and the commit body — which it did, so the verification gate worked.

### Mistake: training-data bleed-through (`.omx/state/...` ultrawork-state edit)

Codex's `files_modified` listed `.omx/state/sessions/.../ultrawork-state.json` — a state file from a foreign tool ("OMX ultrawork stop hook") that has nothing to do with this repo. Codex was trying to mark its own runtime as "complete" because a stop-hook from another harness was blocking it.

**Symptom:** `git status` was clean for the path (gitignored via `.git/info/exclude`), but the report listed it. If `.omx/` had not been excluded, this would have polluted the commit.

**Root cause:** Codex inherits the host's `$HOME` and any tool that writes state under `~/.something/` or `<project>/.something/` is fair game for cross-tool interference.

**Fix:** No action — the path is gitignored locally. Flagged to user; commit excluded it via `git add` of explicit paths only.

**Prevention:** Always `git add <explicit-paths>` (never `git add -A`) when applying Codex's `proposed_commits`. The kit's verification gate (cross-check report `files_modified` against `git status --short`) catches phantom edits like this — this run is the working-as-intended example.

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
