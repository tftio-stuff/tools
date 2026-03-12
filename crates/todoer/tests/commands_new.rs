use tempfile::tempdir;
use todoer::commands::new::run_new;
use todoer::config::Config;
use todoer::project::ResolvedProject;
use todoer::db::{open_db, init_db};
use todoer::repo::ensure_project;

#[test]
fn new_creates_task() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("todoer.db");
    let config = Config { db_path: Some(db.to_string_lossy().to_string()) };
    let project = ResolvedProject { name: "Test".to_string(), key: "test".to_string() };

    let conn = open_db(&db).unwrap();
    init_db(&conn).unwrap();
    ensure_project(&conn, &project.key, &project.name).unwrap();

    let result = run_new(&config, &project, "do thing").unwrap();
    assert_eq!(result.task.description, "do thing");
    assert_eq!(result.task.project_key, "test");
}

#[test]
fn new_ensures_project_exists() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("todoer.db");
    let config = Config { db_path: Some(db.to_string_lossy().to_string()) };
    let project = ResolvedProject { name: "Test".to_string(), key: "test".to_string() };

    let conn = open_db(&db).unwrap();
    init_db(&conn).unwrap();

    let result = run_new(&config, &project, "do thing").unwrap();
    assert_eq!(result.task.project_key, "test");
}
