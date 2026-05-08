# code-maps — SPEC

> **Status:** Phase 5 of /project-init
> **Layer:** Engine (Rust crate)
> **Last updated:** 2026-05-08

## 1. Purpose

Generate **token-efficient structural summaries** ("Code Maps") of source files using tree-sitter. Repo-Prompt-style: instead of feeding 7M tokens of full source into a prompt, feed an 85K-token summary that exposes class/function signatures, imports, exports, and module relationships. Called directly from `skill-runner` (Rust crate-to-crate) when assembling agent prompts and exposed to the GUI via Tauri commands for the file picker.

## 2. User stories

| ID | As a... | I want to... | So that... |
|---|---|---|---|
| CM-1 | `skill-runner` assembling a prompt | Get a Code Map for `src/auth.ts` | I send signatures, not 800 lines of body |
| CM-2 | GUI file picker | Show the symbol outline of every selected file | The user sees what's in scope at a glance |
| CM-3 | `/project-execute` skill | Get a token-budgeted Code Map for the whole repo | The dispatch prompt fits in the model's context |
| CM-4 | `/project-spec` skill | Generate a dependency map of `src/modules/*` | The spec template auto-fills "Integration Points" |
| CM-5 | GUI Code Maps viewer | Highlight cross-file references (imports/exports) | The user sees how a file connects to the rest of the codebase |

## 3. Data model

```rust
pub struct CodeMap {
    pub path: PathBuf,
    pub language: Language,
    pub imports: Vec<Import>,           // 'import { foo } from "./bar"'
    pub exports: Vec<Export>,           // 'export class Foo'
    pub items: Vec<Item>,               // top-level decls (classes, functions, types)
    pub stats: CodeMapStats,            // line count, token estimate
}

pub enum Item {
    Class { name: String, methods: Vec<Signature>, fields: Vec<Signature>, doc: Option<String> },
    Function(Signature),
    Type { name: String, definition: String },
    Constant { name: String, type_hint: Option<String>, doc: Option<String> },
}

pub struct Signature {
    pub name: String,
    pub params: Vec<Param>,             // names + type hints
    pub return_type: Option<String>,
    pub doc: Option<String>,            // first paragraph of docstring
    pub line: u32,
    pub is_async: bool,
    pub is_exported: bool,
}

pub enum Language { Rust, TypeScript, JavaScript, Python, Go, Ruby }
```

Token estimate: 1 token ≈ 4 chars; conservative.

## 4. Public API

```rust
pub fn detect_language(path: &Path) -> Option<Language>;
pub fn generate(path: &Path) -> Result<CodeMap>;
pub fn generate_directory(root: &Path, opts: ScanOpts) -> Result<Vec<CodeMap>>;
pub fn render_markdown(map: &CodeMap) -> String;       // for prompt embedding
pub fn budget(maps: Vec<CodeMap>, max_tokens: usize) -> Vec<CodeMap>;  // priority-based pruning
```

Called directly as a Rust crate by `skill-runner`. Exposed to React via Tauri commands from `gui-shell`. No N-API, no Node bindings — see `SPEC_REVISION_2026-05-08.md` for the pivot rationale.

## 5. Business rules

- **Languages in v1:** Rust, TypeScript, JavaScript, Python, Go, Ruby.
- **Languages in v1.1:** Java, C, C++.
- **Detection** by file extension first; shebang fallback for ambiguous cases (e.g. `.h`).
- **Budgeting priority:** exports > public functions > classes > private items > constants. Truncate from lowest priority when over budget.
- **Doc extraction:** first paragraph only; multi-line / `/** */` style; collapse whitespace.
- **Body stripping** is the whole point — never include function bodies in the output.
- **No code modification.** Read-only over filesystem.

## 6. Integration points

| Module | Relationship | Notes |
|---|---|---|
| `skill-runner` | Used by | Generates Code Maps when assembling agent prompts (direct Rust call) |
| `gui-shell` | Used by | File picker viewer + symbol outline (Tauri commands) |
| `session-store` | None | Code Maps regenerate on demand; not persisted |
| `spec-engine` | Used by | Optional — `/project-spec` may call `generate_directory` to auto-fill Integration Points |

## 7. Acceptance criteria

- [ ] `cargo test --package code-maps` passes (one fixture file per supported language with golden-output assertion).
- [ ] Generates a Code Map for a 1000-line TypeScript file in <100ms.
- [ ] `generate_directory` on a 50-file Rust crate completes in <2s.
- [ ] Token estimate within ±15% of actual tokenisation by `tiktoken-rs` for a sample.
- [ ] `serde_json` round-trip of `CodeMap` is lossless (used by Tauri command serialisation and for cache-on-disk if added later).
- [ ] Tauri command exposed as `code_map_for_file(path: PathBuf)` — wiring lives in `gui-shell` but the Rust signature must accept a `PathBuf` directly so the Tauri shim is mechanical.
- [ ] Fallback for files in unsupported languages: return a Code Map with empty `items` + path metadata only.

## 8. Out of scope

- Semantic analysis (callgraph, type inference) — pure structural extraction only.
- Caching to disk (regenerate on demand; speed is sufficient).
- Custom language grammars (use crates.io grammars).
- Treemap visualisation (gui-shell can do that on top).
- Cross-file dependency resolution beyond imports list.

## 9. Open questions

- [ ] **Incremental updates:** v1 = full rescans only; v1.1 = watch-mode + tree-sitter incremental edits. Where does that switch happen?
- [ ] **Doc extraction quality** varies wildly per language; do we ship language-specific extractors, or a single generic one?
- [ ] **Nested types** (e.g. inner classes) — flatten or preserve hierarchy? Probably preserve.
- [ ] **Sourcemap-aware mode** for built JS — skip `dist/` automatically, or surface as a setting?
