//! Shared error presentation.

use crate::runner::FatalCliError;

/// Build a shared fatal CLI error value.
#[must_use]
pub fn fatal_error(command: impl Into<String>, json_output: bool, message: impl Into<String>) -> FatalCliError {
    FatalCliError::new(command, json_output, message)
}

/// Print a standard error response and return a failing exit code.
#[must_use]
pub fn print_error(command: &str, json_output: bool, message: &str) -> i32 {
    fatal_error(command, json_output, message).emit_and_exit_code()
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
