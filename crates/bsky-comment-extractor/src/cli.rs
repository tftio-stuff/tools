//! Command-line argument definitions for the `bce` binary.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// Extract and query `BlueSky` posts from a local `SQLite` database.
#[derive(Parser, Debug)]
#[command(name = "bce")]
#[command(version)]
#[command(about = "Extract and query BlueSky posts from a local SQLite database")]
#[command(after_help = "\
CREDENTIALS:
  Set BSKY_APP_PASSWORD before running.
  Create an app password at https://bsky.app/settings/app-passwords

EXAMPLES:
  bce fetch <HANDLE>
  bce fetch alice.bsky.social
  bce fetch alice.bsky.social --since '3 months ago'
  bce query --limit 25 --offset 50")]
pub struct Cli {
    /// Show top-level agent reference help instead of running a subcommand.
    #[arg(long, hide = true)]
    pub agent_help: bool,

    /// Show the top-level Claude skill document instead of running a subcommand.
    #[arg(long, hide = true)]
    pub agent_skill: bool,

    /// Select the networked fetch path or the local read-only query path.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Available `bce` subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Fetch posts from the network into the local database.
    Fetch(FetchArgs),
    /// Query posts from the local database without making network requests.
    Query(QueryArgs),
}

/// Arguments for the networked extractor path.
#[derive(Args, Debug)]
pub struct FetchArgs {
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

/// Arguments for the read-only local query path.
#[derive(Args, Debug)]
pub struct QueryArgs {
    /// Path to the existing `SQLite` database file.
    #[arg(long, value_name = "PATH")]
    pub db: Option<PathBuf>,

    /// Maximum number of posts to return.
    #[arg(long, default_value_t = 50)]
    pub limit: u64,

    /// Number of posts to skip before returning results.
    #[arg(long, default_value_t = 0)]
    pub offset: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{CommandFactory, Parser};

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
        assert!(!cli.agent_skill);
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_parse_top_level_agent_skill() {
        let cli = Cli::try_parse_from(["bce", "--agent-skill"]).unwrap();

        assert!(!cli.agent_help);
        assert!(cli.agent_skill);
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

    #[test]
    fn test_query_help_hides_agent_help_flag() {
        let mut command = Cli::command();
        let query = command
            .find_subcommand_mut("query")
            .expect("query subcommand must exist");
        let mut output = Vec::new();
        query
            .write_long_help(&mut output)
            .expect("query help must render");

        let help = String::from_utf8(output).expect("help must be utf-8");
        assert!(help.contains("--db"));
        assert!(help.contains("--limit"));
        assert!(help.contains("--offset"));
        assert!(!help.contains("--agent-help"));
        assert!(!help.contains("--agent-skill"));
    }

    #[test]
    fn test_subcommand_rejects_top_level_agent_help() {
        assert!(Cli::try_parse_from(["bce", "query", "--agent-help"]).is_err());
    }
}
