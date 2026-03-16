//! CLI argument definitions.

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Known agent targets.
#[derive(Debug, Clone, ValueEnum)]
pub enum Agent {
    /// Anthropic Claude Code
    Claude,
    /// `OpenAI` Codex
    Codex,
    /// Google Gemini CLI
    Gemini,
}

/// Agent sandbox harness.
///
/// Wraps coding agents with macOS sandbox-exec and prompter integration.
/// Profiles are prompter profile names (e.g., `rust.full`, `python.full`).
/// Base profiles `core.baseline`, `core.agent`, `core.git` are always included.
#[derive(Parser, Debug)]
#[command(name = "gator", version, about)]
pub struct Cli {
    /// Agent to run
    #[arg(value_enum)]
    pub agent: Agent,

    /// Prompter profiles to compose (validated against `prompter list`)
    #[arg(trailing_var_arg = false)]
    pub profiles: Vec<String>,

    /// Explicit working directory (default: git root or pwd)
    #[arg(long, value_name = "PATH")]
    pub workdir: Option<PathBuf>,

    /// Extra read-write directory grant (repeatable)
    #[arg(long = "add-dirs", value_name = "PATH")]
    pub add_dirs: Vec<PathBuf>,

    /// Extra read-only directory grant (repeatable)
    #[arg(long = "add-dirs-ro", value_name = "PATH")]
    pub add_dirs_ro: Vec<PathBuf>,

    /// Skip prompter integration
    #[arg(long)]
    pub no_prompt: bool,

    /// Print assembled policy to stderr without executing
    #[arg(long)]
    pub dry_run: bool,

    /// JSON output for errors
    #[arg(long, global = true)]
    pub json: bool,

    /// Arguments forwarded to the agent command (after --)
    #[arg(last = true)]
    pub agent_args: Vec<String>,
}

impl std::fmt::Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
            Self::Gemini => write!(f, "gemini"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_minimal() {
        let cli = Cli::parse_from(["gator", "claude"]);
        assert!(matches!(cli.agent, Agent::Claude));
        assert!(cli.profiles.is_empty());
        assert!(cli.agent_args.is_empty());
    }

    #[test]
    fn parse_with_profiles() {
        let cli = Cli::parse_from(["gator", "claude", "rust.full", "python.full"]);
        assert_eq!(cli.profiles, vec!["rust.full", "python.full"]);
    }

    #[test]
    fn parse_with_flags() {
        let cli = Cli::parse_from([
            "gator",
            "codex",
            "--workdir=/tmp/project",
            "--add-dirs=/tmp/extra",
            "--add-dirs-ro=/tmp/readonly",
        ]);
        assert!(matches!(cli.agent, Agent::Codex));
        assert_eq!(cli.workdir, Some(PathBuf::from("/tmp/project")));
        assert_eq!(cli.add_dirs, vec![PathBuf::from("/tmp/extra")]);
        assert_eq!(cli.add_dirs_ro, vec![PathBuf::from("/tmp/readonly")]);
    }

    #[test]
    fn parse_agent_args_after_separator() {
        let cli = Cli::parse_from([
            "gator",
            "claude",
            "rust.full",
            "--",
            "--model",
            "opus",
        ]);
        assert_eq!(cli.profiles, vec!["rust.full"]);
        assert_eq!(cli.agent_args, vec!["--model", "opus"]);
    }

    #[test]
    fn parse_dry_run() {
        let cli = Cli::parse_from(["gator", "gemini", "--dry-run"]);
        assert!(cli.dry_run);
    }

    #[test]
    fn parse_multiple_add_dirs() {
        let cli = Cli::parse_from([
            "gator",
            "claude",
            "--add-dirs=/a",
            "--add-dirs=/b",
            "--add-dirs-ro=/c",
        ]);
        assert_eq!(cli.add_dirs.len(), 2);
        assert_eq!(cli.add_dirs_ro.len(), 1);
    }
}
