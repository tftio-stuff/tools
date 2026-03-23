//! Binary entry point for the Asana CLI.

use asana_cli::{cli, init_tracing};
use tftio_cli_common::{detect_agent_doc_request, run_with_display_error_handler};

fn main() {
    if let Some(request) = detect_agent_doc_request(std::env::args_os()) {
        cli::run_agent_doc_request(request);
        return;
    }

    if let Err(err) = init_tracing() {
        eprintln!("failed to initialize tracing: {err}");
    }

    let exit_code = run_with_display_error_handler("asana-cli", false, || {
        cli::run().inspect_err(|err| {
            tracing::error!(error = %err, "command execution failed");
        })
    });
    std::process::exit(exit_code);
}
