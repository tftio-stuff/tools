use todoer::project::find_project_file;
use std::fs;
use std::path::PathBuf;

#[test]
fn finds_todoer_toml_in_parent() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("repo");
    let sub = root.join("a/b/c");
    fs::create_dir_all(&sub).unwrap();
    fs::write(root.join(".todoer.toml"), "project = \"Test\"").unwrap();

    let found = find_project_file(&sub, &root).unwrap().unwrap();
    assert_eq!(found, PathBuf::from(root.join(".todoer.toml")));
}
