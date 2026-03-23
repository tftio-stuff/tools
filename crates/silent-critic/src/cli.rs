use crate::models::{DecisionType, EvaluatorType, ExportFormat};
use clap::{Parser, Subcommand};
use tftio_cli_common::{
    AgentArgument, AgentCommand, AgentConfigFile, AgentDoc, AgentEnvironmentVar, AgentExample,
    AgentFailureMode, AgentOperatorMistake, AgentOutputShape, AgentPath, AgentSection, AgentTool,
    AgentUsage,
};

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
    /// Output resolved sandbox specification for a session's contract
    Sandbox {
        /// Session ID (defaults to current session for the worktree)
        #[arg(value_name = "SESSION_ID")]
        session_id: Option<String>,

        /// Output format
        #[arg(long, default_value = "json")]
        format: String,
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

#[must_use]
pub fn agent_doc() -> AgentDoc {
    AgentDoc {
        schema_version: "1".to_owned(),
        tool: AgentTool {
            name: "silent-critic".to_owned(),
            binary: "silent-critic".to_owned(),
            summary: "SQLite-backed supervision framework for project, criterion, session, contract, decision, and log workflows.".to_owned(),
        },
        usage: AgentUsage {
            invocation: "silent-critic [--json] <COMMAND>".to_owned(),
            notes: vec![
                "Use `silent-critic --agent-help` for canonical YAML and `silent-critic --agent-skill` for the markdown skill rendering.".to_owned(),
                "The hidden agent-doc flags are top-level only; ordinary invocations still require a subcommand.".to_owned(),
                "Most success paths honor the global `--json` envelope, but `session sandbox` and `log --format {json|markdown}` return their dedicated export payloads directly.".to_owned(),
            ],
        },
        shared_sections: vec![
            AgentSection {
                title: "agent-doc flags".to_owned(),
                content: "`--agent-help` and `--agent-skill` are hidden top-level-only requests. They print to stdout and exit 0. Requests like `silent-critic session --agent-help` are rejected because the raw-argv detector only accepts exact top-level invocations.".to_owned(),
            },
            AgentSection {
                title: "config and database resolution".to_owned(),
                content: "Config loads from `$XDG_CONFIG_HOME/silent-critic/config.toml` or `~/.config/silent-critic/config.toml`. The config may set `db_dir` and `project_name`. Database paths resolve to `<db_dir>/<repo-hash>/db.sqlite` or `$XDG_DATA_HOME/silent-critic/<repo-hash>/db.sqlite` or `~/.local/share/silent-critic/<repo-hash>/db.sqlite` after deriving the repository root and repo hash from the current working directory.".to_owned(),
            },
            AgentSection {
                title: "session state machine".to_owned(),
                content: "Active sessions move through `discovering -> composing -> ready -> executing -> awaiting_adjudication -> adjudicated`. `session new` creates the active session, `session compose-from` transitions from `composing` to `ready`, `session go --prompt-only` transitions from `ready` to `executing`, `session end` moves to `awaiting_adjudication`, and `decide` completes the transition to `adjudicated`.".to_owned(),
            },
            AgentSection {
                title: "worker authentication and prompt flow".to_owned(),
                content: "`session manifest` and `session submit` require the worker-side `SILENT_CRITIC_TOKEN` environment variable. `session go` currently requires `--prompt-only`; it generates the worker prompt, stores the worker token in SQLite, and expects the worker process to execute `silent-critic session submit --criterion <ID>` commands against the active session.".to_owned(),
            },
        ],
        commands: vec![
            router_command(
                "project",
                "Namespace for repository-scoped project initialization.",
                "silent-critic project <SUBCOMMAND>",
            ),
            AgentCommand {
                name: "project init".to_owned(),
                summary: "Initialize the project row and SQLite schema for the current Git repository.".to_owned(),
                usage: "silent-critic [--json] project init [--name <NAME>]".to_owned(),
                arguments: vec![flag(
                    "name",
                    "Optional project name override. Otherwise uses `config.project_name`, the repo directory name, or `unnamed`.",
                    false,
                )],
                output_shapes: vec![
                    text_shape(
                        "project-init-text",
                        "Two stdout lines: `project: <name>` and `db: <path>`.",
                    ),
                    json_shape(
                        "project-init-json",
                        "Success envelope with `project_id`, `project_name`, `repo_hash`, and `db_path`.",
                    ),
                ],
            },
            router_command(
                "criterion",
                "Namespace for criterion library management.",
                "silent-critic [--json] criterion <SUBCOMMAND>",
            ),
            AgentCommand {
                name: "criterion create".to_owned(),
                summary: "Create a criterion in the shared library for later contract composition.".to_owned(),
                usage: "silent-critic [--json] criterion create [--namespace <NS>] [--name <NAME>] [--claim <CLAIM>] [--evaluator-type <TYPE>] [--check-spec <CMD>] [--parameter-schema <JSON>]".to_owned(),
                arguments: vec![
                    flag("namespace", "Criterion namespace string.", false),
                    flag("name", "Criterion name.", false),
                    flag("claim", "Human-readable claim text.", false),
                    flag(
                        "evaluator-type",
                        "One of `automated`, `human_judgment`, or `agent_evaluated`.",
                        false,
                    ),
                    flag(
                        "check-spec",
                        "Command or spec the worker should satisfy for tool-authored evidence.",
                        false,
                    ),
                    flag(
                        "parameter-schema",
                        "Optional JSON schema string for criterion parameters.",
                        false,
                    ),
                ],
                output_shapes: vec![
                    text_shape("criterion-create-text", "Single stdout line: `<id> [<namespace>] <name>`."),
                    json_shape("criterion-create-json", "Success envelope with the serialized criterion row."),
                ],
            },
            AgentCommand {
                name: "criterion list".to_owned(),
                summary: "List criteria, optionally filtered by namespace.".to_owned(),
                usage: "silent-critic [--json] criterion list [--namespace <NS>]".to_owned(),
                arguments: vec![flag("namespace", "Optional namespace filter.", false)],
                output_shapes: vec![
                    text_shape(
                        "criterion-list-text",
                        "Tab-separated rows: `id`, `[namespace]`, `name`, and `claim`.",
                    ),
                    json_shape("criterion-list-json", "Success envelope with `data.criteria`."),
                ],
            },
            AgentCommand {
                name: "criterion show".to_owned(),
                summary: "Show one criterion in detail.".to_owned(),
                usage: "silent-critic [--json] criterion show <ID>".to_owned(),
                arguments: vec![positional("id", "Criterion ID.", true)],
                output_shapes: vec![
                    text_shape(
                        "criterion-show-text",
                        "Plain-text field dump including id, namespace, name, claim, evaluator, check_spec, and created timestamp.",
                    ),
                    json_shape("criterion-show-json", "Success envelope with `data.criterion`."),
                ],
            },
            AgentCommand {
                name: "criterion update".to_owned(),
                summary: "Update mutable criterion fields.".to_owned(),
                usage: "silent-critic [--json] criterion update <ID> [--namespace <NS>] [--name <NAME>] [--claim <CLAIM>] [--evaluator-type <TYPE>] [--check-spec <CMD>]".to_owned(),
                arguments: vec![
                    positional("id", "Criterion ID.", true),
                    flag("namespace", "Replacement namespace.", false),
                    flag("name", "Replacement name.", false),
                    flag("claim", "Replacement claim text.", false),
                    flag(
                        "evaluator-type",
                        "Replacement evaluator type (`automated`, `human_judgment`, `agent_evaluated`).",
                        false,
                    ),
                    flag("check-spec", "Replacement verification command/spec.", false),
                ],
                output_shapes: vec![
                    text_shape("criterion-update-text", "Single stdout line: `updated: <id> [<namespace>] <name>`."),
                    json_shape("criterion-update-json", "Success envelope with the updated criterion row."),
                ],
            },
            AgentCommand {
                name: "criterion deprecate".to_owned(),
                summary: "Mark a criterion deprecated.".to_owned(),
                usage: "silent-critic [--json] criterion deprecate <ID>".to_owned(),
                arguments: vec![positional("id", "Criterion ID.", true)],
                output_shapes: vec![
                    text_shape("criterion-deprecate-text", "Single stdout line: `deprecated: <id>`."),
                    json_shape("criterion-deprecate-json", "Success envelope with the deprecated criterion id."),
                ],
            },
            AgentCommand {
                name: "criterion export".to_owned(),
                summary: "Export one criterion as TOML.".to_owned(),
                usage: "silent-critic [--json] criterion export <ID>".to_owned(),
                arguments: vec![positional("id", "Criterion ID.", true)],
                output_shapes: vec![
                    text_shape("criterion-export-text", "Raw TOML representation of the criterion."),
                    json_shape("criterion-export-json", "Success envelope with the TOML string in `data.toml`."),
                ],
            },
            AgentCommand {
                name: "criterion import".to_owned(),
                summary: "Import a criterion from a TOML file.".to_owned(),
                usage: "silent-critic [--json] criterion import <FILE>".to_owned(),
                arguments: vec![positional("file", "Path to a TOML criterion file.", true)],
                output_shapes: vec![
                    text_shape("criterion-import-text", "Single stdout line: `imported: <id> [<namespace>] <name>`."),
                    json_shape("criterion-import-json", "Success envelope with the imported criterion row."),
                ],
            },
            router_command(
                "session",
                "Namespace for active-session lifecycle operations.",
                "silent-critic [--json] session <SUBCOMMAND>",
            ),
            AgentCommand {
                name: "session new".to_owned(),
                summary: "Create a new active session for a worktree.".to_owned(),
                usage: "silent-critic [--json] session new --worktree <PATH>".to_owned(),
                arguments: vec![flag("worktree", "Path to the worktree under review.", true)],
                output_shapes: vec![
                    text_shape(
                        "session-new-text",
                        "Three stdout lines: session id, session status, and operator token.",
                    ),
                    json_shape(
                        "session-new-json",
                        "Success envelope with `session_id`, `status`, and `operator_token`.",
                    ),
                ],
            },
            AgentCommand {
                name: "session discover".to_owned(),
                summary: "Gather repository context documents for the active session.".to_owned(),
                usage: "silent-critic [--json] session discover [--doc <PATH>]...".to_owned(),
                arguments: vec![flag(
                    "doc",
                    "Additional document path to ingest; repeat for multiple files.",
                    false,
                )],
                output_shapes: vec![
                    text_shape("session-discover-text", "Single stdout line reporting discovered context count."),
                    json_shape("session-discover-json", "Success envelope with `context_count` and `contexts`."),
                ],
            },
            AgentCommand {
                name: "session status".to_owned(),
                summary: "Show the current active session report.".to_owned(),
                usage: "silent-critic [--json] session status".to_owned(),
                arguments: vec![],
                output_shapes: vec![
                    text_shape(
                        "session-status-text",
                        "Plain-text session report with ids, worktree, goal, criteria count, evidence count, and discovery count.",
                    ),
                    json_shape("session-status-json", "Success envelope with the structured session status report."),
                ],
            },
            AgentCommand {
                name: "session end".to_owned(),
                summary: "End the active session and compute residual uncertainty.".to_owned(),
                usage: "silent-critic [--json] session end".to_owned(),
                arguments: vec![],
                output_shapes: vec![
                    text_shape(
                        "session-end-text",
                        "Text summary including contract id, criteria count, evidence count, and residuals.",
                    ),
                    json_shape(
                        "session-end-json",
                        "Success envelope with residual count and residual detail objects.",
                    ),
                ],
            },
            AgentCommand {
                name: "session manifest".to_owned(),
                summary: "Show the visible worker contract surface for the active executing session.".to_owned(),
                usage: "SILENT_CRITIC_TOKEN=<worker-token> silent-critic [--json] session manifest".to_owned(),
                arguments: vec![],
                output_shapes: vec![
                    text_shape("session-manifest-text", "Worker-facing goal text plus visible criteria and check specs."),
                    json_shape("session-manifest-json", "Success envelope with `goal` and visible `criteria`."),
                ],
            },
            AgentCommand {
                name: "session submit".to_owned(),
                summary: "Execute and record tool-authored evidence for one criterion.".to_owned(),
                usage: "SILENT_CRITIC_TOKEN=<worker-token> silent-critic [--json] session submit --criterion <ID>".to_owned(),
                arguments: vec![flag("criterion", "Criterion ID to verify.", true)],
                output_shapes: vec![
                    text_shape("session-submit-text", "Text lines with evidence id, exit code, and pass boolean."),
                    json_shape("session-submit-json", "Success envelope with `evidence_id`, `criterion_id`, `exit_code`, and `pass`."),
                ],
            },
            AgentCommand {
                name: "session sandbox".to_owned(),
                summary: "Export the resolved sandbox for a session's contract.".to_owned(),
                usage: "silent-critic session sandbox [<SESSION_ID>] [--format json]".to_owned(),
                arguments: vec![
                    positional(
                        "session_id",
                        "Optional session id. When absent, the active session is used.",
                        false,
                    ),
                    flag("format", "Output format string. The current implementation returns JSON.", false),
                ],
                output_shapes: vec![AgentOutputShape {
                    name: "session-sandbox-json".to_owned(),
                    format: "json".to_owned(),
                    description: "Pretty-printed JSON with `workdir`, `grants.rw`, `grants.ro`, and `denies`. This command bypasses the generic `--json` envelope.".to_owned(),
                }],
            },
            AgentCommand {
                name: "session compose-from".to_owned(),
                summary: "Create a contract from JSON on stdin and transition the active session to ready.".to_owned(),
                usage: "cat contract.json | silent-critic [--json] session compose-from".to_owned(),
                arguments: vec![],
                output_shapes: vec![
                    text_shape(
                        "session-compose-from-text",
                        "Text lines with contract id, goal, criteria created, and criteria reused.",
                    ),
                    json_shape(
                        "session-compose-from-json",
                        "Success envelope with `contract_id`, `goal`, `criteria_created`, and `criteria_reused`.",
                    ),
                ],
            },
            AgentCommand {
                name: "session go".to_owned(),
                summary: "Generate the worker prompt and transition the active session to executing.".to_owned(),
                usage: "silent-critic [--json] session go --prompt-only".to_owned(),
                arguments: vec![flag(
                    "prompt-only",
                    "Required. Process spawning is not supported, so prompt generation must stay explicit.",
                    false,
                )],
                output_shapes: vec![
                    text_shape("session-go-text", "Worker prompt text containing visible criteria and submit commands."),
                    json_shape("session-go-json", "Success envelope with `worker_token` and `prompt`."),
                ],
            },
            router_command(
                "contract",
                "Namespace for inspecting a composed contract.",
                "silent-critic [--json] contract <SUBCOMMAND>",
            ),
            AgentCommand {
                name: "contract show".to_owned(),
                summary: "Show contract details and criteria from the requested role perspective.".to_owned(),
                usage: "silent-critic [--json] contract show <ID> [--role <ROLE>]".to_owned(),
                arguments: vec![
                    positional("id", "Contract ID.", true),
                    flag("role", "View role, default `operator`.", false),
                ],
                output_shapes: vec![
                    text_shape("contract-show-text", "Text report with goal and criteria list."),
                    json_shape("contract-show-json", "Success envelope with `id`, `goal`, and criteria entries."),
                ],
            },
            AgentCommand {
                name: "decide".to_owned(),
                summary: "Record an adjudication decision for a contract.".to_owned(),
                usage: "silent-critic [--json] decide --contract <ID> --type <TYPE> --basis <TEXT> [--evidence-refs <CSV>]".to_owned(),
                arguments: vec![
                    flag("contract", "Contract ID.", true),
                    flag(
                        "type",
                        "Decision type: `accept`, `reject`, `accept_residual`, `insufficient_evidence`, `waive_criterion`, `require_rework`, or `rescope`.",
                        true,
                    ),
                    flag("basis", "Basis text for the decision.", true),
                    flag("evidence-refs", "Optional comma-separated evidence ids.", false),
                ],
                output_shapes: vec![
                    text_shape("decide-text", "Text lines with decision id, type, and outcome."),
                    json_shape("decide-json", "Success envelope with `decision_id`, `type`, and `outcome`."),
                ],
            },
            AgentCommand {
                name: "log".to_owned(),
                summary: "Export the decision log for a contract as JSON or Markdown.".to_owned(),
                usage: "silent-critic log <CONTRACT_ID> [--format json|markdown]".to_owned(),
                arguments: vec![
                    positional("contract", "Contract ID.", true),
                    flag("format", "Export format: `json` or `markdown`.", false),
                ],
                output_shapes: vec![
                    AgentOutputShape {
                        name: "log-json".to_owned(),
                        format: "json".to_owned(),
                        description: "Pretty-printed JSON with contract, session, criteria, evidence, decisions, and audit trail. This command bypasses the generic `--json` envelope.".to_owned(),
                    },
                    AgentOutputShape {
                        name: "log-markdown".to_owned(),
                        format: "markdown".to_owned(),
                        description: "Markdown decision log with discovery context, criteria table, evidence sections, and decision summary. This command bypasses the generic `--json` envelope.".to_owned(),
                    },
                ],
            },
        ],
        arguments: vec![flag(
            "json",
            "Wrap most command successes and all structured errors in a JSON envelope on stdout.",
            false,
        )],
        environment_variables: vec![
            AgentEnvironmentVar {
                name: "SILENT_CRITIC_TOKEN".to_owned(),
                description: "Worker token required for `session manifest` and `session submit`.".to_owned(),
                required: false,
            },
            AgentEnvironmentVar {
                name: "XDG_CONFIG_HOME".to_owned(),
                description: "Overrides the config directory for `silent-critic/config.toml`.".to_owned(),
                required: false,
            },
            AgentEnvironmentVar {
                name: "XDG_DATA_HOME".to_owned(),
                description: "Overrides the default data directory for repo-hash-scoped `db.sqlite` files.".to_owned(),
                required: false,
            },
            AgentEnvironmentVar {
                name: "HOME".to_owned(),
                description: "Fallback root for default config and local data directories when XDG variables are unset.".to_owned(),
                required: false,
            },
        ],
        config_files: vec![AgentConfigFile {
            path: "$XDG_CONFIG_HOME/silent-critic/config.toml or ~/.config/silent-critic/config.toml".to_owned(),
            purpose: "Optional config file. Today it supports `db_dir` and `project_name`.".to_owned(),
        }],
        default_paths: vec![
            AgentPath {
                name: "config".to_owned(),
                path: "~/.config/silent-critic/config.toml".to_owned(),
                purpose: "Default config path when `XDG_CONFIG_HOME` is unset.".to_owned(),
            },
            AgentPath {
                name: "project-database".to_owned(),
                path: "~/.local/share/silent-critic/<repo-hash>/db.sqlite".to_owned(),
                purpose: "Default SQLite state path when `db_dir` and `XDG_DATA_HOME` are unset.".to_owned(),
            },
        ],
        output_shapes: vec![
            json_shape(
                "ok-response",
                "Success envelope shared by most `--json` commands: `{ ok: true, command, data }`.",
            ),
            json_shape(
                "error-response",
                "Failure envelope used by `run()`: `{ ok: false, command: \"error\", error: { code: \"ERROR\", message, details } }`.",
            ),
        ],
        examples: vec![
            AgentExample {
                name: "project-init".to_owned(),
                command: "silent-critic project init --name workspace-tools".to_owned(),
                description: "Initialize the current repository's SQLite state with an explicit project name.".to_owned(),
            },
            AgentExample {
                name: "criterion-create".to_owned(),
                command: "silent-critic criterion create --namespace testing --name cargo-test --claim 'cargo test passes' --evaluator-type automated --check-spec 'cargo test'".to_owned(),
                description: "Create a reusable automated criterion.".to_owned(),
            },
            AgentExample {
                name: "session-compose-from".to_owned(),
                command: "cat contract.json | silent-critic session compose-from".to_owned(),
                description: "Read `ComposeFromInput` JSON from stdin and move the active session to `ready`.".to_owned(),
            },
            AgentExample {
                name: "session-go".to_owned(),
                command: "silent-critic --json session go --prompt-only".to_owned(),
                description: "Generate the worker prompt and worker token without spawning a process.".to_owned(),
            },
            AgentExample {
                name: "session-submit".to_owned(),
                command: "SILENT_CRITIC_TOKEN=<worker-token> silent-critic session submit --criterion crit-123".to_owned(),
                description: "Record tool-authored evidence for one visible criterion.".to_owned(),
            },
            AgentExample {
                name: "log-markdown".to_owned(),
                command: "silent-critic log contract-123 --format markdown".to_owned(),
                description: "Export a markdown decision log for human review.".to_owned(),
            },
        ],
        failure_modes: vec![
            AgentFailureMode {
                name: "not-in-git-repo".to_owned(),
                symptom: "`project init` or other repo-scoped commands cannot resolve the repository root.".to_owned(),
                resolution: "Run inside a Git worktree or provide the expected repository context before invoking silent-critic.".to_owned(),
            },
            AgentFailureMode {
                name: "missing-worker-token".to_owned(),
                symptom: "`session manifest` or `session submit` fails with `SILENT_CRITIC_TOKEN not set`.".to_owned(),
                resolution: "Use the worker token emitted by `session go --prompt-only` and export it into the worker environment.".to_owned(),
            },
            AgentFailureMode {
                name: "wrong-session-state".to_owned(),
                symptom: "A lifecycle command reports that the active session is in the wrong state.".to_owned(),
                resolution: "Advance commands in state-machine order: discover, compose-from, go, end, then decide.".to_owned(),
            },
            AgentFailureMode {
                name: "missing-sandbox-or-contract".to_owned(),
                symptom: "`session sandbox` reports no active session, no contract, or no sandbox configuration.".to_owned(),
                resolution: "Compose a contract with sandbox settings first, or pass an explicit session id that already has sandbox data.".to_owned(),
            },
            AgentFailureMode {
                name: "prompt-only-required".to_owned(),
                symptom: "`session go` fails because `--prompt-only` was omitted.".to_owned(),
                resolution: "Always pass `--prompt-only`; process spawning is intentionally unsupported.".to_owned(),
            },
        ],
        operator_mistakes: vec![
            AgentOperatorMistake {
                name: "agent-doc-flag-after-subcommand".to_owned(),
                symptom: "`silent-critic session --agent-help` does not return the agent document.".to_owned(),
                correction: "Request agent docs only as `silent-critic --agent-help` or `silent-critic --agent-skill`.".to_owned(),
            },
            AgentOperatorMistake {
                name: "assuming-global-json-wraps-exports".to_owned(),
                symptom: "`--json` is passed to `log` or `session sandbox`, but the output is still raw export content.".to_owned(),
                correction: "Treat `log` and `session sandbox` as dedicated export commands whose own format controls the payload.".to_owned(),
            },
            AgentOperatorMistake {
                name: "skipping-session-order".to_owned(),
                symptom: "Running `session go`, `session end`, or `decide` too early causes state errors.".to_owned(),
                correction: "Follow the state machine exactly: `session new`, `session discover`, `session compose-from`, `session go --prompt-only`, `session end`, then `decide`.".to_owned(),
            },
        ],
        constraints: vec![
            "Top-level agent-doc flags stay hidden from normal help output.".to_owned(),
            "Session state is persisted in SQLite under a repo-hash-specific database path.".to_owned(),
            "Worker-only commands require `SILENT_CRITIC_TOKEN`; operator commands do not mint a token implicitly.".to_owned(),
        ],
    }
}

fn flag(name: &str, description: &str, required: bool) -> AgentArgument {
    AgentArgument {
        name: name.to_owned(),
        positional: false,
        description: description.to_owned(),
        required,
    }
}

fn positional(name: &str, description: &str, required: bool) -> AgentArgument {
    AgentArgument {
        name: name.to_owned(),
        positional: true,
        description: description.to_owned(),
        required,
    }
}

fn text_shape(name: &str, description: &str) -> AgentOutputShape {
    AgentOutputShape {
        name: name.to_owned(),
        format: "text".to_owned(),
        description: description.to_owned(),
    }
}

fn json_shape(name: &str, description: &str) -> AgentOutputShape {
    AgentOutputShape {
        name: name.to_owned(),
        format: "json".to_owned(),
        description: description.to_owned(),
    }
}

fn router_command(name: &str, summary: &str, usage: &str) -> AgentCommand {
    AgentCommand {
        name: name.to_owned(),
        summary: summary.to_owned(),
        usage: usage.to_owned(),
        arguments: vec![],
        output_shapes: vec![AgentOutputShape {
            name: format!("{name}-dispatch"),
            format: "none".to_owned(),
            description: "This command is a namespace and requires a nested subcommand before it emits output.".to_owned(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tftio_cli_common::agent_docs::{assert_argument_coverage, assert_command_coverage};

    #[test]
    fn agent_doc_covers_silent_critic_command_tree() {
        assert_command_coverage::<Cli>(&[
            "contract",
            "contract show",
            "criterion",
            "criterion create",
            "criterion deprecate",
            "criterion export",
            "criterion import",
            "criterion list",
            "criterion show",
            "criterion update",
            "decide",
            "log",
            "project",
            "project init",
            "session",
            "session compose-from",
            "session discover",
            "session end",
            "session go",
            "session manifest",
            "session new",
            "session sandbox",
            "session status",
            "session submit",
        ]);
    }

    #[test]
    fn agent_doc_covers_silent_critic_arguments() {
        assert_argument_coverage::<Cli>(&[], &["json"], &[], &[]);
        assert_argument_coverage::<Cli>(&["project"], &[], &[], &[]);
        assert_argument_coverage::<Cli>(&["project", "init"], &["name"], &[], &[]);
        assert_argument_coverage::<Cli>(&["criterion"], &[], &[], &[]);
        assert_argument_coverage::<Cli>(
            &["criterion", "create"],
            &[
                "namespace",
                "name",
                "claim",
                "evaluator-type",
                "check-spec",
                "parameter-schema",
            ],
            &[],
            &[],
        );
        assert_argument_coverage::<Cli>(&["criterion", "list"], &["namespace"], &[], &[]);
        assert_argument_coverage::<Cli>(&["criterion", "show"], &[], &["id"], &[]);
        assert_argument_coverage::<Cli>(
            &["criterion", "update"],
            &["namespace", "name", "claim", "evaluator-type", "check-spec"],
            &["id"],
            &[],
        );
        assert_argument_coverage::<Cli>(&["criterion", "deprecate"], &[], &["id"], &[]);
        assert_argument_coverage::<Cli>(&["criterion", "export"], &[], &["id"], &[]);
        assert_argument_coverage::<Cli>(&["criterion", "import"], &[], &["file"], &[]);
        assert_argument_coverage::<Cli>(&["session"], &[], &[], &[]);
        assert_argument_coverage::<Cli>(&["session", "new"], &["worktree"], &[], &[]);
        assert_argument_coverage::<Cli>(&["session", "discover"], &["doc"], &[], &[]);
        assert_argument_coverage::<Cli>(&["session", "status"], &[], &[], &[]);
        assert_argument_coverage::<Cli>(&["session", "end"], &[], &[], &[]);
        assert_argument_coverage::<Cli>(&["session", "manifest"], &[], &[], &[]);
        assert_argument_coverage::<Cli>(&["session", "submit"], &["criterion"], &[], &[]);
        assert_argument_coverage::<Cli>(
            &["session", "sandbox"],
            &["format"],
            &["session_id"],
            &[],
        );
        assert_argument_coverage::<Cli>(&["session", "compose-from"], &[], &[], &[]);
        assert_argument_coverage::<Cli>(&["session", "go"], &["prompt-only"], &[], &[]);
        assert_argument_coverage::<Cli>(&["contract"], &[], &[], &[]);
        assert_argument_coverage::<Cli>(&["contract", "show"], &["role"], &["id"], &[]);
        assert_argument_coverage::<Cli>(
            &["decide"],
            &["contract", "type", "basis", "evidence-refs"],
            &[],
            &[],
        );
        assert_argument_coverage::<Cli>(&["log"], &["format"], &["contract"], &[]);
    }
}
