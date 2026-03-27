//! CLI definition.

use clap::{Parser, Subcommand};

use crate::commands::export::ExportFormat;
use crate::models::{Role, SourceFormat};

/// Chronicle: interaction history corpus indexer.
#[derive(Parser, Debug)]
#[command(name = "chronicle")]
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level commands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Metadata commands (version, license, completions).
    Meta {
        /// Metadata subcommand.
        #[command(subcommand)]
        command: MetaCommand,
    },
    /// Ingest interaction history files.
    Ingest {
        /// Source name.
        #[arg(long)]
        source: String,
        /// Path to scan for files.
        #[arg(long)]
        path: Option<String>,
        /// Parser format (codex, `claude_code`).
        #[arg(long)]
        parser: Option<SourceFormat>,
        /// Drop and rebuild from scratch.
        #[arg(long)]
        full: bool,
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Re-ingest using stored base paths.
    Reindex {
        /// Limit to a single source.
        #[arg(long)]
        source: Option<String>,
        /// Drop and rebuild from scratch.
        #[arg(long)]
        full: bool,
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Full-text search message content.
    Search {
        /// FTS5 query string.
        query: String,
        /// Filter by source name.
        #[arg(long)]
        source: Option<String>,
        /// Filter by role (user, assistant, system, tool).
        #[arg(long)]
        role: Option<Role>,
        /// Filter by project path.
        #[arg(long)]
        project: Option<String>,
        /// Maximum results.
        #[arg(long, default_value = "20")]
        limit: i64,
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Show source statistics.
    Stats {
        /// Limit to a single source.
        #[arg(long)]
        source: Option<String>,
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Export a session.
    Export {
        /// Session id.
        #[arg(long)]
        session: String,
        /// Output format (json, markdown).
        #[arg(long, default_value = "json")]
        format: ExportFormat,
    },
    /// Manage sources.
    Sources {
        /// Sources subcommand.
        #[command(subcommand)]
        command: SourcesCommand,
    },
}

/// Sources subcommands.
#[derive(Subcommand, Debug)]
pub enum SourcesCommand {
    /// List registered sources.
    List {
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Register a new source.
    Add {
        /// Source name.
        #[arg(long)]
        name: String,
        /// Parser format.
        #[arg(long)]
        parser: SourceFormat,
        /// Base path for file scanning.
        #[arg(long)]
        path: String,
        /// Optional description.
        #[arg(long)]
        description: Option<String>,
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
}

/// Metadata subcommands.
#[derive(Subcommand, Debug)]
pub enum MetaCommand {
    /// Show version.
    Version {
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
    /// Show license.
    License,
    /// Generate shell completions.
    Completions {
        /// Shell to generate for.
        shell: clap_complete::Shell,
    },
    /// Run doctor checks.
    Doctor {
        /// JSON output.
        #[arg(long)]
        json: bool,
    },
}
