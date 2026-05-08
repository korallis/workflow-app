use std::path::{Path, PathBuf};

use crate::lang::common::{base_map, line_no, parse_params, push_export, signature};
use crate::{CodeMap, Import, Item, Language, LanguageExtractor};

pub struct PythonExtractor;

impl LanguageExtractor for PythonExtractor {
    fn language(&self) -> Language {
        Language::Python
    }

    fn tree_sitter_language(&self, _path: &Path) -> tree_sitter::Language {
        tree_sitter_python::LANGUAGE.into()
    }

    fn extract(&self, path: PathBuf, source: &str) -> CodeMap {
        let mut map = base_map(path, self.language(), source);
        for (idx, raw) in source.lines().enumerate() {
            let line = raw.trim();
            let line_no = line_no(idx);
            if line.starts_with("import ") || line.starts_with("from ") {
                map.imports.push(Import {
                    module: line.to_string(),
                    items: Vec::new(),
                    line: line_no,
                });
            } else if let Some(sig) = parse_python_fn(line, line_no) {
                if !sig.name.starts_with('_') {
                    push_export(&mut map.exports, &sig.name, "function", line_no);
                }
                map.items.push(Item::Function(sig));
            } else if let Some(name) = line
                .strip_prefix("class ")
                .and_then(|s| s.split(['(', ':']).next())
            {
                let name = name.trim();
                if !name.is_empty() {
                    push_export(&mut map.exports, name, "class", line_no);
                    map.items.push(Item::Class {
                        name: name.to_string(),
                        methods: Vec::new(),
                        fields: Vec::new(),
                        doc: None,
                    });
                }
            }
        }
        map
    }
}

fn parse_python_fn(line: &str, line_no: u32) -> Option<crate::Signature> {
    let is_async = line.starts_with("async def ");
    let rest = line
        .strip_prefix("async def ")
        .or_else(|| line.strip_prefix("def "))?;
    let name_end = rest.find('(')?;
    let name = rest[..name_end].trim().to_string();
    let params_end = rest[name_end + 1..].find(')')? + name_end + 1;
    let params = parse_params(&rest[name_end + 1..params_end]);
    let return_type = rest[params_end + 1..]
        .split(':')
        .next()
        .and_then(|s| s.trim().strip_prefix("->"))
        .map(|s| s.trim().to_string());
    Some(signature(
        name.clone(),
        params,
        return_type,
        None,
        line_no,
        is_async,
        !name.starts_with('_'),
    ))
}
