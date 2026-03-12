use crate::config::{Config, resolve_db_path};
use crate::db::open_db;
use crate::repo::{get_task_status, update_task_status, add_note, get_task_with_notes};
use crate::models::{Status, TaskNote};

pub struct StatusResult {
    pub description: String,
    pub status: Status,
    pub created_at: String,
}

pub struct UpdateStatusResult {
    pub status: Status,
}

pub struct NoteResult {
    pub note: TaskNote,
}

pub struct ShowResult {
    pub description: String,
    pub status: Status,
    pub created_at: String,
    pub notes: Vec<TaskNote>,
}

fn open_conn(config: &Config) -> anyhow::Result<rusqlite::Connection> {
    let db_path = resolve_db_path(config)?;
    if !db_path.exists() {
        anyhow::bail!("database not initialized");
    }
    Ok(open_db(&db_path)?)
}

pub fn run_status(config: &Config, id: &str) -> anyhow::Result<StatusResult> {
    let conn = open_conn(config)?;
    let (desc, status, created_at) = get_task_status(&conn, id)?;
    Ok(StatusResult { description: desc, status, created_at })
}

pub fn run_update_status(config: &Config, id: &str, status: Status) -> anyhow::Result<UpdateStatusResult> {
    let conn = open_conn(config)?;
    update_task_status(&conn, id, status.clone())?;
    Ok(UpdateStatusResult { status })
}

pub fn run_note(config: &Config, id: &str, note: &str) -> anyhow::Result<NoteResult> {
    let conn = open_conn(config)?;
    let note = add_note(&conn, id, note)?;
    Ok(NoteResult { note })
}

pub fn run_show(config: &Config, id: &str) -> anyhow::Result<ShowResult> {
    let conn = open_conn(config)?;
    let (desc, status, created_at, notes) = get_task_with_notes(&conn, id)?;
    Ok(ShowResult { description: desc, status, created_at, notes })
}
