//! Command-line interface entry points for the Asana CLI.

mod custom_field;
mod project;
mod section;
mod tag;
mod task;
mod user;
mod workspace;

use crate::api::{ApiClient, ApiError, AuthToken};
use crate::config::Config;
use crate::error::Result;
use anyhow::{Context, anyhow};
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use custom_field::CustomFieldCommand;
use project::ProjectCommand;
use secrecy::SecretString;
use section::SectionCommand;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use tag::TagCommand;
use task::TaskCommand;
use tftio_cli_common::{
    AgentArgument, AgentCommand, AgentConfigFile, AgentDoc, AgentDocRequest,
    AgentEnvironmentVar, AgentExample, AgentFailureMode, AgentOperatorMistake, AgentOutputShape,
    AgentPath, AgentSection, AgentTool, AgentUsage, DoctorCheck, DoctorChecks, LicenseType,
    RepoInfo, StandardCommand, ToolSpec, command::run_standard_command, render_agent_help_yaml,
    render_agent_skill, workspace_tool,
};
use tokio::runtime::Builder as RuntimeBuilder;
use tracing::{debug, info};
use user::UserCommand;
use workspace::WorkspaceCommand;

const MANPAGE_SOURCE: &str = include_str!("../../docs/man/asana-cli.1");

const VERSION: &str = match option_env!("CARGO_PKG_VERSION") {
    Some(version) => version,
    None => "unknown",
};

struct AsanaCliDoctor;

impl DoctorChecks for AsanaCliDoctor {
    fn repo_info() -> RepoInfo {
        RepoInfo::new("tftio-stuff", "tools")
    }

    fn current_version() -> &'static str {
        VERSION
    }

    fn tool_checks(&self) -> Vec<DoctorCheck> {
        crate::doctor::tool_specific_checks()
    }
}

const TOOL_SPEC: ToolSpec = workspace_tool(
    "asana-cli",
    "Asana CLI",
    VERSION,
    LicenseType::MIT,
    false,
    true,
    true,
);

#[derive(Parser, Debug)]
#[command(name = "asana-cli")]
#[command(about = "An interface to the Asana API")]
#[command(version = VERSION)]
struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show version information.
    Version,
    /// Show license information.
    License,
    /// Manage persisted configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Task operations.
    Task {
        #[command(subcommand)]
        command: Box<TaskCommand>,
    },
    /// Project operations.
    Project {
        #[command(subcommand)]
        command: Box<ProjectCommand>,
    },
    /// Section operations.
    Section {
        #[command(subcommand)]
        command: Box<SectionCommand>,
    },
    /// Tag operations.
    Tag {
        #[command(subcommand)]
        command: Box<TagCommand>,
    },
    /// Custom field operations.
    #[command(name = "custom-field")]
    CustomField {
        #[command(subcommand)]
        command: Box<CustomFieldCommand>,
    },
    /// Workspace operations.
    Workspace {
        #[command(subcommand)]
        command: Box<WorkspaceCommand>,
    },
    /// User operations.
    User {
        #[command(subcommand)]
        command: Box<UserCommand>,
    },
    /// Generate shell completion scripts.
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },
    /// Generate the man page (roff output).
    Manpage {
        /// Output directory for the generated man page (defaults to stdout).
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Check health and configuration.
    Doctor,
    /// Update to the latest version.
    Update {
        /// Specific version to install.
        #[arg(long)]
        version: Option<String>,
        /// Force update even if already up-to-date.
        #[arg(short, long)]
        force: bool,
        /// Custom installation directory.
        #[arg(long)]
        install_dir: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    /// Store configuration values.
    Set {
        #[command(subcommand)]
        command: ConfigSetCommand,
    },
    /// Display the current configuration (token redacted).
    Get,
    /// Validate the stored Personal Access Token against the Asana API.
    Test,
}

#[derive(Subcommand, Debug)]
enum ConfigSetCommand {
    /// Store the Personal Access Token.
    Token {
        /// Personal Access Token value; omit to be prompted securely.
        #[arg(long)]
        token: Option<String>,
    },
    /// Store the default workspace gid.
    Workspace {
        /// Workspace gid to use when none is supplied on the command line.
        #[arg(long, value_name = "GID")]
        workspace: Option<String>,
        /// Clear the stored default workspace.
        #[arg(long)]
        clear: bool,
    },
    /// Store the default assignee identifier.
    Assignee {
        /// Identifier (email or gid) that should replace the `me` alias.
        #[arg(long, value_name = "ID")]
        assignee: Option<String>,
        /// Clear the stored default assignee.
        #[arg(long)]
        clear: bool,
    },
    /// Store the default project identifier.
    Project {
        /// Project gid to use when none is supplied on the command line.
        #[arg(long, value_name = "GID")]
        project: Option<String>,
        /// Clear the stored default project.
        #[arg(long)]
        clear: bool,
    },
}

/// Print the authored agent documentation for `asana-cli`.
pub fn run_agent_doc_request(request: AgentDocRequest) {
    let doc = agent_doc();
    let rendered = match request {
        AgentDocRequest::Help => render_agent_help_yaml(&doc),
        AgentDocRequest::Skill => render_agent_skill(&doc),
    };
    print!("{rendered}");
}

/// Build the canonical agent-facing documentation for `asana-cli`.
#[must_use]
pub fn agent_doc() -> AgentDoc {
    AgentDoc {
        schema_version: "1".to_owned(),
        tool: AgentTool {
            name: "asana-cli".to_owned(),
            binary: "asana-cli".to_owned(),
            summary: "Asana API client covering config, tasks, projects, sections, tags, custom fields, workspaces, users, completions, manpages, and health checks.".to_owned(),
        },
        usage: AgentUsage {
            invocation: "asana-cli <COMMAND>".to_owned(),
            notes: vec![
                "Use `asana-cli --agent-help` for canonical YAML and `asana-cli --agent-skill` for the markdown skill rendering.".to_owned(),
                "The hidden agent-doc flags are top-level only; `asana-cli task list --agent-help` is rejected before `Cli::parse()` runs, so normal required-subcommand behavior is preserved.".to_owned(),
                "Most networked commands execute through a single-threaded Tokio runtime, talk to the Asana REST API asynchronously, and print either human-readable tables/details or JSON payloads depending on command-specific output flags.".to_owned(),
            ],
        },
        shared_sections: vec![
            AgentSection {
                title: "authentication and config precedence".to_owned(),
                content: "API commands need a Personal Access Token from `ASANA_PAT` or the persisted config file. Environment overrides win over stored values for `ASANA_PAT`, `ASANA_BASE_URL`, `ASANA_WORKSPACE`, `ASANA_ASSIGNEE`, and `ASANA_PROJECT`. `config set token`, `config set workspace`, `config set assignee`, and `config set project` write defaults to the config file.".to_owned(),
            },
            AgentSection {
                title: "filesystem layout".to_owned(),
                content: "Config loads from `$ASANA_CLI_CONFIG_HOME/config.toml` when that variable is set; otherwise the platform default config directory is used. Persistent data lives under `$ASANA_CLI_DATA_HOME` when set, otherwise the platform data directory. The data directory also contains `cache/`, `templates/`, and `filters/` subdirectories.".to_owned(),
            },
            AgentSection {
                title: "output contracts".to_owned(),
                content: "List/show commands generally support table/detail/json output, some section commands also support csv, batch task commands accept structured JSON or CSV input, `attachments download` writes to a file path, `completions` prints shell completion scripts, and `manpage` prints roff text or writes `asana-cli.1` into a directory.".to_owned(),
            },
            AgentSection {
                title: "network and cache behavior".to_owned(),
                content: "Task, project, section, tag, custom-field, workspace, and user commands contact the Asana API over the network unless the workflow explicitly operates on local config or cached task history. Errors can come from authentication failure, rate limiting, offline mode, invalid identifiers, or malformed local files for batch operations and downloads.".to_owned(),
            },
        ],
        commands: vec![
            command("version", "Show version information.", "asana-cli version", vec![], vec![shape("version-text", "text", "Single stdout line with the binary name and version.")]),
            command("license", "Show license information.", "asana-cli license", vec![], vec![shape("license-text", "text", "Bundled license text on stdout.")]),
            router_command("config", "Namespace for persisted configuration commands.", "asana-cli config <SUBCOMMAND>"),
            router_command("config set", "Namespace for storing configuration defaults.", "asana-cli config set <SUBCOMMAND>"),
            command(
                "config set token",
                "Persist the Personal Access Token, or prompt securely when `--token` is omitted.",
                "asana-cli config set token [--token <VALUE>]",
                vec![flag("token", "Personal Access Token value; omit to be prompted securely.", false)],
                vec![shape("config-set-token-text", "text", "Confirms that the Personal Access Token was stored in the configuration file.")],
            ),
            command(
                "config set workspace",
                "Persist or clear the default workspace gid.",
                "asana-cli config set workspace [--workspace <GID> | --clear]",
                vec![],
                vec![shape("config-set-workspace-text", "text", "Confirms that the default workspace was stored or cleared.")],
            ),
            command(
                "config set assignee",
                "Persist or clear the default assignee alias replacement.",
                "asana-cli config set assignee [--assignee <ID> | --clear]",
                vec![],
                vec![shape("config-set-assignee-text", "text", "Confirms that the default assignee was stored or cleared.")],
            ),
            command(
                "config set project",
                "Persist or clear the default project gid.",
                "asana-cli config set project [--project <GID> | --clear]",
                vec![],
                vec![shape("config-set-project-text", "text", "Confirms that the default project was stored or cleared.")],
            ),
            command("config get", "Display the current config with the token status redacted.", "asana-cli config get", vec![], vec![shape("config-get-text", "text", "Human-readable config summary including file path, API base URL, defaults, and token status.")]),
            command("config test", "Validate the effective Personal Access Token against the Asana API.", "asana-cli config test", vec![], vec![shape("config-test-text", "text", "Success or failure text after an authenticated `users/me` request.")]),
            router_command("task", "Namespace for task CRUD, search, relationship, comment, attachment, and batch workflows.", "asana-cli task <SUBCOMMAND>"),
            command("task list", "List tasks with filtering, sorting, and format selection.", "asana-cli task list [--workspace <GID>] [--project <GID>] [--section <GID>] [--assignee <ID>] [--completed <BOOL>] [--due-before <DATE>] [--due-after <DATE>] [--include-subtasks] [--limit <N>] [--sort <FIELD>] [--fields <FIELD>...] [--output <FORMAT>]", vec![], vec![shape("task-list-output", "table|json|detail", "Task list output in the requested format.")]),
            command("task show", "Show one task with optional extra fields.", "asana-cli task show <TASK> [--fields <FIELD>...] [--output <FORMAT>]", vec![], vec![shape("task-show-output", "detail|json|table", "Detailed task output.")]),
            command("task create", "Create one task, optionally interactive and enriched with dates, tags, followers, and custom fields.", "asana-cli task create [--name <TEXT>] [--workspace <GID>] [--project <GID>...] [--section <GID>] [--parent <TASK>] [--assignee <ID>] [--notes <TEXT>] [--html-notes <HTML>] [--due-on <DATE>] [--due-at <DATETIME>] [--start-on <DATE>] [--start-at <DATETIME>] [--tag <TAG>...] [--follower <USER>...] [--custom-field <KEY=VALUE>...] [--interactive] [--output <FORMAT>]", vec![], vec![shape("task-create-output", "detail|json|table", "Created task representation.")]),
            command("task update", "Update task fields, completion state, notes, dates, assignments, and relationships.", "asana-cli task update <TASK> ...", vec![], vec![shape("task-update-output", "detail|json|table", "Updated task representation.")]),
            command("task delete", "Delete a task, optionally skipping confirmation.", "asana-cli task delete <TASK> [--yes]", vec![], vec![shape("task-delete-output", "text", "Confirmation text after deletion.")]),
            command("task create-batch", "Create multiple tasks from structured JSON or CSV input.", "asana-cli task create-batch --file <PATH> --format <json|csv> [--output <FORMAT>]", vec![], vec![shape("task-create-batch-output", "json|table|detail", "Per-record results for the batch create workflow.")]),
            command("task update-batch", "Update multiple tasks from structured JSON or CSV input.", "asana-cli task update-batch --file <PATH> --format <json|csv> [--output <FORMAT>]", vec![], vec![shape("task-update-batch-output", "json|table|detail", "Per-record results for the batch update workflow.")]),
            command("task complete-batch", "Mark multiple tasks complete or incomplete from structured input.", "asana-cli task complete-batch --file <PATH> --format <json|csv> [--output <FORMAT>]", vec![], vec![shape("task-complete-batch-output", "json|table|detail", "Per-record results for batch completion changes.")]),
            command("task search", "Search tasks with fuzzy text matching and optional recent-only cache usage.", "asana-cli task search --query <TEXT> [--workspace <GID>] [--project <GID>] [--assignee <ID>] [--limit <N>] [--recent-only] [--output <FORMAT>]", vec![], vec![shape("task-search-output", "json|table|detail", "Task search results.")]),
            router_command("task subtasks", "Namespace for subtask listing, creation, and conversion.", "asana-cli task subtasks <SUBCOMMAND>"),
            command("task subtasks list", "List subtasks for a parent task, optionally recursively.", "asana-cli task subtasks list <TASK> [--recursive] [--fields <FIELD>...] [--output <FORMAT>]", vec![], vec![shape("task-subtasks-list-output", "json|table|detail", "Subtask list output.")]),
            command(
                "task subtasks create",
                "Create a new subtask beneath a parent task.",
                "asana-cli task subtasks create <TASK> [--name <TEXT>] [--assignee <ID>] [--due-on <DATE>] [--due-at <DATETIME>] [--start-on <DATE>] [--start-at <DATETIME>] [--tag <TAG>...] [--follower <USER>...] [--custom-field <KEY=VALUE>...] [--interactive] [--output <FORMAT>]",
                vec![
                    positional("parent", "Parent task identifier.", true),
                    flag("name", "Task name; required unless `--interactive` supplies it.", false),
                    flag("assignee", "Assignee identifier (gid or email).", false),
                    flag("due-on", "All-day due date; natural language accepted.", false),
                    flag("due-at", "Timed due date; natural language accepted.", false),
                    flag("start-on", "All-day start date; natural language accepted.", false),
                    flag("start-at", "Timed start date; natural language accepted.", false),
                    flag("tag", "Tag identifier; repeat for multiple tags.", false),
                    flag("follower", "Follower identifier; repeat for multiple followers.", false),
                    flag("custom-field", "Custom field assignment in KEY=VALUE form; repeat as needed.", false),
                    flag("interactive", "Prompt for missing values interactively.", false),
                    flag("output", "Output format override.", false),
                ],
                vec![shape("task-subtasks-create-output", "json|table|detail", "Created subtask representation.")],
            ),
            command("task subtasks convert", "Attach a task under a new parent or return it to the root level.", "asana-cli task subtasks convert <TASK> [--parent <TASK> | --root]", vec![], vec![shape("task-subtasks-convert-output", "text", "Confirmation text after conversion.")]),
            router_command("task depends-on", "Namespace for dependency management.", "asana-cli task depends-on <SUBCOMMAND>"),
            command("task depends-on list", "List task dependencies.", "asana-cli task depends-on list <TASK> [--output <FORMAT>]", vec![], vec![shape("task-dependency-list-output", "json|table|detail", "Dependency list output.")]),
            command("task depends-on add", "Add dependencies to a task.", "asana-cli task depends-on add <TASK> --dependency <TASK>...", vec![], vec![shape("task-dependency-add-output", "text", "Confirmation text after dependency creation.")]),
            command("task depends-on remove", "Remove dependencies from a task.", "asana-cli task depends-on remove <TASK> --dependency <TASK>...", vec![], vec![shape("task-dependency-remove-output", "text", "Confirmation text after dependency removal.")]),
            router_command("task blocks", "Namespace for dependent management.", "asana-cli task blocks <SUBCOMMAND>"),
            command("task blocks list", "List tasks blocked by the target task.", "asana-cli task blocks list <TASK> [--output <FORMAT>]", vec![], vec![shape("task-blocks-list-output", "json|table|detail", "Dependent list output.")]),
            command("task blocks add", "Add dependents to a task.", "asana-cli task blocks add <TASK> --dependent <TASK>...", vec![], vec![shape("task-blocks-add-output", "text", "Confirmation text after dependent creation.")]),
            command("task blocks remove", "Remove dependents from a task.", "asana-cli task blocks remove <TASK> --dependent <TASK>...", vec![], vec![shape("task-blocks-remove-output", "text", "Confirmation text after dependent removal.")]),
            router_command("task projects", "Namespace for task-project membership.", "asana-cli task projects <SUBCOMMAND>"),
            command("task projects add", "Associate a task with a project and optional section.", "asana-cli task projects add <TASK> --project <GID> [--section <GID>]", vec![], vec![shape("task-projects-add-output", "text", "Confirmation text after project assignment.")]),
            command("task projects remove", "Remove a task from a project.", "asana-cli task projects remove <TASK> --project <GID>", vec![], vec![shape("task-projects-remove-output", "text", "Confirmation text after project removal.")]),
            router_command("task followers", "Namespace for follower management.", "asana-cli task followers <SUBCOMMAND>"),
            command("task followers add", "Add followers to a task.", "asana-cli task followers add <TASK> --follower <USER>...", vec![], vec![shape("task-followers-add-output", "text", "Confirmation text after follower addition.")]),
            command("task followers remove", "Remove followers from a task.", "asana-cli task followers remove <TASK> --follower <USER>...", vec![], vec![shape("task-followers-remove-output", "text", "Confirmation text after follower removal.")]),
            router_command("task tags", "Namespace for task-tag association.", "asana-cli task tags <SUBCOMMAND>"),
            command("task tags add", "Add a tag to a task.", "asana-cli task tags add <TASK> --tag <TAG>", vec![], vec![shape("task-tags-add-output", "text", "Confirmation text after tag addition.")]),
            command("task tags remove", "Remove a tag from a task.", "asana-cli task tags remove <TASK> --tag <TAG>", vec![], vec![shape("task-tags-remove-output", "text", "Confirmation text after tag removal.")]),
            command("task tags list", "List tags attached to a task.", "asana-cli task tags list <TASK> [--format <FORMAT>]", vec![], vec![shape("task-tags-list-output", "json|table|detail", "Task tag list output.")]),
            router_command("task comments", "Namespace for comment/story management.", "asana-cli task comments <SUBCOMMAND>"),
            command("task comments list", "List comments on a task.", "asana-cli task comments list <TASK> [--limit <N>] [--format <FORMAT>]", vec![], vec![shape("task-comments-list-output", "json|table|detail", "Comment list output.")]),
            command("task comments add", "Add a comment to a task.", "asana-cli task comments add <TASK> --text <TEXT> [--pin] [--format <FORMAT>]", vec![], vec![shape("task-comments-add-output", "json|detail", "Created comment representation.")]),
            command("task comments show", "Show one comment by story id.", "asana-cli task comments show <COMMENT> [--format <FORMAT>]", vec![], vec![shape("task-comments-show-output", "json|detail", "Comment detail output.")]),
            command("task comments update", "Update or pin/unpin a comment.", "asana-cli task comments update <COMMENT> [--text <TEXT>] [--pin] [--unpin] [--format <FORMAT>]", vec![], vec![shape("task-comments-update-output", "json|detail", "Updated comment representation.")]),
            command("task comments delete", "Delete a comment.", "asana-cli task comments delete <COMMENT> [--yes]", vec![], vec![shape("task-comments-delete-output", "text", "Confirmation text after deletion.")]),
            router_command("task attachments", "Namespace for attachment management.", "asana-cli task attachments <SUBCOMMAND>"),
            command("task attachments list", "List attachments on a task.", "asana-cli task attachments list <TASK> [--limit <N>] [--format <FORMAT>]", vec![], vec![shape("task-attachments-list-output", "json|table|detail", "Attachment list output.")]),
            command("task attachments upload", "Upload a file attachment to a task.", "asana-cli task attachments upload <TASK> --file <PATH> [--name <FILENAME>] [--format <FORMAT>]", vec![], vec![shape("task-attachments-upload-output", "json|detail", "Uploaded attachment representation.")]),
            command("task attachments download", "Download an attachment to a local file path.", "asana-cli task attachments download <ATTACHMENT> --output <PATH>", vec![], vec![shape("task-attachments-download-output", "file", "Binary attachment written to the requested output path.")]),
            command("task attachments show", "Show attachment metadata.", "asana-cli task attachments show <ATTACHMENT> [--format <FORMAT>]", vec![], vec![shape("task-attachments-show-output", "json|detail", "Attachment detail output.")]),
            command("task attachments delete", "Delete an attachment.", "asana-cli task attachments delete <ATTACHMENT> [--yes]", vec![], vec![shape("task-attachments-delete-output", "text", "Confirmation text after deletion.")]),
            command("task move-to-section", "Move a task into a section within a project.", "asana-cli task move-to-section <TASK> --project <GID> --section <GID>", vec![], vec![shape("task-move-to-section-output", "text", "Confirmation text after the move operation.")]),
            router_command("project", "Namespace for project CRUD and member management.", "asana-cli project <SUBCOMMAND>"),
            command("project list", "List projects with filters, saved filters, and output selection.", "asana-cli project list [--workspace <GID>] [--team <GID>] [--archived <BOOL>] [--sort <FIELD>] [--output <FORMAT>] [--filter <EXPR>...] [--filter-saved <NAME>...] [--save-filter <NAME>] [--limit <N>] [--fields <FIELD>...]", vec![], vec![shape("project-list-output", "json|table|detail", "Project list output.")]),
            command("project show", "Show one project by gid or by name, optionally including members and status summaries.", "asana-cli project show <PROJECT> [--by-name] [--output <FORMAT>] [--fields <FIELD>...] [--include-members] [--status-limit <N>]", vec![], vec![shape("project-show-output", "json|table|detail", "Project detail output.")]),
            command("project create", "Create a project with optional template variables, members, and custom fields.", "asana-cli project create [--name <TEXT>] [--workspace <GID>] [--team <GID>] [--notes <TEXT>] [--color <COLOR>] [--start-on <DATE>] [--due-on <DATE>] [--owner <ID>] [--public <BOOL>] [--template <NAME|PATH>] [--member <USER>...] [--custom-field <KEY=VALUE>...] [--var <KEY=VALUE>...] [--interactive] [--output <FORMAT>]", vec![], vec![shape("project-create-output", "json|table|detail", "Created project representation.")]),
            command("project update", "Update project metadata, ownership, visibility, and archive state.", "asana-cli project update <PROJECT> [--by-name] ...", vec![], vec![shape("project-update-output", "json|table|detail", "Updated project representation.")]),
            command("project delete", "Delete a project.", "asana-cli project delete <PROJECT> [--by-name] [--force]", vec![], vec![shape("project-delete-output", "text", "Confirmation text after deletion.")]),
            router_command("project members", "Namespace for project member management.", "asana-cli project members <SUBCOMMAND>"),
            command("project members list", "List project members.", "asana-cli project members list <PROJECT> [--by-name] [--output <FORMAT>]", vec![], vec![shape("project-members-list-output", "json|table|detail", "Project member list output.")]),
            command(
                "project members add",
                "Add members to a project.",
                "asana-cli project members add <PROJECT> [--by-name] <USER>... [--role <ROLE>]",
                vec![
                    positional("project", "Project identifier or name.", true),
                    flag("by-name", "Treat the project argument as a name instead of a gid.", false),
                    positional("members", "One or more users to add.", true),
                    flag("role", "Optional member permission role.", false),
                ],
                vec![shape("project-members-add-output", "text", "Confirmation text after adding members.")],
            ),
            command("project members remove", "Remove members from a project.", "asana-cli project members remove <PROJECT> [--by-name] <USER>...", vec![], vec![shape("project-members-remove-output", "text", "Confirmation text after removing members.")]),
            command("project members update", "Update an existing member role.", "asana-cli project members update <PROJECT> [--by-name] [--membership <ID> | --member <USER>] --role <ROLE>", vec![], vec![shape("project-members-update-output", "text", "Confirmation text after updating the membership role.")]),
            router_command("section", "Namespace for section operations.", "asana-cli section <SUBCOMMAND>"),
            command("section list", "List sections in a project.", "asana-cli section list --project <GID> [--output <FORMAT>]", vec![], vec![shape("section-list-output", "json|table|csv", "Section list output.")]),
            command("section show", "Show one section.", "asana-cli section show <SECTION> [--fields <FIELD>...] [--output <FORMAT>]", vec![], vec![shape("section-show-output", "json|table|detail", "Section detail output.")]),
            command(
                "section create",
                "Create a section inside a project.",
                "asana-cli section create --name <TEXT> --project <GID> [--insert-before <SECTION>] [--insert-after <SECTION>] [--output <FORMAT>]",
                vec![
                    flag("name", "Section name.", true),
                    flag("project", "Project gid in which to create the section.", true),
                    flag("insert-before", "Insert before this section gid.", false),
                    flag("insert-after", "Insert after this section gid.", false),
                    flag("output", "Output format override.", false),
                ],
                vec![shape("section-create-output", "json|table|detail", "Created section representation.")],
            ),
            command("section tasks", "List tasks in a section.", "asana-cli section tasks <SECTION> [--fields <FIELD>...] [--output <FORMAT>]", vec![], vec![shape("section-tasks-output", "json|table|csv", "Task list for a section.")]),
            router_command("tag", "Namespace for tag CRUD.", "asana-cli tag <SUBCOMMAND>"),
            command("tag list", "List tags in a workspace.", "asana-cli tag list [--workspace <GID>] [--limit <N>] [--format <FORMAT>]", vec![], vec![shape("tag-list-output", "table|detail|json", "Tag list output.")]),
            command("tag show", "Show one tag.", "asana-cli tag show <TAG> [--format <FORMAT>]", vec![], vec![shape("tag-show-output", "detail|json", "Tag detail output.")]),
            command(
                "tag create",
                "Create a tag in a workspace.",
                "asana-cli tag create --name <TEXT> [--workspace <GID>] [--color <COLOR>] [--notes <TEXT>] [--format <FORMAT>]",
                vec![
                    flag("name", "Tag name.", true),
                    flag("workspace", "Workspace gid; required unless a default workspace is configured.", false),
                    flag("color", "Tag color slug.", false),
                    flag("notes", "Optional tag notes or description.", false),
                    flag("format", "Output format.", false),
                ],
                vec![shape("tag-create-output", "detail|json|table", "Created tag representation.")],
            ),
            command("tag update", "Update a tag.", "asana-cli tag update <TAG> [--name <TEXT>] [--color <COLOR>] [--notes <TEXT>] [--clear-notes] [--format <FORMAT>]", vec![], vec![shape("tag-update-output", "detail|json|table", "Updated tag representation.")]),
            command("tag delete", "Delete a tag.", "asana-cli tag delete <TAG> [--yes]", vec![], vec![shape("tag-delete-output", "text", "Confirmation text after deletion.")]),
            router_command("custom-field", "Namespace for custom field inspection.", "asana-cli custom-field <SUBCOMMAND>"),
            command(
                "custom-field list",
                "List custom fields in a workspace.",
                "asana-cli custom-field list [--workspace <GID>] [--limit <N>] [--format <FORMAT>]",
                vec![
                    flag("workspace", "Workspace gid; required unless a default workspace is configured.", false),
                    flag("limit", "Maximum number of custom fields to retrieve.", false),
                    flag("format", "Output format.", false),
                ],
                vec![shape("custom-field-list-output", "table|detail|json", "Custom field list output.")],
            ),
            command("custom-field show", "Show one custom field.", "asana-cli custom-field show <GID> [--format <FORMAT>]", vec![], vec![shape("custom-field-show-output", "detail|json", "Custom field detail output.")]),
            router_command("workspace", "Namespace for workspace inspection.", "asana-cli workspace <SUBCOMMAND>"),
            command("workspace list", "List workspaces for the authenticated user.", "asana-cli workspace list [--limit <N>] [--format <FORMAT>]", vec![], vec![shape("workspace-list-output", "table|detail|json", "Workspace list output.")]),
            command(
                "workspace show",
                "Show one workspace.",
                "asana-cli workspace show <GID> [--format <FORMAT>]",
                vec![
                    positional("gid", "Workspace identifier.", true),
                    flag("format", "Output format.", false),
                ],
                vec![shape("workspace-show-output", "detail|json", "Workspace detail output.")],
            ),
            router_command("user", "Namespace for user inspection.", "asana-cli user <SUBCOMMAND>"),
            command("user list", "List users in a workspace.", "asana-cli user list [--workspace <GID>] [--limit <N>] [--format <FORMAT>]", vec![], vec![shape("user-list-output", "table|detail|json", "User list output.")]),
            command("user show", "Show one user.", "asana-cli user show <GID> [--format <FORMAT>]", vec![], vec![shape("user-show-output", "detail|json", "User detail output.")]),
            command(
                "user me",
                "Show the current authenticated user.",
                "asana-cli user me [--format <FORMAT>]",
                vec![flag("format", "Output format.", false)],
                vec![shape("user-me-output", "detail|json", "Current-user detail output.")],
            ),
            command("completions", "Generate shell completion scripts.", "asana-cli completions <SHELL>", vec![], vec![shape("completions-script", "shell script", "Completion script written to stdout.")]),
            command(
                "manpage",
                "Print the bundled roff manpage or write it into a directory.",
                "asana-cli manpage [--dir <PATH>]",
                vec![flag("dir", "Output directory for `asana-cli.1`; stdout is used when omitted.", false)],
                vec![
                    shape("manpage-stdout", "roff", "The bundled manpage content written to stdout."),
                    shape("manpage-file", "file", "Writes `asana-cli.1` into the requested directory."),
                ],
            ),
            command("doctor", "Run shared health checks plus tool-specific config checks.", "asana-cli doctor", vec![], vec![shape("doctor-output", "text", "Doctor report with configuration and installation status.")]),
            command("update", "Legacy self-update entrypoint that now prints manual-upgrade instructions and exits non-zero.", "asana-cli update [--version <VERSION>] [--force] [--install-dir <PATH>]", vec![], vec![shape("update-output", "stderr text", "Manual upgrade instructions; the self-update path is intentionally disabled.")]),
        ],
        arguments: vec![],
        environment_variables: vec![
            env_var("ASANA_PAT", "Personal Access Token used for API requests when not stored in the config file.", true),
            env_var("ASANA_BASE_URL", "Override for private or mock Asana API base URLs.", false),
            env_var("ASANA_WORKSPACE", "Default workspace gid override.", false),
            env_var("ASANA_ASSIGNEE", "Default assignee identifier override.", false),
            env_var("ASANA_PROJECT", "Default project gid override.", false),
            env_var("ASANA_CLI_CONFIG_HOME", "Override directory containing `config.toml`.", false),
            env_var("ASANA_CLI_DATA_HOME", "Override directory containing cache, templates, and filters.", false),
        ],
        config_files: vec![AgentConfigFile {
            path: "$ASANA_CLI_CONFIG_HOME/config.toml or platform default config file".to_owned(),
            purpose: "Stores API base URL, default workspace/assignee/project, and an optional persisted Personal Access Token.".to_owned(),
        }],
        default_paths: vec![
            AgentPath {
                name: "config-file".to_owned(),
                path: "$ASANA_CLI_CONFIG_HOME/config.toml".to_owned(),
                purpose: "Resolved config file when the config-home override is set.".to_owned(),
            },
            AgentPath {
                name: "data-dir".to_owned(),
                path: "$ASANA_CLI_DATA_HOME".to_owned(),
                purpose: "Resolved persistent data directory when the data-home override is set.".to_owned(),
            },
            AgentPath {
                name: "cache-dir".to_owned(),
                path: "$ASANA_CLI_DATA_HOME/cache".to_owned(),
                purpose: "API response cache directory used by the client builder.".to_owned(),
            },
            AgentPath {
                name: "templates-dir".to_owned(),
                path: "$ASANA_CLI_DATA_HOME/templates".to_owned(),
                purpose: "Reusable local project templates.".to_owned(),
            },
            AgentPath {
                name: "filters-dir".to_owned(),
                path: "$ASANA_CLI_DATA_HOME/filters".to_owned(),
                purpose: "Saved project filter definitions.".to_owned(),
            },
        ],
        output_shapes: vec![
            shape("agent-help", "yaml", "Canonical agent-facing reference document written by `asana-cli --agent-help`."),
            shape("agent-skill", "markdown with yaml front matter", "Claude-style skill file rendered from the same source as `--agent-help`."),
        ],
        examples: vec![
            AgentExample {
                name: "persist-token".to_owned(),
                command: "asana-cli config set token --token \"$ASANA_PAT\"".to_owned(),
                description: "Persist a Personal Access Token in the config file instead of relying on an environment variable.".to_owned(),
            },
            AgentExample {
                name: "list-my-tasks-as-json".to_owned(),
                command: "asana-cli task list --workspace ws-123 --output json".to_owned(),
                description: "List tasks for a workspace in machine-readable JSON.".to_owned(),
            },
            AgentExample {
                name: "create-project-from-template".to_owned(),
                command: "asana-cli project create --name \"Roadmap\" --workspace ws-123 --template kickoff --var owner=alice@example.com --output json".to_owned(),
                description: "Create a project while applying template variables and requesting JSON output.".to_owned(),
            },
            AgentExample {
                name: "download-attachment".to_owned(),
                command: "asana-cli task attachments download 12345 --output ./brief.pdf".to_owned(),
                description: "Download an attachment to a local file path.".to_owned(),
            },
            AgentExample {
                name: "inspect-workspaces".to_owned(),
                command: "asana-cli workspace list --format json".to_owned(),
                description: "List workspaces before choosing a default workspace gid.".to_owned(),
            },
            AgentExample {
                name: "generate-manpage".to_owned(),
                command: "asana-cli manpage --dir ./dist/man".to_owned(),
                description: "Write `asana-cli.1` into a documentation output directory.".to_owned(),
            },
        ],
        failure_modes: vec![
            AgentFailureMode {
                name: "missing-token".to_owned(),
                symptom: "API commands fail with `no Personal Access Token found; run `asana-cli config set token`` or equivalent credential messages.".to_owned(),
                resolution: "Set `ASANA_PAT` or persist a token with `config set token` before running networked commands.".to_owned(),
            },
            AgentFailureMode {
                name: "authentication-failed".to_owned(),
                symptom: "`config test` or API commands report authentication failure or 401 responses.".to_owned(),
                resolution: "Rotate the token, update the stored value, and retry the command.".to_owned(),
            },
            AgentFailureMode {
                name: "rate-limited".to_owned(),
                symptom: "The Asana API rejects requests and tells the client to retry after a delay.".to_owned(),
                resolution: "Wait for the reported retry interval, reduce request volume, and avoid unnecessary batch retries.".to_owned(),
            },
            AgentFailureMode {
                name: "workspace-or-default-missing".to_owned(),
                symptom: "Workspace-scoped commands fail because `--workspace` was omitted and no default workspace is configured.".to_owned(),
                resolution: "Pass `--workspace <GID>`, set `ASANA_WORKSPACE`, or store a default workspace with `config set workspace`.".to_owned(),
            },
            AgentFailureMode {
                name: "local-file-error".to_owned(),
                symptom: "Batch commands or attachment uploads/downloads fail because the input or output path is missing, unreadable, or malformed.".to_owned(),
                resolution: "Validate the local file path and expected format (JSON/CSV or writable output path) before retrying.".to_owned(),
            },
        ],
        operator_mistakes: vec![
            AgentOperatorMistake {
                name: "agent-doc-flag-placement".to_owned(),
                symptom: "Passing `--agent-help` or `--agent-skill` after a subcommand produces a normal clap parse error instead of agent docs.".to_owned(),
                correction: "Place the agent-doc flag immediately after `asana-cli` with no subcommand.".to_owned(),
            },
            AgentOperatorMistake {
                name: "environment-overrides-hidden-config".to_owned(),
                symptom: "Config output or API behavior does not match the stored config because `ASANA_BASE_URL`, `ASANA_WORKSPACE`, `ASANA_ASSIGNEE`, or `ASANA_PROJECT` are overriding persisted values.".to_owned(),
                correction: "Check environment variables first when observed runtime behavior differs from the config file.".to_owned(),
            },
            AgentOperatorMistake {
                name: "wrong-output-assumption".to_owned(),
                symptom: "Automation expects JSON but the command defaults to table/detail output, or expects a table while `--output json` is enabled.".to_owned(),
                correction: "Set the explicit output flag each time an automated consumer depends on a specific shape.".to_owned(),
            },
            AgentOperatorMistake {
                name: "self-update-expectation".to_owned(),
                symptom: "Calling `asana-cli update` is expected to install a new version automatically, but the command only prints manual-upgrade instructions and exits non-zero.".to_owned(),
                correction: "Install updates through the release channel instead of relying on the removed self-update path.".to_owned(),
            },
        ],
        constraints: vec![
            "Agent-doc flags are exact top-level requests only.".to_owned(),
            "Networked commands require valid authentication and may fail on rate limits or offline mode.".to_owned(),
            "Batch operations depend on well-formed local JSON or CSV files.".to_owned(),
        ],
    }
}

fn env_var(name: &str, description: &str, required: bool) -> AgentEnvironmentVar {
    AgentEnvironmentVar {
        name: name.to_owned(),
        description: description.to_owned(),
        required,
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

fn shape(name: &str, format: &str, description: &str) -> AgentOutputShape {
    AgentOutputShape {
        name: name.to_owned(),
        format: format.to_owned(),
        description: description.to_owned(),
    }
}

fn router_command(name: &str, summary: &str, usage: &str) -> AgentCommand {
    command(
        name,
        summary,
        usage,
        vec![],
        vec![shape(
            "dispatch",
            "none",
            "This command only dispatches to nested subcommands.",
        )],
    )
}

fn command(
    name: &str,
    summary: &str,
    usage: &str,
    arguments: Vec<AgentArgument>,
    output_shapes: Vec<AgentOutputShape>,
) -> AgentCommand {
    AgentCommand {
        name: name.to_owned(),
        summary: summary.to_owned(),
        usage: usage.to_owned(),
        arguments,
        output_shapes,
    }
}

/// Parse and execute CLI commands, returning the desired process exit code.
///
/// # Errors
/// Returns an error when command execution fails prior to producing an exit code.
pub fn run() -> Result<i32> {
    let cli = Cli::parse();
    debug!(?cli, "parsed CLI arguments");

    let mut config = Config::load()?;
    debug!(
        config_path = %config.path().display(),
        "configuration handle prepared"
    );

    let exit_code = match cli.command {
        Commands::Version => run_standard_command::<Cli, AsanaCliDoctor>(
            &TOOL_SPEC,
            &StandardCommand::Version { json: false },
            Some(&AsanaCliDoctor),
        ),
        Commands::License => run_standard_command::<Cli, AsanaCliDoctor>(
            &TOOL_SPEC,
            &StandardCommand::License,
            Some(&AsanaCliDoctor),
        ),
        Commands::Config { command } => {
            handle_config_command(command, &mut config)?;
            0
        }
        Commands::Task { command } => {
            task::handle_task_command(*command, &config)?;
            0
        }
        Commands::Project { command } => {
            handle_project_command(*command, &config)?;
            0
        }
        Commands::Section { command } => {
            handle_section_command(*command, &config)?;
            0
        }
        Commands::Tag { command } => {
            handle_tag_command(*command, &config)?;
            0
        }
        Commands::CustomField { command } => {
            handle_custom_field_command(*command, &config)?;
            0
        }
        Commands::Workspace { command } => {
            handle_workspace_command(*command, &config)?;
            0
        }
        Commands::User { command } => {
            handle_user_command(*command, &config)?;
            0
        }
        Commands::Completions { shell } => run_standard_command::<Cli, AsanaCliDoctor>(
            &TOOL_SPEC,
            &StandardCommand::Completions { shell },
            Some(&AsanaCliDoctor),
        ),
        Commands::Manpage { dir } => {
            write_manpage(dir)?;
            0
        }
        Commands::Doctor => {
            let exit = run_standard_command::<Cli, AsanaCliDoctor>(
                &TOOL_SPEC,
                &StandardCommand::Doctor,
                Some(&AsanaCliDoctor),
            );
            info!(exit_code = exit, "doctor command completed");
            exit
        }
        Commands::Update {
            version,
            force,
            install_dir,
        } => run_standard_command::<Cli, AsanaCliDoctor>(
            &TOOL_SPEC,
            &StandardCommand::Update {
                version,
                force,
                install_dir,
            },
            Some(&AsanaCliDoctor),
        ),
    };

    Ok(exit_code)
}

fn write_manpage(dir: Option<PathBuf>) -> Result<()> {
    if let Some(path) = dir {
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create manpage directory {}", path.display()))?;
        let output = path.join("asana-cli.1");
        let mut file = File::create(&output)
            .with_context(|| format!("failed to create manpage file {}", output.display()))?;
        write!(file, "{MANPAGE_SOURCE}")
            .map_err(|err| anyhow!("failed to write manpage: {err}"))?;
        println!("Man page written to {}", output.display());
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        write!(handle, "{MANPAGE_SOURCE}")
            .map_err(|err| anyhow!("failed to write manpage: {err}"))?;
    }

    Ok(())
}

fn handle_config_command(command: ConfigCommand, config: &mut Config) -> Result<()> {
    match command {
        ConfigCommand::Set { command } => handle_config_set(command, config),
        ConfigCommand::Get => {
            handle_config_get(config);
            Ok(())
        }
        ConfigCommand::Test => handle_config_test(config),
    }
}

fn handle_config_set(command: ConfigSetCommand, config: &mut Config) -> Result<()> {
    match command {
        ConfigSetCommand::Token { token } => {
            let value = match token {
                Some(value) => value,
                None => rpassword::prompt_password("Enter Personal Access Token: ")
                    .context("failed to read token from prompt")?,
            };

            if value.trim().is_empty() {
                return Err(anyhow!("token value cannot be empty"));
            }

            let secret = SecretString::new(value.into());
            config
                .store_personal_access_token(&secret)
                .context("failed to store Personal Access Token")?;
            println!("Personal Access Token stored in configuration file.");
            Ok(())
        }
        ConfigSetCommand::Workspace { workspace, clear } => {
            if clear {
                config
                    .set_default_workspace(None)
                    .context("failed to clear default workspace")?;
                println!("Default workspace cleared.");
                return Ok(());
            }

            let value = workspace
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow!("provide --workspace <gid> or use --clear"))?;

            config
                .set_default_workspace(Some(value.to_string()))
                .context("failed to store default workspace")?;
            println!("Default workspace stored in configuration file.");
            Ok(())
        }
        ConfigSetCommand::Assignee { assignee, clear } => {
            if clear {
                config
                    .set_default_assignee(None)
                    .context("failed to clear default assignee")?;
                println!("Default assignee cleared.");
                return Ok(());
            }

            let value = assignee
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow!("provide --assignee <id> or use --clear"))?;

            config
                .set_default_assignee(Some(value.to_string()))
                .context("failed to store default assignee")?;
            println!("Default assignee stored in configuration file.");
            Ok(())
        }
        ConfigSetCommand::Project { project, clear } => {
            if clear {
                config
                    .set_default_project(None)
                    .context("failed to clear default project")?;
                println!("Default project cleared.");
                return Ok(());
            }

            let value = project
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow!("provide --project <gid> or use --clear"))?;

            config
                .set_default_project(Some(value.to_string()))
                .context("failed to store default project")?;
            println!("Default project stored in configuration file.");
            Ok(())
        }
    }
}

fn handle_config_get(config: &Config) {
    println!("Configuration file: {}", config.path().display());
    println!("API base URL: {}", config.effective_api_base_url());
    println!(
        "Default workspace: {}",
        config
            .default_workspace()
            .filter(|workspace| !workspace.is_empty())
            .unwrap_or("not set")
    );
    println!(
        "Default assignee: {}",
        config
            .default_assignee()
            .filter(|assignee| !assignee.is_empty())
            .unwrap_or("not set")
    );
    println!(
        "Default project: {}",
        config
            .default_project()
            .filter(|project| !project.is_empty())
            .unwrap_or("not set")
    );

    if let Some(_token) = config.personal_access_token() {
        let status = if config.environment_token_available() {
            "provided via environment variable"
        } else if config.has_persisted_token() {
            "stored in configuration file"
        } else {
            "available"
        };
        println!("Personal Access Token: {status}");
    } else {
        println!("Personal Access Token: not set");
    }
}

fn handle_config_test(config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialise async runtime")?;

    runtime.block_on(async move {
        match client.get_current_user().await {
            Ok(payload) => {
                let user_name = payload
                    .get("data")
                    .and_then(|data| data.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or("unknown user");
                println!("Personal Access Token validated for {user_name}.");
                Ok(())
            }
            Err(ApiError::Authentication(_)) => Err(anyhow!(
                "authentication failed; verify your Personal Access Token"
            )),
            Err(ApiError::RateLimited { retry_after, .. }) => Err(anyhow!(
                "Asana rate limited the request. Retry after {:.1} seconds",
                retry_after.as_secs_f32()
            )),
            Err(ApiError::Offline { .. }) => Err(anyhow!(
                "offline mode enabled; disable offline mode to contact Asana"
            )),
            Err(err) => Err(anyhow!(err)),
        }
    })
}

pub(super) fn build_api_client(config: &Config) -> Result<ApiClient> {
    let token = config.personal_access_token().ok_or_else(|| {
        anyhow!("no Personal Access Token found; run `asana-cli config set token`")
    })?;

    let auth_token = AuthToken::new(token);
    let cache_dir = config.cache_dir().to_path_buf();

    let client = ApiClient::builder(auth_token)
        .base_url(config.effective_api_base_url().to_string())
        .cache_dir(cache_dir)
        .build()?;

    Ok(client)
}

fn handle_project_command(command: ProjectCommand, config: &Config) -> Result<()> {
    project::handle_project_command(command, config)
}

fn handle_section_command(command: SectionCommand, config: &Config) -> Result<()> {
    section::execute_section_command(command, config)
}

fn handle_tag_command(command: TagCommand, config: &Config) -> Result<()> {
    tag::handle_tag_command(command, config)
}

fn handle_custom_field_command(command: CustomFieldCommand, config: &Config) -> Result<()> {
    custom_field::handle_custom_field_command(command, config)
}

fn handle_workspace_command(command: WorkspaceCommand, config: &Config) -> Result<()> {
    workspace::handle_workspace_command(command, config)
}

fn handle_user_command(command: UserCommand, config: &Config) -> Result<()> {
    user::handle_user_command(command, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tftio_cli_common::agent_docs::{assert_argument_coverage, assert_command_coverage};

    fn command_arguments<'a>(doc: &'a AgentDoc, name: &str) -> &'a [AgentArgument] {
        doc.commands
            .iter()
            .find(|command| command.name == name)
            .map(|command| command.arguments.as_slice())
            .unwrap_or_else(|| panic!("documented command missing from agent doc: {name}"))
    }

    fn assert_doc_argument_coverage(command_path: &[&str], arguments: &[AgentArgument]) {
        let long_flags = arguments
            .iter()
            .filter(|argument| !argument.positional)
            .map(|argument| argument.name.as_str())
            .collect::<Vec<_>>();
        let positionals = arguments
            .iter()
            .filter(|argument| argument.positional)
            .map(|argument| argument.name.as_str())
            .collect::<Vec<_>>();
        assert_argument_coverage::<Cli>(command_path, &long_flags, &positionals, &[]);
    }

    #[test]
    fn agent_help_documentation_covers_asana_cli_tree() {
        let doc = agent_doc();
        let documented_paths = doc
            .commands
            .iter()
            .map(|command| command.name.as_str())
            .collect::<Vec<_>>();
        assert_command_coverage::<Cli>(&documented_paths);
    }

    #[test]
    fn agent_help_documentation_covers_selected_argument_surfaces() {
        let doc = agent_doc();
        assert_doc_argument_coverage(&["config", "set", "token"], command_arguments(&doc, "config set token"));
        assert_doc_argument_coverage(&["task", "subtasks", "create"], command_arguments(&doc, "task subtasks create"));
        assert_doc_argument_coverage(&["project", "members", "add"], command_arguments(&doc, "project members add"));
        assert_doc_argument_coverage(&["section", "create"], command_arguments(&doc, "section create"));
        assert_doc_argument_coverage(&["tag", "create"], command_arguments(&doc, "tag create"));
        assert_doc_argument_coverage(&["custom-field", "list"], command_arguments(&doc, "custom-field list"));
        assert_doc_argument_coverage(&["workspace", "show"], command_arguments(&doc, "workspace show"));
        assert_doc_argument_coverage(&["user", "me"], command_arguments(&doc, "user me"));
        assert_doc_argument_coverage(&["manpage"], command_arguments(&doc, "manpage"));
    }

    #[test]
    fn agent_help_document_mentions_async_groups_env_and_output_shapes() {
        let rendered = render_agent_skill(&agent_doc());

        assert!(rendered.contains("task subtasks"));
        assert!(rendered.contains("project members"));
        assert!(rendered.contains("ASANA_PAT"));
        assert!(rendered.contains("network"));
        assert!(rendered.contains("json"));
        assert!(rendered.contains("Operator mistakes"));
    }
}
