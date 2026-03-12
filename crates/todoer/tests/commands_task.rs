use tempfile::tempdir;
use todoer::commands::task::{run_status, run_update_status, run_note, run_show};
use todoer::config::Config;
use todoer::db::{open_db, init_db};
use todoer::repo::{ensure_project, insert_task};
use todoer::project::ResolvedProject;
use todoer::models::Status;

#[test]
fn status_and_update() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("todoer.db");
    let config = Config { db_path: Some(db.to_string_lossy().to_string()) };
    let project = ResolvedProject { name: "Test".to_string(), key: "test".to_string() };

    let conn = open_db(&db).unwrap();
    init_db(&conn).unwrap();
    ensure_project(&conn, &project.key, &project.name).unwrap();
    let task = insert_task(&conn, &project.key, "t1").unwrap();

    let status = run_status(&config, &task.id).unwrap();
    assert_eq!(status.description, "t1");

    let updated = run_update_status(&config, &task.id, Status::Completed).unwrap();
    assert_eq!(updated.status, Status::Completed);
}

#[test]
fn note_and_show() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("todoer.db");
    let config = Config { db_path: Some(db.to_string_lossy().to_string()) };
    let project = ResolvedProject { name: "Test".to_string(), key: "test".to_string() };

    let conn = open_db(&db).unwrap();
    init_db(&conn).unwrap();
    ensure_project(&conn, &project.key, &project.name).unwrap();
    let task = insert_task(&conn, &project.key, "t1").unwrap();

    run_note(&config, &task.id, "note 1").unwrap();
    run_note(&config, &task.id, "note 2").unwrap();

    let show = run_show(&config, &task.id).unwrap();
    assert_eq!(show.notes.len(), 2);
    assert_eq!(show.notes[0].note, "note 2");
}
