use std::path::PathBuf;
use rusqlite::Connection;
use crate::config::{Config, resolve_db_path};
use crate::db::init_db;
use crate::repo::ensure_project;
use crate::project::ResolvedProject;

pub struct InitResult {
    pub db_path: PathBuf,
    pub schema_created: bool,
}

pub fn run_init(config: &Config, project: &ResolvedProject) -> anyhow::Result<InitResult> {
    let db_path = resolve_db_path(config)?;
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(&db_path)?;
    init_db(&conn)?;
    ensure_project(&conn, &project.key, &project.name)?;
    Ok(InitResult { db_path, schema_created: true })
}
