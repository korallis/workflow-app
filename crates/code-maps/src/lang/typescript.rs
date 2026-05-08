use std::path::{Path, PathBuf};

use crate::lang::common::{base_map, clean_doc, line_no, parse_params, push_export, signature};
use crate::{CodeMap, Import, Item, Language, LanguageExtractor};

pub struct TypeScriptExtractor;

impl LanguageExtractor for TypeScriptExtractor {
    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn tree_sitter_language(&self, path: &Path) -> tree_sitter::Language {
        if path.extension().and_then(|e| e.to_str()) == Some("tsx") {
            tree_sitter_typescript::LANGUAGE_TSX.into()
        } else {
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
        }
    }

    fn extract(&self, path: PathBuf, source: &str) -> CodeMap {
        let mut map = base_map(path, self.language(), source);
        let mut docs = Vec::new();

        for (idx, raw) in source.lines().enumerate() {
            let line = raw.trim();
            collect_js_doc(line, &mut docs);
            if line.is_empty()
                || line.starts_with('*')
                || line.starts_with("//")
                || line.starts_with("/*")
            {
                continue;
            }
            let line_no = line_no(idx);
            if line.starts_with("import ") {
                map.imports.push(parse_import(line, line_no));
            } else if line.starts_with("export ") && line.contains(" from ") {
                map.exports.push(crate::Export {
                    name: line.trim_end_matches(';').to_string(),
                    kind: "re-export".to_string(),
                    line: line_no,
                });
            } else if let Some((kind, name)) = parse_type_decl(line) {
                push_export_if_needed(line, &mut map.exports, &name, kind, line_no);
                map.items.push(Item::Type {
                    name,
                    definition: strip_body(line).to_string(),
                });
            } else if let Some((name, params, ret, is_async, exported)) = parse_function_like(line)
            {
                if exported {
                    push_export(&mut map.exports, &name, "function", line_no);
                }
                map.items.push(Item::Function(signature(
                    name,
                    params,
                    ret,
                    clean_doc(&docs),
                    line_no,
                    is_async,
                    exported,
                )));
            } else if let Some(name) = parse_class(line) {
                let exported = line.starts_with("export ");
                if exported {
                    push_export(&mut map.exports, &name, "class", line_no);
                }
                map.items.push(Item::Class {
                    name,
                    methods: Vec::new(),
                    fields: Vec::new(),
                    doc: clean_doc(&docs),
                });
            } else if let Some((name, hint, exported)) = parse_const(line) {
                if exported {
                    push_export(&mut map.exports, &name, "constant", line_no);
                }
                map.items.push(Item::Constant {
                    name,
                    type_hint: hint,
                    doc: clean_doc(&docs),
                });
            }
            if !line.starts_with('*') {
                docs.clear();
            }
        }

        map
    }
}

pub(crate) fn collect_js_doc(line: &str, docs: &mut Vec<String>) {
    if line.starts_with("/**") || line.starts_with("/*") {
        let cleaned = line
            .trim_start_matches("/**")
            .trim_start_matches("/*")
            .trim();
        if !cleaned.is_empty() && cleaned != "*/" {
            docs.push(cleaned.trim_end_matches("*/").trim().to_string());
        }
    } else if line.starts_with('*') {
        let cleaned = line
            .trim_start_matches('*')
            .trim()
            .trim_end_matches("*/")
            .trim();
        if !cleaned.is_empty() {
            docs.push(cleaned.to_string());
        }
    }
}

pub(crate) fn parse_import(line: &str, line_no: u32) -> Import {
    let module = if let Some((_, module)) = line.rsplit_once(" from ") {
        module
    } else {
        line.strip_prefix("import ").unwrap_or(line)
    }
    .trim()
    .trim_end_matches(';')
    .trim_matches('"')
    .trim_matches('\'')
    .to_string();
    Import {
        module,
        items: Vec::new(),
        line: line_no,
    }
}

pub(crate) fn parse_function_like(
    line: &str,
) -> Option<(String, Vec<crate::Param>, Option<String>, bool, bool)> {
    let exported = line.starts_with("export ");
    let normalized = line
        .strip_prefix("export default ")
        .or_else(|| line.strip_prefix("export "))
        .unwrap_or(line);
    let is_async = normalized.starts_with("async ");
    let normalized = normalized.strip_prefix("async ").unwrap_or(normalized);
    if let Some(rest) = normalized.strip_prefix("function ") {
        let name_end = rest.find('(')?;
        let name = rest[..name_end].trim().to_string();
        let params_end = rest[name_end + 1..].find(')')? + name_end + 1;
        let params = parse_params(&rest[name_end + 1..params_end]);
        let ret = rest[params_end + 1..]
            .split('{')
            .next()
            .and_then(|s| s.trim().strip_prefix(':'))
            .map(|s| s.trim().to_string());
        return Some((name, params, ret, is_async, exported));
    }
    for prefix in ["const ", "let ", "var "] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            let name = rest.split([':', '=', ' ']).next()?.trim().to_string();
            if rest.contains("=>") {
                let params_raw = rest
                    .split('=')
                    .nth(1)?
                    .split("=>")
                    .next()?
                    .trim()
                    .trim_start_matches('(')
                    .trim_end_matches(')');
                return Some((name, parse_params(params_raw), None, is_async, exported));
            }
        }
    }
    None
}

fn parse_type_decl(line: &str) -> Option<(&'static str, String)> {
    let normalized = line.strip_prefix("export ").unwrap_or(line);
    for (prefix, kind) in [
        ("interface ", "interface"),
        ("type ", "type"),
        ("enum ", "enum"),
    ] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()?;
            return Some((kind, name.to_string()));
        }
    }
    None
}

fn parse_class(line: &str) -> Option<String> {
    let normalized = line
        .strip_prefix("export default ")
        .or_else(|| line.strip_prefix("export "))
        .unwrap_or(line);
    let rest = normalized.strip_prefix("class ")?;
    Some(
        rest.split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .next()?
            .to_string(),
    )
}

fn parse_const(line: &str) -> Option<(String, Option<String>, bool)> {
    let exported = line.starts_with("export ");
    let normalized = line.strip_prefix("export ").unwrap_or(line);
    for prefix in ["const ", "let "] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            let name = rest.split([':', '=', ' ']).next()?.trim().to_string();
            let hint = rest.split_once(':').map(|(_, hint)| {
                hint.split('=')
                    .next()
                    .unwrap_or(hint)
                    .trim()
                    .trim_end_matches(';')
                    .to_string()
            });
            return Some((name, hint, exported));
        }
    }
    None
}

fn push_export_if_needed(
    line: &str,
    exports: &mut Vec<crate::Export>,
    name: &str,
    kind: &str,
    line_no: u32,
) {
    if line.starts_with("export ") {
        push_export(exports, name, kind, line_no);
    }
}

fn strip_body(line: &str) -> &str {
    line.split('{')
        .next()
        .unwrap_or(line)
        .trim_end_matches(';')
        .trim()
}
