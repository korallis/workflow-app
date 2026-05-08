pub mod common;
pub mod go;
pub mod javascript;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod typescript;

use crate::{Language, LanguageExtractor};

static RUST: rust::RustExtractor = rust::RustExtractor;
static TYPESCRIPT: typescript::TypeScriptExtractor = typescript::TypeScriptExtractor;
static JAVASCRIPT: javascript::JavaScriptExtractor = javascript::JavaScriptExtractor;
static PYTHON: python::PythonExtractor = python::PythonExtractor;
static GO: go::GoExtractor = go::GoExtractor;
static RUBY: ruby::RubyExtractor = ruby::RubyExtractor;

pub(crate) fn extractor(language: Language) -> &'static dyn LanguageExtractor {
    match language {
        Language::Rust => &RUST,
        Language::TypeScript => &TYPESCRIPT,
        Language::JavaScript => &JAVASCRIPT,
        Language::Python => &PYTHON,
        Language::Go => &GO,
        Language::Ruby => &RUBY,
        Language::Unsupported => &JAVASCRIPT,
    }
}
