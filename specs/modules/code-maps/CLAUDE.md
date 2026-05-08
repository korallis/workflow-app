# code-maps — Implementation guide

## Patterns

- One Rust module per language (`src/lang/rust.rs`, `src/lang/typescript.rs`, …) implementing a `LanguageExtractor` trait.
- Tree-sitter grammars added as `cargo` deps (`tree-sitter-rust`, `tree-sitter-typescript`, …).
- All extractors return the same `CodeMap` shape; trait makes language addition mechanical.
- Use `tree-sitter::Parser` per-thread (not Send across threads); pool via `thread_local!`.
- Output rendering centralised in `src/render.rs` — markdown, JSON, or compact form.

## Conventions

- Crate name: `kit-code-maps`.
- Every public type derives `Serialize + Deserialize + Clone + Debug`.
- Errors: `thiserror`-based `CodeMapError`; never panic on malformed source.
- Avoid clones on hot paths; tree-sitter `Node` is `Copy`.
- N-API exposure in a sibling crate (`kit-code-maps-napi`) so the core is napi-free for tests.

## Module boundaries

This module **owns**: tree-sitter parsing, structural extraction, language detection, token estimation, budget pruning.

This module **must NOT**: read filesystem outside the path explicitly given; do semantic analysis (type inference, callgraph); modify source files.

## Known gotchas

- **Tree-sitter version pinning.** Each grammar crate must match the `tree-sitter` core version. Pin all together in `Cargo.toml`.
- **Memory.** Parser holds source code; for huge files (>1MB), consider streaming or refusing.
- **Token estimation** is approximate. Document the ±15% disclaimer in the API.
- **TSX vs TS.** Use `tree-sitter-typescript` with both `language_typescript()` and `language_tsx()` based on extension.
- **Language detection edge cases.** `.mjs`, `.cjs`, `.d.ts` all matter; cover them.

## Test patterns

- `tests/fixtures/<lang>/sample.<ext>` + `tests/fixtures/<lang>/expected.json` for golden tests.
- Property tests for budget pruning: any `Vec<CodeMap>` with priorities should produce a budgeted output ≤ `max_tokens`.
- Performance sanity test: parse a 1000-line file and assert <100ms wall time on CI.
