//! Shared JSON response helpers.

use serde_json::{Value, json};

/// Build a success response envelope.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn ok_response(command: &str, data: Value) -> Value {
    json!({
        "ok": true,
        "command": command,
        "data": data
    })
}

/// Build an error response envelope.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn err_response(command: &str, code: &str, message: &str, details: Value) -> Value {
    json!({
        "ok": false,
        "command": command,
        "error": {
            "code": code,
            "message": message,
            "details": details
        }
    })
}

/// Render either the shared JSON envelope or plain text for a command response.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn render_response(
    command: &str,
    json_output: bool,
    data: Value,
    text: impl Into<String>,
) -> String {
    render_response_parts(command, json_output, || data, || text.into())
}

/// Render either the shared JSON envelope or lazily-built plain text for a command response.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn render_response_with<F>(command: &str, json_output: bool, data: Value, text: F) -> String
where
    F: FnOnce() -> String,
{
    render_response_parts(command, json_output, || data, text)
}

/// Render either the shared JSON envelope or lazily-built command data and plain text.
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn render_response_parts<D, T>(command: &str, json_output: bool, data: D, text: T) -> String
where
    D: FnOnce() -> Value,
    T: FnOnce() -> String,
{
    if json_output {
        ok_response(command, data()).to_string()
    } else {
        text()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use serde_json::json;

    use super::*;

    #[test]
    fn ok_response_contains_expected_shape() {
        let value = ok_response("list", json!({ "x": 1 }));
        assert_eq!(value["ok"], json!(true));
        assert_eq!(value["command"], json!("list"));
        assert_eq!(value["data"]["x"], json!(1));
    }

    #[test]
    fn err_response_contains_expected_shape() {
        let value = err_response("list", "ERROR", "bad", json!({}));
        assert_eq!(value["ok"], json!(false));
        assert_eq!(value["error"]["code"], json!("ERROR"));
        assert_eq!(value["error"]["message"], json!("bad"));
    }

    #[test]
    fn render_response_uses_json_envelope_when_requested() {
        let value = render_response("list", true, json!({"x": 1}), "text");
        assert!(value.contains("\"ok\":true"));
    }

    #[test]
    fn render_response_with_skips_text_builder_for_json_output() {
        let called = Cell::new(false);
        let value = render_response_with("list", true, json!({"x": 1}), || {
            called.set(true);
            String::from("text")
        });

        assert!(value.contains("\"ok\":true"));
        assert!(!called.get());
    }

    #[test]
    fn render_response_with_builds_text_for_text_output() {
        let value = render_response_with("list", false, json!({"x": 1}), || String::from("text"));
        assert_eq!(value, "text");
    }

    #[test]
    fn render_response_parts_skips_text_builder_for_json_output() {
        let called = Cell::new(false);
        let value = render_response_parts(
            "list",
            true,
            || json!({"x": 1}),
            || {
                called.set(true);
                String::from("text")
            },
        );

        assert!(value.contains("\"ok\":true"));
        assert!(!called.get());
    }

    #[test]
    fn render_response_parts_skips_data_builder_for_text_output() {
        let called = Cell::new(false);
        let value = render_response_parts(
            "list",
            false,
            || {
                called.set(true);
                json!({"x": 1})
            },
            || String::from("text"),
        );

        assert_eq!(value, "text");
        assert!(!called.get());
    }
}
