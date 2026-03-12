use todoer::input::resolve_input;

#[test]
fn resolve_input_uses_arg_when_not_dash() {
    let out = resolve_input("hello", None).unwrap();
    assert_eq!(out, "hello");
}

#[test]
fn resolve_input_uses_stdin_when_dash() {
    let out = resolve_input("-", Some("stdin".to_string())).unwrap();
    assert_eq!(out, "stdin");
}
