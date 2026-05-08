use crate::{stats_for, CodeMap, Export, Item, Param, Signature};
use std::path::PathBuf;

pub fn base_map(path: PathBuf, language: crate::Language, source: &str) -> CodeMap {
    CodeMap {
        path,
        language,
        imports: Vec::new(),
        exports: Vec::new(),
        items: Vec::new(),
        stats: stats_for(source),
    }
}

pub fn clean_doc(lines: &[String]) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    let joined = lines.join(" ");
    let first = joined.split("\n\n").next().unwrap_or(&joined);
    let cleaned = first.split_whitespace().collect::<Vec<_>>().join(" ");
    (!cleaned.is_empty()).then_some(cleaned)
}

pub fn parse_params(raw: &str) -> Vec<Param> {
    split_top_level(raw, ',')
        .into_iter()
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() || part == "self" || part == "&self" || part == "&mut self" {
                return None;
            }
            let (name, type_hint) = if let Some((name, hint)) = part.split_once(':') {
                (name.trim(), Some(hint.trim().to_string()))
            } else if let Some((name, hint)) = part.split_once(' ') {
                (name.trim(), Some(hint.trim().to_string()))
            } else {
                (part, None)
            };
            let name = name
                .trim_start_matches("mut ")
                .trim_start_matches('&')
                .trim_start_matches("...")
                .to_string();
            (!name.is_empty()).then_some(Param { name, type_hint })
        })
        .collect()
}

pub fn split_top_level(raw: &str, delimiter: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    for (idx, ch) in raw.char_indices() {
        match ch {
            '(' | '<' | '[' | '{' => depth += 1,
            ')' | '>' | ']' | '}' => depth -= 1,
            _ => {}
        }
        if ch == delimiter && depth == 0 {
            out.push(raw[start..idx].to_string());
            start = idx + ch.len_utf8();
        }
    }
    out.push(raw[start..].to_string());
    out
}

pub fn signature(
    name: String,
    params: Vec<Param>,
    return_type: Option<String>,
    doc: Option<String>,
    line: u32,
    is_async: bool,
    is_exported: bool,
) -> Signature {
    Signature {
        name,
        params,
        return_type: return_type.filter(|s| !s.trim().is_empty()),
        doc,
        line,
        is_async,
        is_exported,
    }
}

pub fn push_export(exports: &mut Vec<Export>, name: &str, kind: &str, line: u32) {
    if !exports.iter().any(|e| e.name == name && e.kind == kind) {
        exports.push(Export {
            name: name.to_string(),
            kind: kind.to_string(),
            line,
        });
    }
}

pub fn line_no(idx: usize) -> u32 {
    (idx + 1).try_into().unwrap_or(u32::MAX)
}
