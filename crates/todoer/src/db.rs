use rusqlite::{Connection, Result};
use std::path::Path;

pub fn open_db(path: &Path) -> Result<Connection> {
    Connection::open(path)
}

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS projects (
            name_key TEXT PRIMARY KEY,
            name TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            project_key TEXT NOT NULL,
            created_at TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL,
            FOREIGN KEY(project_key) REFERENCES projects(name_key)
        );
        CREATE TABLE IF NOT EXISTS task_notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            created_at TEXT NOT NULL,
            note TEXT NOT NULL,
            FOREIGN KEY(task_id) REFERENCES tasks(id)
        );
        "
    )
}
