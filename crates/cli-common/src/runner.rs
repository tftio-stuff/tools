//! Shared CLI runner helpers.

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
}
