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
    fn test_cli_parse_fetch_subcommand() {
        let cli = Cli::try_parse_from([
            "bce",
            "fetch",
            "alice.bsky.social",
            "--db",
            "/tmp/test.db",
            "--since",
            "2025-01-01",
            "-q",
        ])
        .unwrap();

        assert!(!cli.agent_help);
        match cli.command {
            Some(Command::Fetch(args)) => {
                assert_eq!(args.handle, "alice.bsky.social");
                assert_eq!(args.db, Some(std::path::PathBuf::from("/tmp/test.db")));
                assert_eq!(args.since, Some("2025-01-01".to_string()));
                assert!(args.quiet);
            }
            Some(Command::Query(_)) => panic!("expected Fetch subcommand, got Query"),
            None => panic!("expected Fetch subcommand, got no subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_query_defaults() {
        let cli = Cli::try_parse_from(["bce", "query"]).unwrap();

        assert!(!cli.agent_help);
        match cli.command {
            Some(Command::Query(args)) => {
                assert!(args.db.is_none());
                assert_eq!(args.limit, 50);
                assert_eq!(args.offset, 0);
            }
            Some(Command::Fetch(_)) => panic!("expected Query subcommand, got Fetch"),
            None => panic!("expected Query subcommand, got no subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_query_overrides() {
        let cli = Cli::try_parse_from([
            "bce",
            "query",
            "--db",
            "/tmp/query.db",
            "--limit",
            "25",
            "--offset",
            "75",
        ])
        .unwrap();

        match cli.command {
            Some(Command::Query(args)) => {
                assert_eq!(args.db, Some(std::path::PathBuf::from("/tmp/query.db")));
                assert_eq!(args.limit, 25);
                assert_eq!(args.offset, 75);
            }
            Some(Command::Fetch(_)) => panic!("expected Query subcommand, got Fetch"),
            None => panic!("expected Query subcommand, got no subcommand"),
        }
    }

    #[test]
    fn test_cli_parse_top_level_agent_help() {
        let cli = Cli::try_parse_from(["bce", "--agent-help"]).unwrap();

        assert!(cli.agent_help);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_parse_flat_invocation_fails() {
        assert!(Cli::try_parse_from(["bce", "alice.bsky.social"]).is_err());
    }

    #[test]
    fn test_cli_parse_query_rejects_since_flag() {
        assert!(Cli::try_parse_from(["bce", "query", "--since", "2025-01-01"]).is_err());
    }
}
