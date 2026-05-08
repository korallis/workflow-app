use std::fs;
use std::sync::Arc;

use kit_session_store::SessionStore;
use kit_spec_engine::{
    MasterBlueprint, ModuleSpec, ParallelYaml, ProjectBrief, Roadmap, SpecEngine, SpecEngineError,
};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    engine: SpecEngine,
}

impl Fixture {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let store = Arc::new(SessionStore::open_in_memory().unwrap());
        let engine = SpecEngine::new(store, temp.path().to_path_buf());
        Self {
            _temp: temp,
            engine,
        }
    }

    fn path(&self, rel: &str) -> std::path::PathBuf {
        self._temp.path().join(rel)
    }

    fn write(&self, rel: &str, body: &str) {
        let path = self.path(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, body).unwrap();
    }
}

#[test]
fn saves_and_loads_top_level_specs_with_snapshots() {
    let fx = Fixture::new();
    fx.engine
        .save_brief(&ProjectBrief {
            markdown: "# Brief\n".to_owned(),
        })
        .unwrap();
    fx.engine
        .save_blueprint(&MasterBlueprint {
            markdown: "# Blueprint\n".to_owned(),
        })
        .unwrap();
    fx.write("specs/ROADMAP.md", "# Roadmap\n");

    assert_eq!(fx.engine.load_brief().unwrap().markdown, "# Brief\n");
    assert_eq!(
        fx.engine.load_blueprint().unwrap().markdown,
        "# Blueprint\n"
    );
    assert_eq!(
        fx.engine.load_roadmap().unwrap(),
        Roadmap {
            markdown: "# Roadmap\n".to_owned()
        }
    );

    let history = fx
        .engine
        .snapshot_history("specs/PROJECT_BRIEF.md")
        .unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].body, "# Brief\n");
}

#[test]
fn snapshots_are_deduplicated_by_content_hash() {
    let fx = Fixture::new();
    let brief = ProjectBrief {
        markdown: "# Brief\n".to_owned(),
    };
    fx.engine.save_brief(&brief).unwrap();
    fx.engine.save_brief(&brief).unwrap();

    assert_eq!(
        fx.engine
            .snapshot_history("specs/PROJECT_BRIEF.md")
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn loads_modules_from_markdown_table() {
    let fx = Fixture::new();
    fx.write(
        "specs/MODULES.md",
        r#"
| # | Name | Layer | Language | Depends on | Used by | Purpose |
|---|---|---|---|---|---|---|
| 1 | **session-store** | Core | Rust | — | spec-engine | Store |
| 2 | **spec-engine** | Core | Rust | session-store | gui-shell | Specs |
| 3 | **oauth** | Core | Rust | — | gui-shell | OAuth |
| 4 | **auth** | Core | Rust | oauth | gui-shell | Auth |
"#,
    );

    let modules = fx.engine.load_modules().unwrap();
    assert_eq!(modules.len(), 4);
    assert_eq!(modules[1].name, "spec-engine");
    assert_eq!(modules[1].layer, "Core");
    assert_eq!(modules[1].depends_on, vec!["session-store"]);
    assert_eq!(modules[3].depends_on, vec!["oauth"]);
}

#[test]
fn validates_parallel_yaml_and_reports_distinct_version_errors() {
    let fx = Fixture::new();

    fx.engine
        .validate_parallel_yaml_str(
            r#"
version: 1
touches:
  - crates/spec-engine
shared: []
ports:
  - 3000
migrations: false
"#,
        )
        .unwrap();

    let missing = fx
        .engine
        .validate_parallel_yaml_str("touches: []")
        .unwrap_err();
    assert!(matches!(missing, SpecEngineError::MissingParallelVersion));

    let unsupported = fx
        .engine
        .validate_parallel_yaml_str("version: 2")
        .unwrap_err();
    assert!(matches!(
        unsupported,
        SpecEngineError::UnsupportedParallelVersion { .. }
    ));
}

#[test]
fn round_trips_parallel_yaml_fixture() {
    let fx = Fixture::new();
    let parallel = ParallelYaml {
        version: 1,
        touches: vec!["crates/kit-spec-engine".to_owned()],
        shared: vec!["Cargo.toml".to_owned()],
        ports: vec![1420],
        migrations: true,
    };
    let module = ModuleSpec {
        name: "spec-engine".to_owned(),
        spec_md: "# Spec\n".to_owned(),
        claude_md: "# Guide\n".to_owned(),
        parallel_yaml: Some(parallel.clone()),
    };

    fx.engine.save_module_spec(&module).unwrap();
    assert_eq!(
        fx.engine.load_parallel_yaml("spec-engine").unwrap(),
        parallel
    );
    assert_eq!(fx.engine.load_module_spec("spec-engine").unwrap(), module);
}

#[test]
fn rejects_invalid_module_slugs_before_path_joining() {
    let fx = Fixture::new();
    let err = fx.engine.load_module_spec("../../etc/passwd").unwrap_err();
    assert!(matches!(err, SpecEngineError::InvalidModuleSlug { .. }));
}

#[test]
fn save_module_spec_rolls_back_when_batch_fails() {
    let fx = Fixture::new();
    fs::create_dir_all(fx.path("specs/modules/spec-engine")).unwrap();
    fs::create_dir(fx.path("specs/modules/spec-engine/CLAUDE.md")).unwrap();
    fs::create_dir(fx.path("specs/modules/spec-engine/CLAUDE.md.bak")).unwrap();
    fs::write(
        fx.path("specs/modules/spec-engine/CLAUDE.md.bak/keep"),
        "collision",
    )
    .unwrap();

    let module = ModuleSpec {
        name: "spec-engine".to_owned(),
        spec_md: "# Spec\n".to_owned(),
        claude_md: "# Guide\n".to_owned(),
        parallel_yaml: Some(ParallelYaml {
            version: 1,
            touches: vec![],
            shared: vec![],
            ports: vec![],
            migrations: false,
        }),
    };

    assert!(fx.engine.save_module_spec(&module).is_err());
    assert!(!fx.path("specs/modules/spec-engine/SPEC.md").exists());
    assert!(fx.path("specs/modules/spec-engine/CLAUDE.md").is_dir());
    assert!(!fx.path("specs/modules/spec-engine/parallel.yaml").exists());
}
