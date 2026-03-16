//! Gator CLI entrypoint.

use clap::Parser;
use gator::cli::Cli;

fn main() {
    let _cli = Cli::parse();
    // Dispatch will be implemented in Task 7
    eprintln!("gator: not yet implemented");
    std::process::exit(1);
}
