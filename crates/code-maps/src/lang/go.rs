use std::path::{Path, PathBuf};

use crate::lang::common::{base_map, line_no, parse_params, push_export, signature};
use crate::{CodeMap, Import, Item, Language, LanguageExtractor};

pub struct GoExtractor;

impl LanguageExtractor for GoExtractor {
    fn language(&self) -> Language {
        Language::Go
    }

    fn tree_sitter_language(&self, _path: &Path) -> tree_sitter::Language {
        tree_sitter_go::LANGUAGE.into()
    }

    fn extract(&self, path: PathBuf, source: &str) -> CodeMap {
        let mut map = base_map(path, self.language(), source);
        for (idx, raw) in source.lines().enumerate() {
            let line = raw.trim();
            let line_no = line_no(idx);
            if line.starts_with("import ") {
                map.imports.push(Import {
                    module: line
                        .trim_start_matches("import ")
                        .trim_matches(['"', '(', ')'])
                        .to_string(),
                    items: Vec::new(),
                    line: line_no,
                });
            } else if let Some(sig) = parse_go_fn(line, line_no) {
                if sig.is_exported {
                    push_export(&mut map.exports, &sig.name, "function", line_no);
                }
                map.items.push(Item::Function(sig));
            } else if let Some((kind, name)) = parse_go_type(line) {
                if is_go_exported(&name) {
                    push_export(&mut map.exports, &name, kind, line_no);
                }
                map.items.push(Item::Type {
                    name,
                    definition: line.split('{').next().unwrap_or(line).trim().to_string(),
                });
            }
        }
        map
    }
}

fn parse_go_fn(line: &str, line_no: u32) -> Option<crate::Signature> {
    let rest = line.strip_prefix("func ")?;
    let (name, after_name) = if rest.starts_with('(') {
        let receiver_end = rest.find(')')?;
        let rest = rest[receiver_end + 1..].trim();
        let name_end = rest.find('(')?;
        (rest[..name_end].trim().to_string(), &rest[name_end..])
    } else {
        let name_end = rest.find('(')?;
        (rest[..name_end].trim().to_string(), &rest[name_end..])
    };
    let params_end = after_name[1..].find(')')? + 1;
    let params = parse_params(&after_name[1..params_end]);
    let return_type = after_name[params_end + 1..]
        .split('{')
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Some(signature(
        name.clone(),
        params,
        return_type,
        None,
        line_no,
        false,
        is_go_exported(&name),
    ))
}

fn parse_go_type(line: &str) -> Option<(&'static str, String)> {
    let rest = line.strip_prefix("type ")?;
    let name = rest.split_whitespace().next()?.to_string();
    let kind = if rest.contains(" struct") {
        "struct"
    } else {
        "type"
    };
    Some((kind, name))
}

fn is_go_exported(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}
