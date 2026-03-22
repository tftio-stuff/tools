//! Command-line argument definitions for the `bce` binary.

use clap::Parser;
use std::path::PathBuf;

/// Extract a `BlueSky` user's complete post history to a local `SQLite` database.
#[derive(Parser, Debug)]
#[command(name = "bce")]
#[command(version)]
#[command(about = "Extract a BlueSky user's post history to SQLite")]
#[command(after_help = "\
CREDENTIALS:
  Set BSKY_APP_PASSWORD before running.
  Create an app password at https://bsky.app/settings/app-passwords

EXAMPLES:
  bce alice.bsky.social
  bce alice.bsky.social --since '3 months ago'
  bce did:plc:abc123 --db /tmp/posts.db")]
pub struct Cli {
    /// `BlueSky` handle (e.g. alice.bsky.social) or DID (e.g. did:plc:abc123).
    pub handle: String,

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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parse_handle_only() {
        let cli = Cli::try_parse_from(["bce", "alice.bsky.social"]).unwrap();
        assert_eq!(cli.handle, "alice.bsky.social");
        assert!(cli.db.is_none());
        assert!(cli.since.is_none());
        assert!(!cli.quiet);
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
        assert_eq!(cli.handle, "alice.bsky.social");
        assert_eq!(cli.db.unwrap(), std::path::PathBuf::from("/tmp/test.db"));
        assert_eq!(cli.since.unwrap(), "2025-01-01");
        assert!(cli.quiet);
    }

    #[test]
    fn test_cli_parse_did_input() {
        let cli = Cli::try_parse_from(["bce", "did:plc:abc123"]).unwrap();
        assert_eq!(cli.handle, "did:plc:abc123");
    }

    #[test]
    fn test_cli_parse_missing_handle_fails() {
        assert!(Cli::try_parse_from(["bce"]).is_err());
    }
}
