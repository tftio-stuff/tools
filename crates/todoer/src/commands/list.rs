use crate::config::{Config, resolve_db_path};
use crate::project::ResolvedProject;
use crate::db::open_db;
use crate::repo::{list_tasks_by_project, list_tasks_all};
use crate::models::Task;

pub struct ListResult {
    pub tasks: Vec<Task>,
}

pub fn run_list(config: &Config, project: Option<&ResolvedProject>, all: bool) -> anyhow::Result<ListResult> {
    let db_path = resolve_db_path(config)?;
    if !db_path.exists() {
        anyhow::bail!("database not initialized");
    }
    let conn = open_db(&db_path)?;
    let tasks = if all {
        list_tasks_all(&conn)?
    } else {
        let project = project.ok_or_else(|| anyhow::anyhow!("no project specified"))?;
        list_tasks_by_project(&conn, &project.key)?
    };
    Ok(ListResult { tasks })
}
