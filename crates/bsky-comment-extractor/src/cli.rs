//! Command-line argument definitions for the `bce` binary.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Extract a `BlueSky` user's complete post history to a local `SQLite` database.
#[derive(Parser, Debug)]
#[command(name = "bce")]
#[command(version)]
#[command(about = "Extract a BlueSky user's post history to SQLite")]
#[command(subcommand_negates_reqs = true)]
#[command(after_help = "\
CREDENTIALS:
  Set BSKY_APP_PASSWORD before running.
  Create an app password at https://bsky.app/settings/app-passwords

EXAMPLES:
  bce alice.bsky.social
  bce alice.bsky.social --since '3 months ago'
  bce did:plc:abc123 --db /tmp/posts.db")]
pub struct Cli {
    /// Shared metadata subcommands.
    #[command(subcommand)]
    pub command: Option<Command>,

    /// `BlueSky` handle (e.g. alice.bsky.social) or DID (e.g. did:plc:abc123).
    #[arg(required = true)]
    pub handle: Option<String>,

    /// Path to the `SQLite` database file.
    ///
    /// Defaults to `~/.local/share/bce/bsky-posts.db` (XDG data directory).
    #[arg(long, value_name = "PATH")]
    pub db: Option<PathBuf>,

    /// Only extract posts created after this date.
    ///
    /// Accepts human-friendly dates like "2025-01-01", "3 months ago", "yesterday".
    #[arg(long, value_name = "DATE")]
    pub since: Option<String>,

    /// Suppress the progress spinner.
    #[arg(short, long)]
    pub quiet: bool,
}

/// Shared metadata commands exposed by `bce`.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Show version information.
    Version,
    /// Show license information.
    License,
    /// Generate shell completion scripts.
    Completions {
        /// Shell to generate completions for.
        shell: clap_complete::Shell,
    },
    /// Run health checks.
    Doctor,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parse_handle_only() {
        let cli = Cli::try_parse_from(["bce", "alice.bsky.social"]).unwrap();
        assert_eq!(cli.handle.as_deref(), Some("alice.bsky.social"));
        assert!(cli.db.is_none());
        assert!(cli.since.is_none());
        assert!(!cli.quiet);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_parse_all_flags() {
        let cli = Cli::try_parse_from([
            "bce",
            "alice.bsky.social",
            "--db",
            "/tmp/test.db",
            "--since",
            "2025-01-01",
            "-q",
        ])
        .unwrap();
        assert_eq!(cli.handle.as_deref(), Some("alice.bsky.social"));
        assert_eq!(cli.db.unwrap(), std::path::PathBuf::from("/tmp/test.db"));
        assert_eq!(cli.since.unwrap(), "2025-01-01");
        assert!(cli.quiet);
    }

    #[test]
    fn test_cli_parse_did_input() {
        let cli = Cli::try_parse_from(["bce", "did:plc:abc123"]).unwrap();
        assert_eq!(cli.handle.as_deref(), Some("did:plc:abc123"));
    }

    #[test]
    fn test_cli_parse_missing_handle_fails() {
        assert!(Cli::try_parse_from(["bce"]).is_err());
    }

    #[test]
    fn test_cli_parse_completions() {
        let cli = Cli::try_parse_from(["bce", "completions", "bash"]).unwrap();
        match cli.command {
            Some(Command::Completions { shell }) => {
                assert_eq!(shell, clap_complete::Shell::Bash);
            }
            _ => panic!("expected completions command"),
        }
    }

    #[test]
    fn test_cli_parse_license() {
        let cli = Cli::try_parse_from(["bce", "license"]).unwrap();
        assert!(matches!(cli.command, Some(Command::License)));
    }
}
