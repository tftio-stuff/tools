//! Binary entry point for the Asana CLI.

use asana_cli::{cli, init_tracing};
use tftio_cli_common::run_with_display_error_handler;

fn main() {
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
