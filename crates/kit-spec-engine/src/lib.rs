mod error;
mod slug;
mod types;

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use kit_session_store::SessionStore;

pub use error::{Result, SpecEngineError};
pub use types::*;

const PARALLEL_SCHEMA: &str = include_str!("../schemas/parallel.yaml.schema.json");

pub struct SpecEngine {
    store: Arc<SessionStore>,
    project_root: PathBuf,
}

impl SpecEngine {
    pub fn new(store: Arc<SessionStore>, project_root: PathBuf) -> Self {
        Self {
            store,
            project_root,
        }
    }

    pub fn load_brief(&self) -> Result<ProjectBrief> {
        Ok(ProjectBrief {
            markdown: self.read_required("specs/PROJECT_BRIEF.md")?,
        })
    }

    pub fn save_brief(&self, brief: &ProjectBrief) -> Result<()> {
        self.write_spec("specs/PROJECT_BRIEF.md", &brief.markdown)
    }

    pub fn load_blueprint(&self) -> Result<MasterBlueprint> {
        Ok(MasterBlueprint {
            markdown: self.read_required("specs/MASTER_BLUEPRINT.md")?,
        })
    }

    pub fn save_blueprint(&self, b: &MasterBlueprint) -> Result<()> {
        self.write_spec("specs/MASTER_BLUEPRINT.md", &b.markdown)
    }

    pub fn load_modules(&self) -> Result<Vec<ModuleEntry>> {
        let body = self.read_required("specs/MODULES.md")?;
        parse_modules_table(&body)
    }

    pub fn load_roadmap(&self) -> Result<Roadmap> {
        Ok(Roadmap {
            markdown: self.read_required("specs/ROADMAP.md")?,
        })
    }

    pub fn load_module_spec(&self, name: &str) -> Result<ModuleSpec> {
        slug::validate_module_slug(name)?;
        let base = format!("specs/modules/{name}");
        let spec_md = self.read_required(&format!("{base}/SPEC.md"))?;
        let claude_md = self.read_required(&format!("{base}/CLAUDE.md"))?;
        let parallel_yaml = match self.read_optional(&format!("{base}/parallel.yaml"))? {
            Some(yaml) => Some(self.parse_parallel_yaml_str(&yaml)?),
            None => None,
        };

        Ok(ModuleSpec {
            name: name.to_owned(),
            spec_md,
            claude_md,
            parallel_yaml,
        })
    }

    pub fn save_module_spec(&self, s: &ModuleSpec) -> Result<()> {
        slug::validate_module_slug(&s.name)?;
        let base = PathBuf::from("specs").join("modules").join(&s.name);
        let base_abs = self.project_root.join(&base);
        fs::create_dir_all(&base_abs)?;

        let spec_rel = base.join("SPEC.md");
        let claude_rel = base.join("CLAUDE.md");
        let parallel_rel = base.join("parallel.yaml");

        let mut changes = vec![
            PlannedChange::Write {
                rel_path: spec_rel.clone(),
                body: s.spec_md.clone(),
            },
            PlannedChange::Write {
                rel_path: claude_rel.clone(),
                body: s.claude_md.clone(),
            },
        ];

        if let Some(parallel_yaml) = &s.parallel_yaml {
            let body = parallel_yaml_to_string(parallel_yaml);
            self.validate_parallel_yaml_str(&body)?;
            changes.push(PlannedChange::Write {
                rel_path: parallel_rel.clone(),
                body,
            });
        } else {
            changes.push(PlannedChange::Remove {
                rel_path: parallel_rel.clone(),
            });
        }

        apply_atomic_batch(&self.project_root, &changes)?;

        self.store
            .spec_snapshot(&path_for_snapshot(&spec_rel), &s.spec_md)?;
        self.store
            .spec_snapshot(&path_for_snapshot(&claude_rel), &s.claude_md)?;
        if let Some(parallel_yaml) = &s.parallel_yaml {
            let body = parallel_yaml_to_string(parallel_yaml);
            self.store
                .spec_snapshot(&path_for_snapshot(&parallel_rel), &body)?;
        }

        Ok(())
    }

    pub fn load_parallel_yaml(&self, name: &str) -> Result<ParallelYaml> {
        slug::validate_module_slug(name)?;
        let yaml = self.read_required(&format!("specs/modules/{name}/parallel.yaml"))?;
        self.parse_parallel_yaml_str(&yaml)
    }

    pub fn validate_parallel_yaml_str(&self, yaml: &str) -> Result<()> {
        let _schema_embedded_in_binary = PARALLEL_SCHEMA;
        parse_parallel_yaml(yaml)?;
        Ok(())
    }

    pub fn snapshot_history(&self, path: &str) -> Result<Vec<SpecSnapshot>> {
        Ok(self.store.spec_history(path)?)
    }

    fn parse_parallel_yaml_str(&self, yaml: &str) -> Result<ParallelYaml> {
        self.validate_parallel_yaml_str(yaml)?;
        parse_parallel_yaml(yaml)
    }

    fn write_spec(&self, rel_path: &str, body: &str) -> Result<()> {
        let rel = PathBuf::from(rel_path);
        atomic_write(&self.project_root.join(&rel), body)?;
        self.store.spec_snapshot(rel_path, body)?;
        Ok(())
    }

    fn read_required(&self, rel_path: &str) -> Result<String> {
        let path = self.project_root.join(rel_path);
        fs::read_to_string(&path).map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                SpecEngineError::MissingSpecFile { path }
            } else {
                SpecEngineError::Io(err)
            }
        })
    }

    fn read_optional(&self, rel_path: &str) -> Result<Option<String>> {
        let path = self.project_root.join(rel_path);
        match fs::read_to_string(&path) {
            Ok(body) => Ok(Some(body)),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

enum PlannedChange {
    Write { rel_path: PathBuf, body: String },
    Remove { rel_path: PathBuf },
}

struct Backup {
    final_path: PathBuf,
    backup_path: PathBuf,
    existed: bool,
}

fn apply_atomic_batch(project_root: &Path, changes: &[PlannedChange]) -> Result<()> {
    let mut temp_paths = Vec::new();
    for change in changes {
        if let PlannedChange::Write { rel_path, body } = change {
            let final_path = project_root.join(rel_path);
            let temp_path = temp_path_for(&final_path);
            write_temp_file(&temp_path, body)?;
            temp_paths.push(temp_path);
        }
    }

    let mut backups = Vec::new();
    let result = (|| -> Result<()> {
        for change in changes {
            let rel_path = match change {
                PlannedChange::Write { rel_path, .. } | PlannedChange::Remove { rel_path } => {
                    rel_path
                }
            };
            let final_path = project_root.join(rel_path);
            let backup_path = backup_path_for(&final_path);
            if final_path.exists() {
                fs::rename(&final_path, &backup_path)?;
                backups.push(Backup {
                    final_path,
                    backup_path,
                    existed: true,
                });
            } else {
                backups.push(Backup {
                    final_path,
                    backup_path,
                    existed: false,
                });
            }
        }

        for change in changes {
            match change {
                PlannedChange::Write { rel_path, .. } => {
                    let final_path = project_root.join(rel_path);
                    fs::rename(temp_path_for(&final_path), final_path)?;
                }
                PlannedChange::Remove { .. } => {}
            }
        }

        for backup in &backups {
            if backup.existed {
                remove_path_if_exists(&backup.backup_path)?;
            }
        }

        sync_parent_dirs(project_root, changes)?;
        Ok(())
    })();

    if let Err(err) = result {
        for change in changes {
            if let PlannedChange::Write { rel_path, .. } = change {
                let _ = remove_path_if_exists(&temp_path_for(&project_root.join(rel_path)));
            }
        }
        rollback_backups(&backups).map_err(|source| SpecEngineError::AtomicRollback {
            operation: err.to_string(),
            source,
        })?;
        return Err(err);
    }

    for temp_path in temp_paths {
        let _ = remove_path_if_exists(&temp_path);
    }

    Ok(())
}

fn rollback_backups(backups: &[Backup]) -> std::io::Result<()> {
    for backup in backups.iter().rev() {
        remove_path_if_exists(&backup.final_path)?;
        if backup.existed {
            fs::rename(&backup.backup_path, &backup.final_path)?;
        }
    }
    Ok(())
}

fn atomic_write(path: &Path, body: &str) -> Result<()> {
    let temp_path = temp_path_for(path);
    write_temp_file(&temp_path, body)?;
    fs::rename(&temp_path, path)?;
    sync_parent_dir(path)?;
    Ok(())
}

fn write_temp_file(path: &Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    file.write_all(body.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

fn sync_parent_dirs(project_root: &Path, changes: &[PlannedChange]) -> Result<()> {
    for change in changes {
        let rel_path = match change {
            PlannedChange::Write { rel_path, .. } | PlannedChange::Remove { rel_path } => rel_path,
        };
        sync_parent_dir(&project_root.join(rel_path))?;
    }
    Ok(())
}

fn sync_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        let dir = File::open(parent)?;
        dir.sync_all()?;
    }
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> std::io::Result<()> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn temp_path_for(path: &Path) -> PathBuf {
    path.with_extension(match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => format!("{ext}.tmp"),
        None => "tmp".to_owned(),
    })
}

fn backup_path_for(path: &Path) -> PathBuf {
    path.with_extension(match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => format!("{ext}.bak"),
        None => "bak".to_owned(),
    })
}

fn parse_modules_table(body: &str) -> Result<Vec<ModuleEntry>> {
    let mut raw_entries = Vec::new();
    for line in body.lines() {
        let cells = markdown_table_cells(line);
        if cells.len() < 5 || cells[0] == "#" || cells[0].starts_with("---") {
            continue;
        }

        let name = strip_markdown(&cells[1]);
        if slug::validate_module_slug(&name).is_err() {
            continue;
        }

        raw_entries.push((name, cells[2].to_owned(), cells[4].to_owned()));
    }

    let known_slugs = raw_entries
        .iter()
        .map(|(name, _, _)| name.clone())
        .collect::<Vec<_>>();

    raw_entries
        .into_iter()
        .map(|(name, layer, depends_on_cell)| {
            let mut depends_on = Vec::new();
            for slug in &known_slugs {
                if slug == &name {
                    continue;
                }
                let re = slug::slug_boundary_regex(slug)?;
                if re.is_match(&depends_on_cell) {
                    depends_on.push(slug.clone());
                }
            }

            Ok(ModuleEntry {
                name,
                depends_on,
                layer: strip_markdown(&layer),
            })
        })
        .collect()
}

fn markdown_table_cells(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
        return Vec::new();
    }

    trimmed
        .trim_matches('|')
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect()
}

fn strip_markdown(value: &str) -> String {
    value
        .trim()
        .trim_matches('*')
        .trim()
        .trim_matches('`')
        .trim()
        .to_owned()
}

fn path_for_snapshot(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn parallel_yaml_to_string(value: &ParallelYaml) -> String {
    let mut body = String::new();
    body.push_str(&format!("version: {}\n", value.version));
    push_yaml_string_array(&mut body, "touches", &value.touches);
    push_yaml_string_array(&mut body, "shared", &value.shared);
    push_yaml_u16_array(&mut body, "ports", &value.ports);
    body.push_str(&format!("migrations: {}\n", value.migrations));
    body
}

fn push_yaml_string_array(body: &mut String, key: &str, values: &[String]) {
    if values.is_empty() {
        body.push_str(&format!("{key}: []\n"));
        return;
    }

    body.push_str(&format!("{key}:\n"));
    for value in values {
        body.push_str("  - ");
        body.push_str(value);
        body.push('\n');
    }
}

fn push_yaml_u16_array(body: &mut String, key: &str, values: &[u16]) {
    if values.is_empty() {
        body.push_str(&format!("{key}: []\n"));
        return;
    }

    body.push_str(&format!("{key}:\n"));
    for value in values {
        body.push_str(&format!("  - {value}\n"));
    }
}

fn parse_parallel_yaml(yaml: &str) -> Result<ParallelYaml> {
    let mut version = None;
    let mut touches = Vec::new();
    let mut shared = Vec::new();
    let mut ports = Vec::new();
    let mut migrations = false;
    let mut current_array: Option<&str> = None;

    for (index, raw_line) in yaml.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(item) = line.strip_prefix("- ") {
            match current_array {
                Some("touches") => touches.push(parse_string_item(item)),
                Some("shared") => shared.push(parse_string_item(item)),
                Some("ports") => ports.push(parse_port(item, index + 1)?),
                Some(key) => {
                    return Err(SpecEngineError::ParallelSchemaValidation {
                        messages: format!("'{key}' does not accept sequence items"),
                    })
                }
                None => {
                    return Err(SpecEngineError::ParallelParse {
                        message: format!("line {} has a sequence item without a key", index + 1),
                    })
                }
            }
            continue;
        }

        let Some((key, value)) = line.split_once(':') else {
            return Err(SpecEngineError::ParallelParse {
                message: format!("line {} is not a key/value entry", index + 1),
            });
        };

        let key = key.trim();
        let value = value.trim();
        current_array = None;

        match key {
            "version" => {
                version = Some(parse_version(value)?);
            }
            "touches" => {
                touches = parse_inline_string_array(value, index + 1)?;
                if value.is_empty() {
                    current_array = Some("touches");
                }
            }
            "shared" => {
                shared = parse_inline_string_array(value, index + 1)?;
                if value.is_empty() {
                    current_array = Some("shared");
                }
            }
            "ports" => {
                ports = parse_inline_ports(value, index + 1)?;
                if value.is_empty() {
                    current_array = Some("ports");
                }
            }
            "migrations" => {
                migrations = parse_bool(value, index + 1)?;
            }
            other => {
                return Err(SpecEngineError::ParallelSchemaValidation {
                    messages: format!("unexpected property '{other}'"),
                })
            }
        }
    }

    let Some(version) = version else {
        return Err(SpecEngineError::MissingParallelVersion);
    };
    if version != 1 {
        return Err(SpecEngineError::UnsupportedParallelVersion {
            found: version.to_string(),
        });
    }

    Ok(ParallelYaml {
        version,
        touches,
        shared,
        ports,
        migrations,
    })
}

fn parse_version(value: &str) -> Result<u32> {
    value
        .parse::<u32>()
        .map_err(|_| SpecEngineError::UnsupportedParallelVersion {
            found: value.to_owned(),
        })
}

fn parse_bool(value: &str, line: usize) -> Result<bool> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(SpecEngineError::ParallelParse {
            message: format!("line {line} has invalid boolean '{value}'"),
        }),
    }
}

fn parse_inline_string_array(value: &str, line: usize) -> Result<Vec<String>> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    parse_inline_array(value, line).map(|items| {
        items
            .into_iter()
            .map(|item| parse_string_item(&item))
            .collect()
    })
}

fn parse_inline_ports(value: &str, line: usize) -> Result<Vec<u16>> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    parse_inline_array(value, line)?
        .into_iter()
        .map(|item| parse_port(&item, line))
        .collect()
}

fn parse_inline_array(value: &str, line: usize) -> Result<Vec<String>> {
    let Some(inner) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) else {
        return Err(SpecEngineError::ParallelParse {
            message: format!("line {line} expected an array"),
        });
    };

    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }

    Ok(inner
        .split(',')
        .map(|item| item.trim().to_owned())
        .collect())
}

fn parse_string_item(value: &str) -> String {
    value.trim().trim_matches('"').trim_matches('\'').to_owned()
}

fn parse_port(value: &str, line: usize) -> Result<u16> {
    value
        .trim()
        .parse::<u16>()
        .map_err(|_| SpecEngineError::ParallelParse {
            message: format!("line {line} has invalid port '{value}'"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn public_types_are_send_sync() {
        assert_send_sync::<SpecEngine>();
        assert_send_sync::<ProjectBrief>();
        assert_send_sync::<MasterBlueprint>();
        assert_send_sync::<Roadmap>();
        assert_send_sync::<ParallelYaml>();
        assert_send_sync::<ModuleSpec>();
        assert_send_sync::<ModuleEntry>();
    }

    #[test]
    fn parse_modules_uses_slug_boundaries_for_dependencies() {
        let body = r#"
| # | Name | Layer | Language | Depends on |
|---|---|---|---|---|
| 1 | **auth** | Core | Rust | — |
| 2 | **oauth** | Core | Rust | auth |
| 3 | **api** | Core | Rust | oauth |
"#;
        let entries = parse_modules_table(body).unwrap();
        assert_eq!(entries[1].depends_on, vec!["auth"]);
        assert_eq!(entries[2].depends_on, vec!["oauth"]);
    }
}
