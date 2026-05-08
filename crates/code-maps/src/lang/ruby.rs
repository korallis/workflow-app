use std::path::{Path, PathBuf};

use crate::lang::common::{base_map, line_no, parse_params, push_export, signature};
use crate::{CodeMap, Import, Item, Language, LanguageExtractor};

pub struct RubyExtractor;

impl LanguageExtractor for RubyExtractor {
    fn language(&self) -> Language {
        Language::Ruby
    }

    fn tree_sitter_language(&self, _path: &Path) -> tree_sitter::Language {
        tree_sitter_ruby::LANGUAGE.into()
    }

    fn extract(&self, path: PathBuf, source: &str) -> CodeMap {
        let mut map = base_map(path, self.language(), source);
        for (idx, raw) in source.lines().enumerate() {
            let line = raw.trim();
            let line_no = line_no(idx);
            if line.starts_with("require ") || line.starts_with("require_relative ") {
                map.imports.push(Import {
                    module: line.to_string(),
                    items: Vec::new(),
                    line: line_no,
                });
            } else if let Some(name) = line.strip_prefix("class ") {
                let name = name.split(['<', ' ']).next().unwrap_or_default().trim();
                if !name.is_empty() {
                    push_export(&mut map.exports, name, "class", line_no);
                    map.items.push(Item::Class {
                        name: name.to_string(),
                        methods: Vec::new(),
                        fields: Vec::new(),
                        doc: None,
                    });
                }
            } else if let Some(sig) = parse_ruby_def(line, line_no) {
                push_export(&mut map.exports, &sig.name, "function", line_no);
                map.items.push(Item::Function(sig));
            }
        }
        map
    }
}

fn parse_ruby_def(line: &str, line_no: u32) -> Option<crate::Signature> {
    let rest = line.strip_prefix("def ")?;
    let name = rest.split(['(', ' ']).next()?.trim().to_string();
    let params = if let Some(start) = rest.find('(') {
        let end = rest[start + 1..].find(')')? + start + 1;
        parse_params(&rest[start + 1..end])
    } else {
        Vec::new()
    };
    Some(signature(name, params, None, None, line_no, false, true))
}
