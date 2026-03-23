//! Binary entry point for the Asana CLI.

use asana_cli::{cli, init_tracing};
use tftio_cli_common::detect_agent_doc_request;

fn main() {
    if let Some(request) = detect_agent_doc_request(std::env::args_os()) {
        cli::run_agent_doc_request(request);
        return;
    }

    if let Err(err) = init_tracing() {
        eprintln!("failed to initialize tracing: {err}");
    }

    match cli::run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            tracing::error!(error = %err, "command execution failed");
            eprintln!("{err:?}");
            std::process::exit(1);
        }
    }
}
