use std::path::{Path, PathBuf};

use crate::lang::common::{base_map, clean_doc, line_no, parse_params, push_export, signature};
use crate::{CodeMap, Import, Item, Language, LanguageExtractor};

pub struct RustExtractor;

impl LanguageExtractor for RustExtractor {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn tree_sitter_language(&self, _path: &Path) -> tree_sitter::Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn extract(&self, path: PathBuf, source: &str) -> CodeMap {
        let mut map = base_map(path, self.language(), source);
        let mut docs = Vec::new();

        for (idx, raw) in source.lines().enumerate() {
            let line = raw.trim();
            if let Some(doc) = line.strip_prefix("///") {
                docs.push(doc.trim().to_string());
                continue;
            }
            if line.is_empty() || line.starts_with("#[") {
                continue;
            }

            let line_no = line_no(idx);
            if let Some(rest) = line.strip_prefix("use ") {
                map.imports.push(Import {
                    module: rest.trim_end_matches(';').to_string(),
                    items: Vec::new(),
                    line: line_no,
                });
            } else if let Some(rest) = line.strip_prefix("mod ") {
                map.imports.push(Import {
                    module: rest.trim_end_matches(';').to_string(),
                    items: Vec::new(),
                    line: line_no,
                });
            } else if line.contains("fn ") {
                if let Some(sig) = parse_rust_fn(line, clean_doc(&docs), line_no) {
                    if sig.is_exported {
                        push_export(&mut map.exports, &sig.name, "function", line_no);
                    }
                    map.items.push(Item::Function(sig));
                }
            } else if let Some((kind, name)) = parse_named_decl(line) {
                let exported = line.starts_with("pub ");
                if exported {
                    push_export(&mut map.exports, &name, kind, line_no);
                }
                match kind {
                    "struct" | "enum" | "trait" => map.items.push(Item::Class {
                        name,
                        methods: Vec::new(),
                        fields: Vec::new(),
                        doc: clean_doc(&docs),
                    }),
                    "type" => map.items.push(Item::Type {
                        name,
                        definition: strip_body(line).to_string(),
                    }),
                    "constant" => map.items.push(Item::Constant {
                        name,
                        type_hint: line.split(':').nth(1).map(|s| {
                            s.split('=')
                                .next()
                                .unwrap_or(s)
                                .trim()
                                .trim_end_matches(';')
                                .to_string()
                        }),
                        doc: clean_doc(&docs),
                    }),
                    _ => {}
                }
            }

            docs.clear();
        }

        map
    }
}

fn parse_rust_fn(line: &str, doc: Option<String>, line_no: u32) -> Option<crate::Signature> {
    let exported = line.starts_with("pub ");
    let is_async = line.contains(" async fn ") || line.starts_with("async fn ");
    let fn_pos = line.find("fn ")? + 3;
    let after = &line[fn_pos..];
    let name_end = after.find('(')?;
    let name = after[..name_end].trim().to_string();
    let params_end = after[name_end + 1..].find(')')? + name_end + 1;
    let params = parse_params(&after[name_end + 1..params_end]);
    let return_type = after[params_end + 1..]
        .split('{')
        .next()
        .and_then(|s| s.split(';').next())
        .and_then(|s| s.trim().strip_prefix("->"))
        .map(|s| s.trim().to_string());
    Some(signature(
        name,
        params,
        return_type,
        doc,
        line_no,
        is_async,
        exported,
    ))
}

fn parse_named_decl(line: &str) -> Option<(&'static str, String)> {
    let normalized = line.strip_prefix("pub ").unwrap_or(line);
    for (prefix, kind) in [
        ("struct ", "struct"),
        ("enum ", "enum"),
        ("trait ", "trait"),
        ("type ", "type"),
        ("const ", "constant"),
        ("static ", "constant"),
    ] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()
                .unwrap_or_default();
            if !name.is_empty() {
                return Some((kind, name.to_string()));
            }
        }
    }
    None
}

fn strip_body(line: &str) -> &str {
    line.split('{')
        .next()
        .unwrap_or(line)
        .trim_end_matches(';')
        .trim()
}
