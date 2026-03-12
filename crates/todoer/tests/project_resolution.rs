use todoer::project::{resolve_project, resolve_init_project};
use std::fs;
use tempfile::tempdir;

#[test]
fn resolves_project_from_todoer_file() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("repo");
    let sub = root.join("a");
    fs::create_dir_all(&sub).unwrap();
    fs::write(root.join(".todoer.toml"), "project = \"Name\"").unwrap();

    let proj = resolve_project(None, None, &sub, &root, None).unwrap();
    assert_eq!(proj.name, "Name");
}

#[test]
fn init_falls_back_to_git_name() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("repo");
    fs::create_dir_all(&root).unwrap();

    let proj = resolve_init_project(None, &root, &root, Some("gitrepo")).unwrap();
    assert_eq!(proj.name, "gitrepo");
}

#[test]
fn non_init_falls_back_to_git_name_when_no_todoer() {
    let dir = tempdir().unwrap();
    let root = dir.path().join("repo");
    fs::create_dir_all(&root).unwrap();

    let proj = resolve_project(None, None, &root, &root, Some("gitrepo")).unwrap();
    assert_eq!(proj.name, "gitrepo");
}
