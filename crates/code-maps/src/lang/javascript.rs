use std::path::{Path, PathBuf};

use crate::lang::common::{base_map, clean_doc, line_no, push_export, signature};
use crate::lang::typescript::{collect_js_doc, parse_function_like, parse_import};
use crate::{CodeMap, Item, Language, LanguageExtractor};

pub struct JavaScriptExtractor;

impl LanguageExtractor for JavaScriptExtractor {
    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn tree_sitter_language(&self, _path: &Path) -> tree_sitter::Language {
        tree_sitter_javascript::LANGUAGE.into()
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
            if line.starts_with("import ")
                || line.starts_with("const ") && line.contains("require(")
            {
                map.imports.push(parse_import(line, line_no));
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
            } else if line.starts_with("export class ") || line.starts_with("class ") {
                let exported = line.starts_with("export ");
                let rest = line
                    .strip_prefix("export ")
                    .unwrap_or(line)
                    .strip_prefix("class ")
                    .unwrap_or(line);
                let name = rest
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .next()
                    .unwrap_or_default();
                if !name.is_empty() {
                    if exported {
                        push_export(&mut map.exports, name, "class", line_no);
                    }
                    map.items.push(Item::Class {
                        name: name.to_string(),
                        methods: Vec::new(),
                        fields: Vec::new(),
                        doc: clean_doc(&docs),
                    });
                }
            }
            docs.clear();
        }
        map
    }
}
