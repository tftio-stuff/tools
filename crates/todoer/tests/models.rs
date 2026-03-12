use todoer::models::{Status, normalize_project_key};

#[test]
fn normalize_project_key_lowercases_and_trims() {
    let key = normalize_project_key("  My Project  ");
    assert_eq!(key, "my project");
}

#[test]
fn status_string_roundtrip() {
    let s = Status::InProgress;
    assert_eq!(s.as_str(), "IN-PROGRESS");
}

#[test]
fn status_serializes_as_string() {
    let s = Status::InProgress;
    let v = serde_json::to_string(&s).unwrap();
    assert_eq!(v, "\"IN-PROGRESS\"");
}
