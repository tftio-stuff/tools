use crate::models::{DecisionType, EvaluatorType, ExportFormat};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "silent-critic", about = "Supervision framework for agentic software development")]
pub struct Cli {
    /// Output JSON instead of plain text
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Project management
    Project {
        #[command(subcommand)]
        command: ProjectCommand,
    },
    /// Criterion library management
    Criterion {
        #[command(subcommand)]
        command: CriterionCommand,
    },
    /// Session lifecycle
    Session {
        #[command(subcommand)]
        command: SessionCommand,
    },
    /// Contract inspection
    Contract {
        #[command(subcommand)]
        command: ContractCommand,
    },
    /// Record adjudication decision
    Decide {
        /// Contract ID
        #[arg(long)]
        contract: String,

        /// Decision type
        #[arg(long, value_name = "TYPE")]
        r#type: DecisionType,

        /// Basis for the decision
        #[arg(long)]
        basis: String,

        /// Evidence reference IDs (comma-separated)
        #[arg(long)]
        evidence_refs: Option<String>,
    },
    /// Export decision log
    Log {
        /// Contract ID
        contract: String,

        /// Export format
        #[arg(long, default_value = "json")]
        format: ExportFormat,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommand {
    /// Initialize a new project
    Init {
        /// Project name (defaults to git repo name)
        #[arg(long)]
        name: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum CriterionCommand {
    /// Create a new criterion
    Create {
        /// Criterion namespace
        #[arg(long)]
        namespace: Option<String>,

        /// Criterion name
        #[arg(long)]
        name: Option<String>,

        /// What the criterion claims
        #[arg(long)]
        claim: Option<String>,

        /// Evaluator type
        #[arg(long)]
        evaluator_type: Option<EvaluatorType>,

        /// Command to run for verification
        #[arg(long)]
        check_spec: Option<String>,

        /// JSON schema for parameters
        #[arg(long)]
        parameter_schema: Option<String>,

    },
    /// List criteria
    List {
        /// Filter by namespace
        #[arg(long)]
        namespace: Option<String>,
    },
    /// Show a criterion
    Show {
        /// Criterion ID
        id: String,
    },
    /// Update a criterion
    Update {
        /// Criterion ID
        id: String,

        /// New namespace
        #[arg(long)]
        namespace: Option<String>,

        /// New name
        #[arg(long)]
        name: Option<String>,

        /// New claim
        #[arg(long)]
        claim: Option<String>,

        /// New evaluator type
        #[arg(long)]
        evaluator_type: Option<EvaluatorType>,

        /// New check spec
        #[arg(long)]
        check_spec: Option<String>,
    },
    /// Deprecate a criterion
    Deprecate {
        /// Criterion ID
        id: String,
    },
    /// Export a criterion to TOML
    Export {
        /// Criterion ID
        id: String,
    },
    /// Import a criterion from TOML file
    Import {
        /// Path to TOML file
        file: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum SessionCommand {
    /// Create a new session
    New {
        /// Path to the worktree
        #[arg(long)]
        worktree: String,
    },
    /// Gather repository context
    Discover {
        /// Additional document paths to include
        #[arg(long = "doc")]
        docs: Vec<String>,
    },
    /// Show current session status
    Status,
    /// End the session and compute residuals
    End,
    /// Show the visible contract surface (worker command)
    Manifest,
    /// Submit evidence for a criterion (worker command)
    Submit {
        /// Criterion ID to check
        #[arg(long)]
        criterion: String,
    },
    /// Create contract from JSON input (stdin)
    ComposeFrom,
    /// Generate worker prompt and transition to executing
    Go {
        /// Output worker prompt instead of spawning a process
        #[arg(long)]
        prompt_only: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum ContractCommand {
    /// Show contract details
    Show {
        /// Contract ID
        id: String,

        /// View as role
        #[arg(long, default_value = "operator")]
        role: String,
    },
}
