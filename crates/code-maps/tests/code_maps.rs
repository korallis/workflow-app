use std::path::{Path, PathBuf};
use std::time::Instant;

use kit_code_maps::{
    budget, detect_language, generate, generate_directory, render_markdown, CodeMap, Language,
    ScanOpts,
};

#[test]
fn detects_supported_extensions() {
    assert_eq!(detect_language(Path::new("lib.rs")), Some(Language::Rust));
    assert_eq!(
        detect_language(Path::new("view.tsx")),
        Some(Language::TypeScript)
    );
    assert_eq!(
        detect_language(Path::new("types.d.ts")),
        Some(Language::TypeScript)
    );
    assert_eq!(
        detect_language(Path::new("mod.mjs")),
        Some(Language::JavaScript)
    );
    assert_eq!(
        detect_language(Path::new("script.cjs")),
        Some(Language::JavaScript)
    );
    assert_eq!(
        detect_language(Path::new("main.py")),
        Some(Language::Python)
    );
    assert_eq!(detect_language(Path::new("main.go")), Some(Language::Go));
    assert_eq!(detect_language(Path::new("app.rb")), Some(Language::Ruby));
}

#[test]
fn rust_fixture_matches_golden() {
    assert_golden("rust/sample.rs", "rust/expected.json");
}

#[test]
fn typescript_fixture_matches_golden() {
    assert_golden("typescript/sample.ts", "typescript/expected.json");
}

#[test]
fn javascript_fixture_matches_golden() {
    assert_golden("javascript/sample.js", "javascript/expected.json");
}

#[test]
fn python_fixture_matches_golden() {
    assert_golden("python/sample.py", "python/expected.json");
}

#[test]
fn go_fixture_matches_golden() {
    assert_golden("go/sample.go", "go/expected.json");
}

#[test]
fn ruby_fixture_matches_golden() {
    assert_golden("ruby/sample.rb", "ruby/expected.json");
}

#[test]
fn markdown_does_not_render_function_bodies() {
    let map = generate(fixture("typescript/sample.ts").as_path()).unwrap();
    let rendered = render_markdown(&map);
    assert!(rendered.contains("fetchUser(id: string) -> Promise<User>"));
    assert!(!rendered.contains("return api.get"));
}

#[test]
fn serde_roundtrip_is_lossless() {
    let map = generate(fixture("rust/sample.rs").as_path()).unwrap();
    let encoded = serde_json::to_string(&map).unwrap();
    let decoded: CodeMap = serde_json::from_str(&encoded).unwrap();
    assert_eq!(map, decoded);
}

#[test]
fn unsupported_file_returns_metadata_only() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.txt");
    std::fs::write(&file, "plain text").unwrap();

    let map = generate(&file).unwrap();
    assert_eq!(map.language, Language::Unsupported);
    assert!(map.imports.is_empty());
    assert!(map.exports.is_empty());
    assert!(map.items.is_empty());
}

#[test]
fn budget_prunes_to_requested_estimate() {
    let maps = vec![
        generate(fixture("rust/sample.rs").as_path()).unwrap(),
        generate(fixture("typescript/sample.ts").as_path()).unwrap(),
        generate(fixture("python/sample.py").as_path()).unwrap(),
    ];
    let pruned = budget(maps, 120);
    let total: usize = pruned.iter().map(|m| m.stats.token_estimate).sum();
    assert!(total <= 120);
}

#[test]
fn directory_scan_stays_within_root() {
    let root = fixtures_root();
    let maps = generate_directory(&root, ScanOpts::default()).unwrap();
    assert!(maps.len() >= 6);
    assert!(maps.iter().all(|m| m.path.starts_with(&root)));
}

#[test]
fn parses_large_typescript_file_under_100ms() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("large.ts");
    let mut source = String::from("export interface User { id: string }\n");
    for i in 0..1000 {
        source.push_str(&format!(
            "export function f{i}(id: string): string {{ return id; }}\n"
        ));
    }
    std::fs::write(&file, source).unwrap();

    let start = Instant::now();
    let map = generate(&file).unwrap();
    assert!(start.elapsed().as_millis() < 100);
    assert_eq!(map.items.len(), 1001);
}

#[test]
fn token_estimate_is_char_count_div_four() {
    let map = generate(fixture("javascript/sample.js").as_path()).unwrap();
    let expected = map.stats.char_count.div_ceil(4);
    let delta = map.stats.token_estimate.abs_diff(expected);
    assert!(delta <= expected / 100 + 1);
}

fn assert_golden(sample: &str, expected: &str) {
    let mut actual = generate(fixture(sample).as_path()).unwrap();
    actual.path = PathBuf::from(sample);
    let expected: CodeMap =
        serde_json::from_str(&std::fs::read_to_string(fixture(expected)).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

fn fixture(path: &str) -> PathBuf {
    fixtures_root().join(path)
}

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}
