use crate::{CodeMap, Item, Param, Signature};

pub fn markdown(map: &CodeMap) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", map.path.display()));
    out.push_str(&format!("Language: {:?}\n", map.language));
    out.push_str(&format!(
        "Lines: {} | Estimated tokens: {}\n\n",
        map.stats.line_count, map.stats.token_estimate
    ));

    if !map.imports.is_empty() {
        out.push_str("## Imports\n");
        for import in &map.imports {
            out.push_str(&format!("- L{} `{}`\n", import.line, import.module));
        }
        out.push('\n');
    }

    if !map.exports.is_empty() {
        out.push_str("## Exports\n");
        for export in &map.exports {
            out.push_str(&format!(
                "- L{} {} `{}`\n",
                export.line, export.kind, export.name
            ));
        }
        out.push('\n');
    }

    if !map.items.is_empty() {
        out.push_str("## Items\n");
        for item in &map.items {
            render_item(&mut out, item);
        }
    }

    out
}

fn render_item(out: &mut String, item: &Item) {
    match item {
        Item::Class {
            name,
            methods,
            fields,
            doc,
        } => {
            out.push_str(&format!("- class `{name}`\n"));
            render_doc(out, doc.as_deref());
            for field in fields {
                out.push_str(&format!("  - field `{}`\n", field.name));
            }
            for method in methods {
                out.push_str(&format!("  - method `{}`\n", render_signature(method)));
            }
        }
        Item::Function(sig) => out.push_str(&format!("- fn `{}`\n", render_signature(sig))),
        Item::Type { name, definition } => {
            out.push_str(&format!("- type `{name}`: `{definition}`\n"))
        }
        Item::Constant {
            name,
            type_hint,
            doc,
        } => {
            out.push_str(&format!("- const `{name}`"));
            if let Some(type_hint) = type_hint {
                out.push_str(&format!(": `{type_hint}`"));
            }
            out.push('\n');
            render_doc(out, doc.as_deref());
        }
    }
}

fn render_signature(sig: &Signature) -> String {
    let params = sig
        .params
        .iter()
        .map(render_param)
        .collect::<Vec<_>>()
        .join(", ");
    let async_prefix = sig.is_async.then_some("async ").unwrap_or("");
    let ret = sig
        .return_type
        .as_ref()
        .map(|r| format!(" -> {r}"))
        .unwrap_or_default();
    format!("{async_prefix}{}({params}){ret} @L{}", sig.name, sig.line)
}

fn render_param(param: &Param) -> String {
    match &param.type_hint {
        Some(hint) => format!("{}: {}", param.name, hint),
        None => param.name.clone(),
    }
}

fn render_doc(out: &mut String, doc: Option<&str>) {
    if let Some(doc) = doc {
        out.push_str(&format!("  - doc: {doc}\n"));
    }
}
