use serde_json::{Value, json};

pub fn ok_response(command: &str, data: Value) -> Value {
    json!({
        "ok": true,
        "command": command,
        "data": data
    })
}

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
