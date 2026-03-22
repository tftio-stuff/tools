//! Shared CLI runner helpers.

use crate::err_response;
use serde_json::json;

/// Shared fatal CLI error state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FatalCliError {
    command: String,
    json_output: bool,
    message: String,
}

impl FatalCliError {
    /// Create a fatal CLI error with the shared renderer contract.
    #[must_use]
    pub fn new(
        command: impl Into<String>,
        json_output: bool,
        message: impl Into<String>,
    ) -> Self {
        Self {
            command: command.into(),
            json_output,
            message: message.into(),
        }
    }

    /// Return the command label used in shared error output.
    #[must_use]
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Return whether the error should emit JSON.
    #[must_use]
    pub fn json_output(&self) -> bool {
        self.json_output
    }

    /// Return the fatal message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Render the fatal error as a string without emitting it.
    #[must_use]
    pub fn render(&self) -> String {
        if self.json_output {
            err_response(self.command(), "ERROR", self.message(), json!({})).to_string()
        } else {
            format!("error: {}", self.message())
        }
    }

    /// Emit the fatal error to the correct output stream.
    pub fn emit(&self) {
        if self.json_output {
            println!("{}", self.render());
        } else {
            eprintln!("{}", self.render());
        }
    }

    /// Emit the fatal error and return the shared failure exit code.
    #[must_use]
    pub fn emit_and_exit_code(self) -> i32 {
        self.emit();
        1
    }
}

/// Run a fallible CLI closure and convert shared fatal errors into exit codes.
#[must_use]
pub fn run_with_fatal_handler<F>(run: F) -> i32
where
    F: FnOnce() -> Result<i32, FatalCliError>,
{
    match run() {
        Ok(exit_code) => exit_code,
        Err(error) => error.emit_and_exit_code(),
    }
}

/// Parse CLI state with one closure and execute it with another.
#[must_use]
pub fn parse_and_run<T, P, F>(parse: P, run: F) -> i32
where
    P: FnOnce() -> T,
    F: FnOnce(T) -> Result<i32, FatalCliError>,
{
    run_with_fatal_handler(|| run(parse()))
}

/// Parse CLI state, run the handler, and exit the process with the resulting code.
pub fn parse_and_exit<T, P, F>(parse: P, run: F) -> !
where
    P: FnOnce() -> T,
    F: FnOnce(T) -> Result<i32, FatalCliError>,
{
    std::process::exit(parse_and_run(parse, run))
}

#[cfg(test)]
mod tests {
    use crate::error::fatal_error;

    use super::*;

    #[test]
    fn run_with_fatal_handler_returns_success_code() {
        let exit_code = run_with_fatal_handler(|| Ok(7));
        assert_eq!(exit_code, 7);
    }

    #[test]
    fn run_with_fatal_handler_converts_fatal_error_to_failure_code() {
        let exit_code = run_with_fatal_handler(|| Err(fatal_error("scan", false, "bad")));
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn parse_and_run_passes_parsed_value_to_runner() {
        let exit_code = parse_and_run(|| String::from("parsed"), |cli| {
            if cli == "parsed" {
                Ok(0)
            } else {
                Err(fatal_error("scan", false, "unexpected cli"))
            }
        });
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn fatal_cli_error_renders_json_when_requested() {
        let rendered = FatalCliError::new("scan", true, "bad").render();
        assert!(rendered.contains("\"ok\":false"));
        assert!(rendered.contains("\"code\":\"ERROR\""));
        assert!(rendered.contains("\"command\":\"scan\""));
    }
}
