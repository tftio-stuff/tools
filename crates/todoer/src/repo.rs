use rusqlite::{Connection, params};
use uuid::Uuid;
use chrono::Utc;
use crate::models::{Task, TaskNote, Status};

pub fn ensure_project(conn: &Connection, key: &str, name: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO projects (name_key, name) VALUES (?1, ?2)",
        params![key, name],
    )?;
    Ok(())
}

pub fn insert_task(conn: &Connection, project_key: &str, description: &str) -> rusqlite::Result<Task> {
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let status = Status::New;
    conn.execute(
        "INSERT INTO tasks (id, project_key, created_at, description, status) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, project_key, created_at, description, status.as_str()],
    )?;
    Ok(Task { id, project_key: project_key.to_string(), created_at, description: description.to_string(), status })
}

pub fn list_tasks_by_project(conn: &Connection, project_key: &str) -> rusqlite::Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_key, created_at, description, status FROM tasks WHERE project_key = ?1 ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map(params![project_key], |row| {
        let status: String = row.get(4)?;
        Ok(Task {
            id: row.get(0)?,
            project_key: row.get(1)?,
            created_at: row.get(2)?,
            description: row.get(3)?,
            status: status.parse().unwrap(),
        })
    })?;
    let mut tasks = Vec::new();
    for t in rows { tasks.push(t?); }
    Ok(tasks)
}

pub fn list_tasks_all(conn: &Connection) -> rusqlite::Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_key, created_at, description, status FROM tasks ORDER BY created_at ASC"
    )?;
    let rows = stmt.query_map([], |row| {
        let status: String = row.get(4)?;
        Ok(Task {
            id: row.get(0)?,
            project_key: row.get(1)?,
            created_at: row.get(2)?,
            description: row.get(3)?,
            status: status.parse().unwrap(),
        })
    })?;
    let mut tasks = Vec::new();
    for t in rows { tasks.push(t?); }
    Ok(tasks)
}

pub fn get_task_status(conn: &Connection, id: &str) -> rusqlite::Result<(String, Status, String)> {
    conn.query_row(
        "SELECT description, status, created_at FROM tasks WHERE id = ?1",
        params![id],
        |row| {
            let status: String = row.get(1)?;
            Ok((row.get(0)?, status.parse().unwrap(), row.get(2)?))
        },
    )
}

pub fn update_task_status(conn: &Connection, id: &str, status: Status) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE tasks SET status = ?1 WHERE id = ?2",
        params![status.as_str(), id],
    )?;
    Ok(())
}

pub fn add_note(conn: &Connection, id: &str, note: &str) -> rusqlite::Result<TaskNote> {
    let created_at = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO task_notes (task_id, created_at, note) VALUES (?1, ?2, ?3)",
        params![id, created_at, note],
    )?;
    let note_id = conn.last_insert_rowid();
    Ok(TaskNote { id: note_id, task_id: id.to_string(), created_at, note: note.to_string() })
}

pub fn get_task_with_notes(conn: &Connection, id: &str) -> rusqlite::Result<(String, Status, String, Vec<TaskNote>)> {
    let (desc, status, created_at) = get_task_status(conn, id)?;
    let mut stmt = conn.prepare(
        "SELECT id, task_id, created_at, note FROM task_notes WHERE task_id = ?1 ORDER BY created_at DESC"
    )?;
    let rows = stmt.query_map(params![id], |row| {
        Ok(TaskNote {
            id: row.get(0)?,
            task_id: row.get(1)?,
            created_at: row.get(2)?,
            note: row.get(3)?,
        })
    })?;
    let mut notes = Vec::new();
    for n in rows { notes.push(n?); }
    Ok((desc, status, created_at, notes))
}
