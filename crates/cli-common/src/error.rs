//! Shared error presentation.

use serde_json::json;

use crate::err_response;

/// Print a standard error response and return a failing exit code.
#[must_use]
pub fn print_error(command: &str, json_output: bool, message: &str) -> i32 {
    if json_output {
        println!("{}", err_response(command, "ERROR", message, json!({})));
    } else {
        eprintln!("error: {message}");
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_error_returns_failure_exit_code() {
        assert_eq!(print_error("list", false, "bad"), 1);
    }

    #[test]
    fn fatal_error_preserves_command_and_json_mode() {
        let error = fatal_error("scan", true, "bad");
        assert_eq!(error.command(), "scan");
        assert!(error.json_output());
    }
}
