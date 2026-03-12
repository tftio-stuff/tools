use todoer::db::{init_db, open_db};
use tempfile::tempdir;

#[test]
fn init_creates_schema() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("todoer.db");
    let conn = open_db(&db_path).unwrap();
    init_db(&conn).unwrap();

    let count: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='tasks'",
        [],
        |row| row.get(0),
    ).unwrap();
    assert_eq!(count, 1);
}
