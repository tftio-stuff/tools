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
#[allow(clippy::struct_excessive_bools)]
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

    /// Named policy profile (repeatable). Loaded from
    /// `<workdir>/.gator/policies/<name>.toml` or `~/.config/gator/policies/<name>.toml`
    #[arg(long = "policy", value_name = "NAME")]
    pub policies: Vec<String>,

    /// Silent-critic session ID. When set, the contract is the sole
    /// authority on sandbox grants. Incompatible with `--workdir`,
    /// `--add-dirs`, `--add-dirs-ro`, `--policy`, and `--share-worktrees`.
    #[arg(long, value_name = "ID")]
    pub session: Option<String>,

    /// Grant read-only access to all peer worktrees (disabled by default).
    /// When absent, agents see only their own worktree. Incompatible with
    /// `--session`.
    #[arg(long)]
    pub share_worktrees: bool,

    /// Disable automatic YOLO flag injection (default: inject).
    /// When absent, gator prepends the agent-appropriate autonomous-mode
    /// flag (e.g. `--dangerously-skip-permissions` for Claude).
    /// Incompatible with `--session`.
    #[arg(long)]
    pub no_yolo: bool,

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

impl Cli {
    /// Validate mutual exclusivity of session mode vs non-session flags.
    ///
    /// # Errors
    /// Returns an error if `--session` is combined with incompatible flags.
    pub fn validate(&self) -> Result<(), String> {
        if self.session.is_some() {
            let mut conflicts = Vec::new();
            if self.workdir.is_some() {
                conflicts.push("--workdir");
            }
            if !self.add_dirs.is_empty() {
                conflicts.push("--add-dirs");
            }
            if !self.add_dirs_ro.is_empty() {
                conflicts.push("--add-dirs-ro");
            }
            if !self.policies.is_empty() {
                conflicts.push("--policy");
            }
            if self.share_worktrees {
                conflicts.push("--share-worktrees");
            }
            if self.no_yolo {
                conflicts.push("--no-yolo");
            }
            if !conflicts.is_empty() {
                return Err(format!(
                    "--session is incompatible with: {}",
                    conflicts.join(", ")
                ));
            }
        }
        Ok(())
    }
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
    fn parse_with_policy() {
        let cli = Cli::parse_from(["gator", "claude", "--policy=audit", "--policy=extra"]);
        assert_eq!(cli.policies, vec!["audit", "extra"]);
    }

    #[test]
    fn parse_with_session() {
        let cli = Cli::parse_from(["gator", "claude", "--session=abc-123"]);
        assert_eq!(cli.session, Some("abc-123".to_owned()));
    }

    #[test]
    fn validate_session_exclusive_with_workdir() {
        let cli = Cli::parse_from(["gator", "claude", "--session=abc", "--workdir=/tmp"]);
        assert!(cli.validate().is_err());
    }

    #[test]
    fn validate_session_exclusive_with_policy() {
        let cli = Cli::parse_from(["gator", "claude", "--session=abc", "--policy=audit"]);
        assert!(cli.validate().is_err());
    }

    #[test]
    fn validate_session_alone_ok() {
        let cli = Cli::parse_from(["gator", "claude", "--session=abc"]);
        assert!(cli.validate().is_ok());
    }

    #[test]
    fn validate_no_session_ok() {
        let cli = Cli::parse_from(["gator", "claude", "--policy=audit"]);
        assert!(cli.validate().is_ok());
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

    #[test]
    fn parse_share_worktrees() {
        let cli = Cli::parse_from(["gator", "claude", "--share-worktrees"]);
        assert!(cli.share_worktrees);
    }

    #[test]
    fn validate_share_worktrees_with_session() {
        let cli = Cli::parse_from(["gator", "claude", "--share-worktrees", "--session=abc"]);
        let err = cli.validate().unwrap_err();
        assert!(err.contains("--share-worktrees"));
    }

    #[test]
    fn validate_share_worktrees_without_session_ok() {
        let cli = Cli::parse_from(["gator", "claude", "--share-worktrees"]);
        assert!(cli.validate().is_ok());
    }

    #[test]
    fn parse_no_yolo() {
        let cli = Cli::parse_from(["gator", "claude", "--no-yolo"]);
        assert!(cli.no_yolo);
    }

    #[test]
    fn parse_no_yolo_default_false() {
        let cli = Cli::parse_from(["gator", "claude"]);
        assert!(!cli.no_yolo);
    }

    #[test]
    fn validate_no_yolo_with_session() {
        let cli = Cli::parse_from(["gator", "claude", "--no-yolo", "--session=abc"]);
        let err = cli.validate().unwrap_err();
        assert!(err.contains("--no-yolo"));
    }

    #[test]
    fn validate_no_yolo_without_session_ok() {
        let cli = Cli::parse_from(["gator", "claude", "--no-yolo"]);
        assert!(cli.validate().is_ok());
    }
}
