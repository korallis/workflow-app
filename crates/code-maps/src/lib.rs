mod error;
mod lang;
mod render;
mod types;

use std::fs;
use std::path::{Path, PathBuf};

use tree_sitter::Parser;
use walkdir::WalkDir;

pub use error::{CodeMapError, Result};
pub use types::*;

pub fn detect_language(path: &Path) -> Option<Language> {
    let file_name = path.file_name()?.to_string_lossy();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();

    match file_name.as_ref() {
        name if name.ends_with(".d.ts") => Some(Language::TypeScript),
        _ => match ext {
            "rs" => Some(Language::Rust),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
            "py" => Some(Language::Python),
            "go" => Some(Language::Go),
            "rb" => Some(Language::Ruby),
            _ => detect_shebang(path),
        },
    }
}

pub fn generate(path: &Path) -> Result<CodeMap> {
    let source = fs::read_to_string(path).map_err(|source| CodeMapError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if detect_language(path).is_none() {
        let mut map = CodeMap {
            path: path.to_path_buf(),
            language: Language::Unsupported,
            imports: Vec::new(),
            exports: Vec::new(),
            items: Vec::new(),
            stats: stats_for(&source),
        };
        refresh_stats_from_render(&mut map);
        return Ok(map);
    }
    generate_from_source(path.to_path_buf(), &source)
}

pub fn generate_directory(root: &Path, opts: ScanOpts) -> Result<Vec<CodeMap>> {
    let mut maps = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|source| CodeMapError::WalkDir {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if !opts.include_hidden && path_has_hidden_component(path, root) {
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        if detect_language(path).is_none() {
            continue;
        }
        if entry
            .metadata()
            .map(|m| m.len() as usize)
            .unwrap_or(usize::MAX)
            > opts.max_file_bytes
        {
            continue;
        }
        maps.push(generate(path)?);
    }
    maps.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(maps)
}

pub fn render_markdown(map: &CodeMap) -> String {
    render::markdown(map)
}

pub fn budget(mut maps: Vec<CodeMap>, max_tokens: usize) -> Vec<CodeMap> {
    maps.sort_by_key(|map| std::cmp::Reverse(map_priority(map)));
    let mut out = Vec::new();
    let mut used = 0usize;

    for mut map in maps {
        prune_items(&mut map.items, max_tokens.saturating_sub(used));
        refresh_stats_from_render(&mut map);
        if used + map.stats.token_estimate <= max_tokens {
            used += map.stats.token_estimate;
            out.push(map);
        }
    }

    out
}

pub(crate) trait LanguageExtractor {
    fn language(&self) -> Language;
    fn tree_sitter_language(&self, path: &Path) -> tree_sitter::Language;
    fn extract(&self, path: PathBuf, source: &str) -> CodeMap;
}

pub(crate) fn generate_from_source(path: PathBuf, source: &str) -> Result<CodeMap> {
    let language = detect_language(&path)
        .ok_or_else(|| CodeMapError::UnsupportedLanguage { path: path.clone() })?;
    let extractor = lang::extractor(language.clone());

    let mut parser = Parser::new();
    parser.set_language(&extractor.tree_sitter_language(&path))?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| CodeMapError::ParseFailed { path: path.clone() })?;
    if tree.root_node().has_error() {
        return Err(CodeMapError::ParseFailed { path });
    }

    Ok(extractor.extract(path, source))
}

fn detect_shebang(path: &Path) -> Option<Language> {
    let content = fs::read_to_string(path).ok()?;
    let first = content.lines().next()?.trim();
    if !first.starts_with("#!") {
        return None;
    }
    if first.contains("python") {
        Some(Language::Python)
    } else if first.contains("ruby") {
        Some(Language::Ruby)
    } else if first.contains("node") || first.contains("deno") {
        Some(Language::JavaScript)
    } else {
        None
    }
}

fn path_has_hidden_component(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
}

fn map_priority(map: &CodeMap) -> usize {
    map.exports.len() * 100 + map.items.iter().map(item_priority).max().unwrap_or(0)
}

fn item_priority(item: &Item) -> usize {
    match item {
        Item::Function(sig) if sig.is_exported => 90,
        Item::Class { .. } => 80,
        Item::Function(_) => 60,
        Item::Type { .. } => 50,
        Item::Constant { .. } => 20,
    }
}

fn prune_items(items: &mut Vec<Item>, max_tokens: usize) {
    items.sort_by_key(|item| std::cmp::Reverse(item_priority(item)));
    while estimate_items(items) > max_tokens && !items.is_empty() {
        items.pop();
    }
}

fn estimate_items(items: &[Item]) -> usize {
    estimate_tokens(&format!("{items:?}"))
}

pub(crate) fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

fn refresh_stats_from_render(map: &mut CodeMap) {
    let rendered = render_markdown(map);
    map.stats.token_estimate = estimate_tokens(&rendered);
}

pub(crate) fn stats_for(source: &str) -> CodeMapStats {
    CodeMapStats {
        line_count: source.lines().count(),
        char_count: source.chars().count(),
        token_estimate: estimate_tokens(source),
    }
}
