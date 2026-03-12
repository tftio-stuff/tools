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

pub fn render_task_table(tasks: &[crate::models::Task]) -> String {
    let mut out = String::new();
    out.push_str("UUID\tSTATUS\tCREATED_AT\tDESCRIPTION\n");
    for t in tasks {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            t.id,
            t.status.as_str(),
            t.created_at,
            t.description
        ));
    }
    out
}
