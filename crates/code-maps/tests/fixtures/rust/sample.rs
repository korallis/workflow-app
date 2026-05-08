use std::path::Path;

/// Loads a project from disk.
pub async fn load_project(path: &Path) -> Result<Project, Error> {
    Project::open(path).await
}

fn helper(count: usize) -> usize {
    count + 1
}

/// A project record.
pub struct Project {
    pub name: String,
}

pub type ProjectId = String;

const DEFAULT_LIMIT: usize = 25;
