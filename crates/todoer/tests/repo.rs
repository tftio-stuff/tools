use todoer::repo::{ensure_project, insert_task, list_tasks_by_project, add_note, get_task_status, update_task_status, get_task_with_notes};
use todoer::models::Status;
use todoer::db::{open_db, init_db};
use tempfile::tempdir;

#[test]
fn insert_and_list_task() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("todoer.db");
    let conn = open_db(&db_path).unwrap();
    init_db(&conn).unwrap();

    ensure_project(&conn, "proj", "Project").unwrap();
    let task = insert_task(&conn, "proj", "do thing").unwrap();

    let tasks = list_tasks_by_project(&conn, "proj").unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, task.id);
}

#[test]
fn update_status_and_notes() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("todoer.db");
    let conn = open_db(&db_path).unwrap();
    init_db(&conn).unwrap();

    ensure_project(&conn, "proj", "Project").unwrap();
    let task = insert_task(&conn, "proj", "do thing").unwrap();

    update_task_status(&conn, &task.id, Status::Completed).unwrap();
    let status = get_task_status(&conn, &task.id).unwrap();
    assert_eq!(status.1, Status::Completed);

    add_note(&conn, &task.id, "note 1").unwrap();
    add_note(&conn, &task.id, "note 2").unwrap();

    let show = get_task_with_notes(&conn, &task.id).unwrap();
    assert_eq!(show.3.len(), 2);
    assert_eq!(show.3[0].note, "note 2");
}
