use crate::models::Status;
use clap::{Parser, Subcommand};
use tftio_cli_common::{
    AgentArgument, AgentCommand, AgentConfigFile, AgentDoc, AgentEnvironmentVar, AgentExample,
    AgentFailureMode, AgentOperatorMistake, AgentOutputShape, AgentPath, AgentSection, AgentTool,
    AgentUsage,
};

#[derive(Parser, Debug)]
#[command(name = "todoer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        json: bool,
    },
    New {
        #[arg(long)]
        project: Option<String>,
        description: String,
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
    },
    Task {
        #[command(subcommand)]
        command: TaskCommand,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommand {
    Status {
        id: String,
    },
    Show {
        id: String,
    },
    Note {
        id: String,
        note: String,
    },
    Update {
        #[command(subcommand)]
        command: TaskUpdateCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskUpdateCommand {
    Status { id: String, status: Status },
}

#[must_use]
pub fn agent_doc() -> AgentDoc {
    AgentDoc {
        schema_version: "1".to_owned(),
        tool: AgentTool {
            name: "todoer".to_owned(),
            binary: "todoer".to_owned(),
            summary: "SQLite-backed todo manager with project-scoped task and note commands."
                .to_owned(),
        },
        usage: AgentUsage {
            invocation: "todoer <COMMAND>".to_owned(),
            notes: vec![
                "Use `todoer --agent-help` for canonical YAML and `todoer --agent-skill` for the markdown skill rendering.".to_owned(),
                "Normal CLI parsing still requires a top-level subcommand when agent-doc flags are absent.".to_owned(),
                "Project-scoped commands resolve the target project from `--project`, `.todoer.toml`, or the current Git repository name before touching SQLite.".to_owned(),
            ],
        },
        shared_sections: vec![
            AgentSection {
                title: "agent-doc flags".to_owned(),
                content: "`--agent-help` and `--agent-skill` are hidden top-level-only requests. They print to stdout and exit 0, but `todoer task --agent-help` is rejected because the raw-argv detector only accepts exact top-level invocations.".to_owned(),
            },
            AgentSection {
                title: "project resolution".to_owned(),
                content: "For `init`, `new`, and `list` without `--all`, the resolved project name comes from `--project` first, then the nearest `.todoer.toml` between the current directory and the home directory, then the Git repository directory name, and otherwise the command fails with `no project`.".to_owned(),
            },
            AgentSection {
                title: "json mode".to_owned(),
                content: "Every `--json` command writes a single JSON object to stdout. Success uses `ok_response` with `{ ok, command, data }`; failures use `err_response` with `{ ok: false, command, error: { code, message, details } }`. There is no JSONL stream mode.".to_owned(),
            },
            AgentSection {
                title: "sqlite behavior".to_owned(),
                content: "`init` creates the `projects`, `tasks`, and `task_notes` tables in the configured SQLite database. `new`, `list`, `task status`, `task show`, `task note`, and `task update status` all read or write the same database, and `task show` returns notes in reverse chronological order.".to_owned(),
            },
        ],
        commands: vec![
            AgentCommand {
                name: "init".to_owned(),
                summary: "Initialize SQLite state for the resolved project.".to_owned(),
                usage: "todoer init [--project <PROJECT>] [--json]".to_owned(),
                arguments: vec![
                    AgentArgument {
                        name: "project".to_owned(),
                        positional: false,
                        description: "Project name override used instead of `.todoer.toml` or Git discovery.".to_owned(),
                        required: false,
                    },
                    AgentArgument {
                        name: "json".to_owned(),
                        positional: false,
                        description: "Emit the `init` success or error envelope as JSON on stdout.".to_owned(),
                        required: false,
                    },
                ],
                output_shapes: vec![
                    AgentOutputShape {
                        name: "init-text".to_owned(),
                        format: "text".to_owned(),
                        description: "Two stdout lines: `project: <name>` then `db: <path>`.".to_owned(),
                    },
                    AgentOutputShape {
                        name: "init-json".to_owned(),
                        format: "json".to_owned(),
                        description: "Success envelope with `data.project`, `data.db_path`, and `data.schema_created`.".to_owned(),
                    },
                ],
            },
            AgentCommand {
                name: "new".to_owned(),
                summary: "Insert a new task into the resolved project's SQLite task list.".to_owned(),
                usage: "todoer new [--project <PROJECT>] [--json] <DESCRIPTION|->".to_owned(),
                arguments: vec![
                    AgentArgument {
                        name: "project".to_owned(),
                        positional: false,
                        description: "Project name override for the inserted task.".to_owned(),
                        required: false,
                    },
                    AgentArgument {
                        name: "description".to_owned(),
                        positional: true,
                        description: "Task description text. Pass `-` to read the description from stdin.".to_owned(),
                        required: true,
                    },
                    AgentArgument {
                        name: "json".to_owned(),
                        positional: false,
                        description: "Emit the inserted task as a JSON success envelope.".to_owned(),
                        required: false,
                    },
                ],
                output_shapes: vec![
                    AgentOutputShape {
                        name: "new-text".to_owned(),
                        format: "text".to_owned(),
                        description: "Single stdout line: `<uuid> <STATUS> <description>`.".to_owned(),
                    },
                    AgentOutputShape {
                        name: "new-json".to_owned(),
                        format: "json".to_owned(),
                        description: "Success envelope with `data.task`, including `id`, `project_key`, `created_at`, `description`, and `status`.".to_owned(),
                    },
                ],
            },
            AgentCommand {
                name: "list".to_owned(),
                summary: "List tasks from one resolved project or from every project with `--all`."
                    .to_owned(),
                usage: "todoer list [--project <PROJECT> | --all] [--json]".to_owned(),
                arguments: vec![
                    AgentArgument {
                        name: "project".to_owned(),
                        positional: false,
                        description: "Project name override for scoped listing.".to_owned(),
                        required: false,
                    },
                    AgentArgument {
                        name: "all".to_owned(),
                        positional: false,
                        description: "Disable project resolution and list tasks across all projects in the SQLite database.".to_owned(),
                        required: false,
                    },
                    AgentArgument {
                        name: "json".to_owned(),
                        positional: false,
                        description: "Emit the task array as a JSON success envelope.".to_owned(),
                        required: false,
                    },
                ],
                output_shapes: vec![
                    AgentOutputShape {
                        name: "list-table".to_owned(),
                        format: "text".to_owned(),
                        description: "Tab-separated table with `UUID`, `STATUS`, `CREATED_AT`, and `DESCRIPTION` columns.".to_owned(),
                    },
                    AgentOutputShape {
                        name: "list-json".to_owned(),
                        format: "json".to_owned(),
                        description: "Success envelope with `data.tasks` as an array of serialized task rows.".to_owned(),
                    },
                ],
            },
            AgentCommand {
                name: "task".to_owned(),
                summary: "Namespace for task inspection, note creation, and status updates."
                    .to_owned(),
                usage: "todoer task [--json] <SUBCOMMAND>".to_owned(),
                arguments: vec![AgentArgument {
                    name: "json".to_owned(),
                    positional: false,
                    description: "Apply JSON envelope output to `task status`, `task show`, `task note`, or `task update status`.".to_owned(),
                    required: false,
                }],
                output_shapes: vec![AgentOutputShape {
                    name: "task-dispatch".to_owned(),
                    format: "none".to_owned(),
                    description: "The `task` command itself dispatches to nested subcommands and does not emit output without one.".to_owned(),
                }],
            },
            AgentCommand {
                name: "task status".to_owned(),
                summary: "Read one task's description, status, and creation time.".to_owned(),
                usage: "todoer task [--json] status <ID>".to_owned(),
                arguments: vec![AgentArgument {
                    name: "id".to_owned(),
                    positional: true,
                    description: "Task UUID string.".to_owned(),
                    required: true,
                }],
                output_shapes: vec![
                    AgentOutputShape {
                        name: "task-status-text".to_owned(),
                        format: "text".to_owned(),
                        description: "Three stdout lines: description, status string, and created_at timestamp.".to_owned(),
                    },
                    AgentOutputShape {
                        name: "task-status-json".to_owned(),
                        format: "json".to_owned(),
                        description: "Success envelope with `data.description`, `data.status`, and `data.created_at`.".to_owned(),
                    },
                ],
            },
            AgentCommand {
                name: "task show".to_owned(),
                summary: "Read one task plus all stored notes.".to_owned(),
                usage: "todoer task [--json] show <ID>".to_owned(),
                arguments: vec![AgentArgument {
                    name: "id".to_owned(),
                    positional: true,
                    description: "Task UUID string.".to_owned(),
                    required: true,
                }],
                output_shapes: vec![
                    AgentOutputShape {
                        name: "task-show-text".to_owned(),
                        format: "text".to_owned(),
                        description: "Description, status, and created_at on the first three lines, then `- <note>` for each note.".to_owned(),
                    },
                    AgentOutputShape {
                        name: "task-show-json".to_owned(),
                        format: "json".to_owned(),
                        description: "Success envelope with task metadata plus `data.notes` as an array of note rows.".to_owned(),
                    },
                ],
            },
            AgentCommand {
                name: "task note".to_owned(),
                summary: "Append a note row to the SQLite `task_notes` table.".to_owned(),
                usage: "todoer task [--json] note <ID> <NOTE|->".to_owned(),
                arguments: vec![
                    AgentArgument {
                        name: "id".to_owned(),
                        positional: true,
                        description: "Task UUID string.".to_owned(),
                        required: true,
                    },
                    AgentArgument {
                        name: "note".to_owned(),
                        positional: true,
                        description: "Note body. Pass `-` to read the note from stdin.".to_owned(),
                        required: true,
                    },
                ],
                output_shapes: vec![
                    AgentOutputShape {
                        name: "task-note-text".to_owned(),
                        format: "text".to_owned(),
                        description: "Single stdout line: `note added: <note-id>`.".to_owned(),
                    },
                    AgentOutputShape {
                        name: "task-note-json".to_owned(),
                        format: "json".to_owned(),
                        description: "Success envelope with `data.note`, including SQLite autoincrement note id and timestamp.".to_owned(),
                    },
                ],
            },
            AgentCommand {
                name: "task update".to_owned(),
                summary: "Namespace for task mutation subcommands.".to_owned(),
                usage: "todoer task [--json] update <SUBCOMMAND>".to_owned(),
                arguments: vec![],
                output_shapes: vec![AgentOutputShape {
                    name: "task-update-dispatch".to_owned(),
                    format: "none".to_owned(),
                    description: "The `task update` command only dispatches to nested update operations.".to_owned(),
                }],
            },
            AgentCommand {
                name: "task update status".to_owned(),
                summary: "Change a task's stored status value.".to_owned(),
                usage: "todoer task [--json] update status <ID> <STATUS>".to_owned(),
                arguments: vec![
                    AgentArgument {
                        name: "id".to_owned(),
                        positional: true,
                        description: "Task UUID string.".to_owned(),
                        required: true,
                    },
                    AgentArgument {
                        name: "status".to_owned(),
                        positional: true,
                        description: "One of `NEW`, `IN-PROGRESS`, `COMPLETED`, or `ABANDONED`.".to_owned(),
                        required: true,
                    },
                ],
                output_shapes: vec![
                    AgentOutputShape {
                        name: "task-update-status-text".to_owned(),
                        format: "text".to_owned(),
                        description: "Single stdout line containing the updated status string.".to_owned(),
                    },
                    AgentOutputShape {
                        name: "task-update-status-json".to_owned(),
                        format: "json".to_owned(),
                        description: "Success envelope with `data.status`.".to_owned(),
                    },
                ],
            },
        ],
        arguments: vec![],
        environment_variables: vec![
            AgentEnvironmentVar {
                name: "XDG_CONFIG_HOME".to_owned(),
                description: "Overrides the config directory that contributes `todoer/config.toml`.".to_owned(),
                required: false,
            },
            AgentEnvironmentVar {
                name: "XDG_DATA_HOME".to_owned(),
                description: "Overrides the default data directory that contributes `todoer/todoer.db`.".to_owned(),
                required: false,
            },
            AgentEnvironmentVar {
                name: "HOME".to_owned(),
                description: "Fallback root for `~/.config/todoer/config.toml`, `~/.local/share/todoer/todoer.db`, and the `.todoer.toml` upward search boundary.".to_owned(),
                required: false,
            },
        ],
        config_files: vec![
            AgentConfigFile {
                path: ".todoer.toml".to_owned(),
                purpose: "Per-project file containing `project = \"<name>\"` for automatic project resolution.".to_owned(),
            },
            AgentConfigFile {
                path: "$XDG_CONFIG_HOME/todoer/config.toml or ~/.config/todoer/config.toml".to_owned(),
                purpose: "Optional global config file. Today it supports `db_path` to relocate the SQLite database.".to_owned(),
            },
        ],
        default_paths: vec![
            AgentPath {
                name: "config".to_owned(),
                path: "~/.config/todoer/config.toml".to_owned(),
                purpose: "Default config path when `XDG_CONFIG_HOME` is unset.".to_owned(),
            },
            AgentPath {
                name: "database".to_owned(),
                path: "~/.local/share/todoer/todoer.db".to_owned(),
                purpose: "Default SQLite database path when `db_path` and `XDG_DATA_HOME` are unset.".to_owned(),
            },
        ],
        output_shapes: vec![
            AgentOutputShape {
                name: "ok-response".to_owned(),
                format: "json".to_owned(),
                description: "Success envelope shared by every `--json` command: `{ ok: true, command, data }`.".to_owned(),
            },
            AgentOutputShape {
                name: "error-response".to_owned(),
                format: "json".to_owned(),
                description: "Failure envelope shared by every `--json` command: `{ ok: false, command, error: { code: \"ERROR\", message, details } }`.".to_owned(),
            },
        ],
        examples: vec![
            AgentExample {
                name: "init-current-project".to_owned(),
                command: "todoer init".to_owned(),
                description: "Initialize the SQLite schema for the project discovered from `.todoer.toml` or the current Git repository.".to_owned(),
            },
            AgentExample {
                name: "new-from-stdin".to_owned(),
                command: "printf 'Investigate parser bug\\n' | todoer new -".to_owned(),
                description: "Create a task from stdin when the description should not appear in the shell history.".to_owned(),
            },
            AgentExample {
                name: "list-json".to_owned(),
                command: "todoer list --json".to_owned(),
                description: "List tasks for the resolved project using the JSON envelope.".to_owned(),
            },
            AgentExample {
                name: "show-task-with-notes".to_owned(),
                command: "todoer task --json show 123e4567-e89b-12d3-a456-426614174000".to_owned(),
                description: "Return one task plus every stored note as structured JSON.".to_owned(),
            },
            AgentExample {
                name: "append-note-from-stdin".to_owned(),
                command: "printf 'Need reproduction steps\\n' | todoer task note 123e4567-e89b-12d3-a456-426614174000 -".to_owned(),
                description: "Append a note without placing the note body on the command line.".to_owned(),
            },
            AgentExample {
                name: "update-status".to_owned(),
                command: "todoer task update status 123e4567-e89b-12d3-a456-426614174000 COMPLETED".to_owned(),
                description: "Mark a task as completed in SQLite.".to_owned(),
            },
        ],
        failure_modes: vec![
            AgentFailureMode {
                name: "no-project".to_owned(),
                symptom: "`new`, `list`, or `init` fails with `no project`.".to_owned(),
                resolution: "Pass `--project`, add `.todoer.toml`, or run inside a Git repository whose directory name is the intended project.".to_owned(),
            },
            AgentFailureMode {
                name: "stdin-required".to_owned(),
                symptom: "`new -` or `task note <id> -` fails with `stdin required`.".to_owned(),
                resolution: "Pipe or redirect content into stdin whenever the description or note argument is `-`.".to_owned(),
            },
            AgentFailureMode {
                name: "invalid-status".to_owned(),
                symptom: "`task update status` rejects the supplied value.".to_owned(),
                resolution: "Use one of the clap-validated status strings: `NEW`, `IN-PROGRESS`, `COMPLETED`, or `ABANDONED`.".to_owned(),
            },
            AgentFailureMode {
                name: "config-or-db-error".to_owned(),
                symptom: "The command prints an I/O, TOML, or SQLite error.".to_owned(),
                resolution: "Check the config path, the configured `db_path`, and filesystem permissions, then retry.".to_owned(),
            },
        ],
        operator_mistakes: vec![
            AgentOperatorMistake {
                name: "assuming-list-is-global".to_owned(),
                symptom: "`todoer list` only returns one project's tasks.".to_owned(),
                correction: "Use `--all` for a cross-project listing or pass `--project` to select a different project explicitly.".to_owned(),
            },
            AgentOperatorMistake {
                name: "missing-stdin-for-dash".to_owned(),
                symptom: "Passing `-` for the description or note without piping content leaves the command with no input.".to_owned(),
                correction: "Provide stdin with `printf ... | todoer new -` or `printf ... | todoer task note <id> -`.".to_owned(),
            },
            AgentOperatorMistake {
                name: "agent-doc-flag-after-subcommand".to_owned(),
                symptom: "`todoer task --agent-help` does not return the agent document.".to_owned(),
                correction: "Request agent docs only as `todoer --agent-help` or `todoer --agent-skill`.".to_owned(),
            },
        ],
        constraints: vec![
            "Hidden agent-doc flags stay out of normal clap help output.".to_owned(),
            "Agent-doc requests are top-level only and never bypass normal parsing for subcommand placements.".to_owned(),
            "The current schema stores projects, tasks, and task_notes in SQLite; there is no remote sync or daemon.".to_owned(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tftio_cli_common::agent_docs::{assert_argument_coverage, assert_command_coverage};

    #[test]
    fn agent_doc_covers_todoer_command_tree() {
        assert_command_coverage::<Cli>(&[
            "init",
            "list",
            "new",
            "task",
            "task note",
            "task show",
            "task status",
            "task update",
            "task update status",
        ]);
    }

    #[test]
    fn agent_doc_covers_todoer_arguments() {
        assert_argument_coverage::<Cli>(&[], &[], &[], &[]);
        assert_argument_coverage::<Cli>(&["init"], &["json", "project"], &[], &[]);
        assert_argument_coverage::<Cli>(
            &["new"],
            &["json", "project"],
            &["description"],
            &[],
        );
        assert_argument_coverage::<Cli>(
            &["list"],
            &["all", "json", "project"],
            &[],
            &[],
        );
        assert_argument_coverage::<Cli>(&["task"], &["json"], &[], &[]);
        assert_argument_coverage::<Cli>(&["task", "status"], &[], &["id"], &[]);
        assert_argument_coverage::<Cli>(&["task", "show"], &[], &["id"], &[]);
        assert_argument_coverage::<Cli>(&["task", "note"], &[], &["id", "note"], &[]);
        assert_argument_coverage::<Cli>(&["task", "update"], &[], &[], &[]);
        assert_argument_coverage::<Cli>(
            &["task", "update", "status"],
            &[],
            &["id", "status"],
            &[],
        );
    }
}
