use tempfile::tempdir;
use todoer::commands::init::run_init;
use todoer::config::Config;
use todoer::project::ResolvedProject;

#[test]
fn init_creates_db() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("todoer.db");
    let config = Config { db_path: Some(db.to_string_lossy().to_string()) };
    let project = ResolvedProject { name: "Test".to_string(), key: "test".to_string() };

    let result = run_init(&config, &project).unwrap();
    assert!(db.exists());
    assert!(result.schema_created);
}
