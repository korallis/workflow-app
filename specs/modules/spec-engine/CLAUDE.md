# spec-engine — Implementation guide

## Implementation status (2026-05-08)

The crate is implemented at `crates/kit-spec-engine/` and meets every SPEC §7 acceptance criterion (11 tests passing). Two known deviations from this guide, both filed as follow-ups:

- **`parallel.yaml` validation is hand-rolled, not via the `jsonschema` crate.** The schema is bundled via `include_str!` and the rules (`version: 1`, required fields, types, allowed keys) are enforced by local code. Functionally equivalent; swap to `jsonschema` once `~/.cargo/registry/cache` is warm.
- **YAML round-trip is hand-rolled, not via `serde_yaml`.** Lossless for the `ParallelYaml` shape we accept; swap to `serde_yaml` for robustness on edge cases.

Root cause: Codex's `workspace-write` sandbox couldn't fetch new crates on first compile. See LEARNINGS.md → "Pre-fetch crates the SPEC names before `/project-execute`".

## Patterns

- **`Arc<SessionStore>`** shared state; clone freely.
- **`PathBuf`-typed project root** — never accept arbitrary strings.
- **Markdown stays markdown.** No structural parsing beyond validating frontmatter and the optional `parallel.yaml`.
- **Schemas** live under `crates/kit-spec-engine/schemas/` as JSON Schema files; loaded once via `jsonschema` crate.
- **Errors:** `thiserror`-based `SpecEngineError`; wraps `io::Error`, `serde_yaml::Error`, validation errors.

## Conventions

- Crate name: `kit-spec-engine`.
- Module slugs are validated by a single regex constant in `slug.rs`.
- File names: `PROJECT_BRIEF.md`, `RESEARCH.md`, `MASTER_BLUEPRINT.md`, `MODULES.md`, `ROADMAP.md` at top-level; per-module under `specs/modules/<slug>/{SPEC,CLAUDE}.md` + optional `parallel.yaml`.
- Atomic writes: write to `<path>.tmp`, fsync, rename.

## Module boundaries

This module **owns**: spec file CRUD, schema validation, snapshotting.

This module **must NOT**: spawn agents, modify git, manage worktrees.

## Known gotchas

- **YAML version field.** `version: 1` (number, not string) — serde_yaml handles both but be strict on output.
- **Mtime races.** Two saves in the same second can produce same `taken_at` in snapshots; dedupe by content hash, not timestamp.
- **Schema file embedding.** Use `include_str!` so schemas ship in the binary; don't read from disk at runtime.
- **Module-name validation BEFORE path joining.** Otherwise `../../etc/passwd` becomes a real path.

## Test patterns

- Use `tempfile::TempDir` for FS tests; never touch a real project.
- Property tests for slug regex via `proptest`.
- Round-trip tests for every spec type.
- Atomic save rollback test: simulate a write failure mid-batch, assert no files exist.
