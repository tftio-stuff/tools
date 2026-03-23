//! CLI entry point for the `bce` (`BlueSky` Comment Extractor) binary.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use directories::ProjectDirs;
use tftio_cli_common::{
    DoctorChecks, FatalCliError, LicenseType, StandardCommand, StandardCommandMap, ToolSpec,
    command::maybe_run_standard_command, error::fatal_error, parse_and_exit,
    progress::make_spinner, workspace_tool,
};

use bsky_comment_extractor::cli::{Cli, Command, FetchArgs, QueryArgs};
use bsky_comment_extractor::db::{count_posts, open_existing_db, query_posts};
use bsky_comment_extractor::error::ExtractorError;
use bsky_comment_extractor::models::QueryEnvelope;
use tftio_cli_common::{
    AgentArgument, AgentCommand, AgentConfigFile, AgentDoc, AgentDocRequest,
    AgentEnvironmentVar, AgentExample, AgentFailureMode, AgentOperatorMistake, AgentOutputShape,
    AgentPath, AgentSection, AgentTool, AgentUsage, detect_agent_doc_request,
    render_agent_help_yaml, render_agent_skill,
};

struct BceDoctor;

impl DoctorChecks for BceDoctor {
    fn repo_info() -> tftio_cli_common::RepoInfo {
        tftio_cli_common::app::WORKSPACE_REPO
    }

    fn current_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

const TOOL_SPEC: ToolSpec = workspace_tool(
    "bce",
    "BlueSky Comment Extractor",
    env!("CARGO_PKG_VERSION"),
    LicenseType::MIT,
    false,
    true,
    false,
);

fn main() {
    if let Some(request) = detect_agent_doc_request(std::env::args_os()) {
        print_agent_doc(request);
        std::process::exit(0);
    }

    parse_and_exit(Cli::parse, run);
}

fn run(cli: Cli) -> Result<i32, FatalCliError> {
    let doctor = BceDoctor;

    // Check if this is a metadata command and handle it
    if let Some(ref cmd) = cli.command {
        if is_metadata_command(cmd) {
            let metadata_cmd = BceMetadataCommand(cmd);
            if let Some(exit_code) = maybe_run_standard_command::<Cli, BceDoctor, _>(
                &TOOL_SPEC,
                Some(&metadata_cmd),
                false,
                Some(&doctor),
            ) {
                return Ok(exit_code);
            }
        }
    }

    // Handle domain commands (fetch/query)
    match cli.command {
        Some(Command::Fetch(fetch)) => execute_fetch(fetch)
            .map(|()| 0)
            .map_err(|error| fatal_error("fetch", false, format!("{error:#}"))),
        Some(Command::Query(query)) => match execute_query(query) {
            Ok(()) => Ok(0),
            Err(_) => {
                // Query errors are already written to stderr as JSON
                // Just return exit code 1
                Ok(1)
            }
        },
        Some(_) => {
            // All metadata commands should have been handled above
            Ok(0)
        }
        None => {
            // No command provided - show help
            Ok(1)
        }
    }
}

fn print_agent_doc(request: AgentDocRequest) {
    let doc = build_agent_doc();
    let rendered = match request {
        AgentDocRequest::Help => render_agent_help_yaml(&doc),
        AgentDocRequest::Skill => render_agent_skill(&doc),
    };
    print!("{rendered}");
}

fn is_metadata_command(cmd: &Command) -> bool {
    matches!(
        cmd,
        Command::Version | Command::License | Command::Completions { .. } | Command::Doctor
    )
}

#[derive(Clone, Copy)]
struct BceMetadataCommand<'a>(&'a Command);

impl StandardCommandMap for BceMetadataCommand<'_> {
    fn to_standard_command(&self, _json: bool) -> StandardCommand {
        match self.0 {
            Command::Version => StandardCommand::Version { json: false },
            Command::License => StandardCommand::License,
            Command::Completions { shell } => StandardCommand::Completions { shell: *shell },
            Command::Doctor => StandardCommand::Doctor,
            Command::Fetch(_) | Command::Query(_) => {
                // Domain commands - not mapped to standard commands
                unreachable!("domain commands should not be mapped to standard commands")
            }
        }
    }
}

fn execute_fetch(fetch: FetchArgs) -> Result<()> {
    if std::env::var("BSKY_APP_PASSWORD").is_err() {
        bail!(
            "BSKY_APP_PASSWORD not set. Create an app password at \
             https://bsky.app/settings/app-passwords"
        );
    }

    let db_path = match fetch.db {
        Some(path) => path,
        None => default_db_path()?,
    };

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("failed to create database directory")?;
    }

    let since = fetch
        .since
        .as_deref()
        .map(|value| {
            dateparser::parse_with_timezone(value.trim(), &chrono::Utc)
                .map_err(|err| anyhow::anyhow!("failed to parse date '{value}': {err}"))
        })
        .transpose()?;

    let spinner = make_spinner(
        !fetch.quiet,
        &format!("Fetching posts for {}... 0 records", &fetch.handle),
    );

    let progress_cb = |count: u64| {
        if let Some(ref pb) = spinner {
            pb.set_message(format!(
                "Fetching posts for {}... {} records",
                &fetch.handle, count
            ));
        }
    };

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to start async runtime")?;

    let result = runtime.block_on(bsky_comment_extractor::run_extraction(
        &fetch.handle,
        &db_path,
        since,
        Some(&progress_cb),
    ));

    if let Some(ref pb) = spinner {
        pb.finish_and_clear();
    }

    let summary = result?;

    println!(
        "Extracted {} posts for {} to {} ({} new, {} existing)",
        summary.count,
        fetch.handle,
        db_path.display(),
        summary.new_count,
        summary.existing_count,
    );

    Ok(())
}

fn execute_query(query: QueryArgs) -> Result<()> {
    let db_path = query.db.unwrap_or(default_db_path()?);

    let conn = match open_existing_db(&db_path) {
        Ok(conn) => conn,
        Err(err) => return handle_query_error(err, &db_path),
    };

    let total = match count_posts(&conn) {
        Ok(total) => total,
        Err(err) => return handle_query_error(err, &db_path),
    };
    let posts = match query_posts(&conn, query.limit, query.offset) {
        Ok(posts) => posts,
        Err(err) => return handle_query_error(err, &db_path),
    };
    let has_more = query.offset.saturating_add(posts.len() as u64) < total;
    let envelope = QueryEnvelope {
        total,
        offset: query.offset,
        limit: query.limit,
        has_more,
    };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    serde_json::to_writer(&mut out, &envelope)?;
    writeln!(&mut out)?;
    for post in posts {
        serde_json::to_writer(&mut out, &post)?;
        writeln!(&mut out)?;
    }

    Ok(())
}

fn handle_query_error(err: ExtractorError, db_path: &Path) -> Result<()> {
    match &err {
        ExtractorError::Io(io_err) if io_err.kind() == std::io::ErrorKind::NotFound => {
            write_json_error(
                "db_not_found",
                &format!("database not found: {}", db_path.display()),
            );
        }
        _ => write_json_error("query_failed", &err.to_string()),
    }

    Err(err.into())
}

fn write_json_error(code: &str, message: &str) {
    let stderr = std::io::stderr();
    let mut err = stderr.lock();
    serde_json::to_writer(
        &mut err,
        &serde_json::json!({ "error": code, "message": message }),
    )
    .expect("json stderr write must succeed");
    writeln!(&mut err).expect("stderr newline must succeed");
}

fn build_agent_doc() -> AgentDoc {
    AgentDoc {
        schema_version: text("1.0"),
        tool: AgentTool {
            name: text("BlueSky Comment Extractor"),
            binary: text("bce"),
            summary: text(
                "Fetch a BlueSky account into SQLite, then query the local cache as JSONL.",
            ),
        },
        usage: AgentUsage {
            invocation: text("bce [--agent-help|--agent-skill] <COMMAND> [OPTIONS]"),
            notes: vec![
                text("`bce fetch` requires `BSKY_APP_PASSWORD`; `bce query` never does."),
                text("Agent-doc flags are top-level only. `bce query --agent-help` is not a valid invocation."),
                text("Query output is envelope-first JSONL: one metadata line followed by one JSON object per stored post."),
            ],
        },
        shared_sections: vec![AgentSection {
            title: text("Top-level agent-doc contract"),
            content: text(
                "Use `bce --agent-help` for canonical YAML and `bce --agent-skill` for a ready-to-save Claude skill document. Both commands print to stdout, exit 0, and stay hidden from normal `--help` output.",
            ),
        }],
        commands: vec![
            AgentCommand {
                name: text("fetch"),
                summary: text(
                    "Resolve a BlueSky handle or DID and store every available post in the local SQLite cache.",
                ),
                usage: text("bce fetch <HANDLE> [--db <PATH>] [--since <DATE>] [--quiet]"),
                arguments: vec![
                    flag_arg(
                        "--db",
                        "Override the default SQLite path instead of using the XDG data directory.",
                        false,
                    ),
                    flag_arg(
                        "--since",
                        "Only fetch posts created after a parseable date such as `2025-01-01` or `3 months ago`.",
                        false,
                    ),
                    flag_arg("--quiet", "Disable the progress spinner.", false),
                    positional_arg(
                        "handle",
                        "BlueSky handle like `alice.bsky.social` or a DID such as `did:plc:abc123`.",
                        true,
                    ),
                ],
                output_shapes: vec![AgentOutputShape {
                    name: text("fetch_summary"),
                    format: text("stdout text"),
                    description: text(
                        "Prints `Extracted N posts for HANDLE to PATH (NEW new, EXISTING existing)` after a successful sync.",
                    ),
                }],
            },
            AgentCommand {
                name: text("query"),
                summary: text(
                    "Read the existing SQLite cache without network access and stream JSONL records.",
                ),
                usage: text("bce query [--db <PATH>] [--limit <N>] [--offset <N>]"),
                arguments: vec![
                    flag_arg("--db", "Read from an existing SQLite file.", false),
                    flag_arg("--limit", "Maximum number of posts to return. Default: 50.", false),
                    flag_arg("--offset", "Number of rows to skip before returning results. Default: 0.", false),
                ],
                output_shapes: vec![
                    AgentOutputShape {
                        name: text("query_envelope"),
                        format: text("jsonl line 1"),
                        description: text(
                            "First line is a JSON object with `total`, `offset`, `limit`, and `has_more`.",
                        ),
                    },
                    AgentOutputShape {
                        name: text("query_post"),
                        format: text("jsonl lines 2+"),
                        description: text(
                            "Subsequent lines are post objects containing `uri`, `author_did`, `text`, and `created_at`.",
                        ),
                    },
                ],
            },
        ],
        arguments: vec![
            flag_arg("--agent-help", "Print this canonical YAML reference document.", false),
            flag_arg("--agent-skill", "Print the same content as a Claude skill document.", false),
        ],
        environment_variables: vec![AgentEnvironmentVar {
            name: text("BSKY_APP_PASSWORD"),
            description: text(
                "Required by `fetch`. Create an app password in BlueSky settings and export it before networked extraction.",
            ),
            required: true,
        }],
        config_files: vec![
            AgentConfigFile {
                path: text("none"),
                purpose: text("`bce` does not read a config file; behavior comes from flags, defaults, and `BSKY_APP_PASSWORD`."),
            },
            AgentConfigFile {
                path: text("stdout / stderr"),
                purpose: text("Successful fetch and query output goes to stdout; structured query errors go to stderr as JSON."),
            },
        ],
        default_paths: vec![AgentPath {
            name: text("default database"),
            path: text("~/.local/share/bce/bsky-posts.db"),
            purpose: text(
                "Used when `--db` is omitted for both `fetch` and `query`; parent directories are created for fetch.",
            ),
        }],
        output_shapes: vec![
            AgentOutputShape {
                name: text("fetch_summary"),
                format: text("stdout text"),
                description: text("Single summary line after extraction completes successfully."),
            },
            AgentOutputShape {
                name: text("query_envelope"),
                format: text("jsonl"),
                description: text("The first JSONL line contains `total`, `offset`, `limit`, and `has_more`."),
            },
            AgentOutputShape {
                name: text("query_error"),
                format: text("stderr json"),
                description: text(
                    "Failures emit `{\"error\":\"db_not_found\"|\"query_failed\",\"message\":\"...\"}`.",
                ),
            },
        ],
        examples: vec![
            AgentExample {
                name: text("fetch account"),
                command: text("bce fetch alice.bsky.social"),
                description: text("Fetch the entire post history for a handle into the default database."),
            },
            AgentExample {
                name: text("incremental fetch"),
                command: text("bce fetch alice.bsky.social --since '3 months ago' --db /tmp/bsky.db"),
                description: text("Resume into a custom database and skip older posts."),
            },
            AgentExample {
                name: text("page through cached posts"),
                command: text("bce query --limit 50 --offset 0"),
                description: text("Read the first page of cached JSONL output with the default database."),
            },
        ],
        failure_modes: vec![
            AgentFailureMode {
                name: text("App password missing"),
                symptom: text("`bce fetch` exits with `BSKY_APP_PASSWORD not set`."),
                resolution: text("Export `BSKY_APP_PASSWORD` before running `fetch`. `query` does not require it."),
            },
            AgentFailureMode {
                name: text("db_not_found"),
                symptom: text("`bce query` writes a structured stderr JSON error when the database path does not exist."),
                resolution: text("Run `bce fetch <HANDLE>` first or point `--db` at an existing SQLite file."),
            },
            AgentFailureMode {
                name: text("query_failed"),
                symptom: text("SQLite open or query operations fail for an existing path."),
                resolution: text("Inspect the stderr JSON `message`, confirm the file is a writable SQLite database, and retry."),
            },
        ],
        operator_mistakes: vec![
            AgentOperatorMistake {
                name: text("Running query without fetching first"),
                symptom: text("The default database path exists nowhere or contains no fetched posts."),
                correction: text("Run `bce fetch <HANDLE>` first, or pass `--db` to an existing populated database."),
            },
            AgentOperatorMistake {
                name: text("Using agent-doc flags after a subcommand"),
                symptom: text("Invocations such as `bce query --agent-help` fail clap validation."),
                correction: text("Move the request to the top level: `bce --agent-help` or `bce --agent-skill`."),
            },
            AgentOperatorMistake {
                name: text("Expecting query output to be a single JSON array"),
                symptom: text("The first line is envelope metadata and later lines are independent JSON objects."),
                correction: text("Treat stdout as JSONL: parse line 1 as pagination metadata, then parse each remaining line separately."),
            },
        ],
        constraints: vec![
            text("`query` is read-only and opens an existing SQLite file without creating one."),
            text("Dates passed to `--since` must be parseable by the dateparser crate."),
            text("Agent-doc flags intentionally bypass normal clap dispatch only for the exact top-level invocations."),
        ],
    }
}

fn positional_arg(name: &str, description: &str, required: bool) -> AgentArgument {
    AgentArgument {
        name: text(name),
        positional: true,
        description: text(description),
        required,
    }
}

fn flag_arg(name: &str, description: &str, required: bool) -> AgentArgument {
    AgentArgument {
        name: text(name),
        positional: false,
        description: text(description),
        required,
    }
}

fn text(value: &str) -> String {
    value.to_owned()
}

/// Resolve the default database path using XDG data directories.
///
/// Returns `~/.local/share/bce/bsky-posts.db` on Linux/macOS.
///
/// # Errors
///
/// Returns an error if the user data directory cannot be resolved.
fn default_db_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "tftio", "bce")
        .ok_or_else(|| anyhow::anyhow!("unable to resolve user data directory"))?;
    Ok(dirs.data_dir().join("bsky-posts.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_spinner_quiet() {
        // When quiet=true (inverted to false), make_spinner returns None
        assert!(make_spinner(false, "Fetching posts for alice.bsky.social... 0 records").is_none());
    }

    #[test]
    fn test_db_path_default() {
        let path = default_db_path().expect("should resolve XDG path");
        let path_str = path.to_string_lossy();
        assert!(
            path_str.ends_with("bsky-posts.db"),
            "unexpected default db path: {}",
            path.display()
        );
        assert!(
            path_str.contains("bce"),
            "path should contain 'bce' directory component: {}",
            path.display()
        );
    }

    #[test]
    fn metadata_commands_map_to_shared_standard_command() {
        assert_eq!(
            BceMetadataCommand(&Command::Doctor).to_standard_command(false),
            StandardCommand::Doctor
        );
        assert_eq!(
            BceMetadataCommand(&Command::Version).to_standard_command(false),
            StandardCommand::Version { json: false }
        );
    }

    #[test]
    fn run_returns_success_for_version_command() {
        let cli = Cli::parse_from(["bce", "version"]);
        assert_eq!(run(cli).expect("version command should succeed"), 0);
    }
}
