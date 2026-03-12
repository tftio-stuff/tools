//! Binary entry point for the Asana CLI.

use asana_cli::{cli, init_tracing};

fn main() {
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
