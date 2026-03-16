//! CLI argument definitions.

use clap::Parser;

/// Agent sandbox harness.
///
/// Wraps coding agents with macOS sandbox-exec and prompter integration.
#[derive(Parser, Debug)]
#[command(name = "gator", version, about)]
pub struct Cli {
    /// Agent to run (claude, codex, gemini)
    pub agent: String,
}
