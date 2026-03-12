use todoer::output::render_task_table;
use todoer::models::{Task, Status};

#[test]
fn table_renders_header_and_row() {
    let tasks = vec![Task {
        id: "id1".to_string(),
        project_key: "proj".to_string(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        description: "hello".to_string(),
        status: Status::New,
    }];
    let out = render_task_table(&tasks);
    assert!(out.contains("UUID"));
    assert!(out.contains("id1"));
}
