use tempfile::tempdir;
use todoer::commands::list::run_list;
use todoer::config::Config;
use todoer::project::ResolvedProject;
use todoer::db::{open_db, init_db};
use todoer::repo::{ensure_project, insert_task};

#[test]
fn list_by_project() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("todoer.db");
    let config = Config { db_path: Some(db.to_string_lossy().to_string()) };
    let project = ResolvedProject { name: "Test".to_string(), key: "test".to_string() };

    let conn = open_db(&db).unwrap();
    init_db(&conn).unwrap();
    ensure_project(&conn, &project.key, &project.name).unwrap();
    insert_task(&conn, &project.key, "t1").unwrap();
    insert_task(&conn, &project.key, "t2").unwrap();

    let result = run_list(&config, Some(&project), false).unwrap();
    assert_eq!(result.tasks.len(), 2);
}

#[test]
fn list_all_projects() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("todoer.db");
    let config = Config { db_path: Some(db.to_string_lossy().to_string()) };
    let project1 = ResolvedProject { name: "Test".to_string(), key: "test".to_string() };
    let project2 = ResolvedProject { name: "Other".to_string(), key: "other".to_string() };

    let conn = open_db(&db).unwrap();
    init_db(&conn).unwrap();
    ensure_project(&conn, &project1.key, &project1.name).unwrap();
    ensure_project(&conn, &project2.key, &project2.name).unwrap();
    insert_task(&conn, &project1.key, "t1").unwrap();
    insert_task(&conn, &project2.key, "t2").unwrap();

    let result = run_list(&config, None, true).unwrap();
    assert_eq!(result.tasks.len(), 2);
}
