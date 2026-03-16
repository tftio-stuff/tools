//! Gator CLI entrypoint.

use clap::Parser;
use gator::cli::Cli;

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    if let Err(e) = cli.validate() {
        if json {
            eprintln!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\""));
        } else {
            eprintln!("gator: {e}");
        }
        std::process::exit(1);
    }

    if let Err(e) = gator::run(&cli) {
        if json {
            eprintln!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\""));
        } else {
            eprintln!("gator: {e}");
        }
        std::process::exit(1);
    }
}
