//! CLI entry point for the `bce` (`BlueSky` Comment Extractor) binary.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::CommandFactory;
use directories::ProjectDirs;
use tftio_cli_common::{
    AgentCapability, AgentDispatch, AgentSurfaceSpec, CommandSelector, FlagSelector, LicenseType,
    parse_with_agent_surface, progress::make_spinner, workspace_tool,
};

use bsky_comment_extractor::cli::{Cli, Command as CliCommand, FetchArgs, QueryArgs};
use bsky_comment_extractor::db::{count_posts, open_existing_db, query_posts};
use bsky_comment_extractor::error::ExtractorError;
use bsky_comment_extractor::models::QueryEnvelope;

const QUERY_COMMAND: CommandSelector = CommandSelector::new(&["query"]);
const QUERY_DB_FLAG: FlagSelector = FlagSelector::new(&["query"], "db");
const QUERY_LIMIT_FLAG: FlagSelector = FlagSelector::new(&["query"], "limit");
const QUERY_OFFSET_FLAG: FlagSelector = FlagSelector::new(&["query"], "offset");

const QUERY_POSTS_CAPABILITY: AgentCapability = AgentCapability::new(
    "query-posts",
    "Read paginated post records from the local SQLite store",
    &[QUERY_COMMAND],
    &[QUERY_DB_FLAG, QUERY_LIMIT_FLAG, QUERY_OFFSET_FLAG],
)
.with_examples(&[
    "bce query --limit 25",
    "bce query --db /tmp/bsky.db --offset 50",
])
.with_output("stdout emits one JSON envelope line followed by JSON post lines")
.with_constraints("local SQLite only; no network; missing DB returns structured stderr JSON");

const AGENT_SURFACE: AgentSurfaceSpec = AgentSurfaceSpec::new(&[QUERY_POSTS_CAPABILITY]);

const TOOL_SPEC: tftio_cli_common::ToolSpec = workspace_tool(
    "bce",
    "BCE",
    env!("CARGO_PKG_VERSION"),
    LicenseType::MIT,
    false,
    false,
    false,
)
.with_agent_surface(&AGENT_SURFACE);

fn main() {
    match parse_with_agent_surface::<Cli>(&TOOL_SPEC) {
        Ok(AgentDispatch::Cli(cli)) => std::process::exit(run(cli)),
        Ok(AgentDispatch::Printed(code)) => std::process::exit(code),
        Err(error) => error.exit(),
    }
}

fn run(cli: Cli) -> i32 {
    let Some(command) = cli.command else {
        return print_top_level_help();
    };

    match command {
        CliCommand::Fetch(fetch) => match execute_fetch(fetch) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("Error: {err:#}");
                1
            }
        },
        CliCommand::Query(query) => i32::from(execute_query(query).is_err()),
    }
}

fn print_top_level_help() -> i32 {
    let mut command = Cli::command();
    command
        .print_help()
        .expect("clap help write to stdout must succeed");
    println!();
    0
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

    let spinner = make_spinner(fetch.quiet, &fetch.handle);

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
            write_json_error("db_not_found", &format!("database not found: {}", db_path.display()));
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
    use clap::Parser;

    #[test]
    fn test_make_spinner_quiet() {
        assert!(make_spinner(true, "alice.bsky.social").is_none());
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
    fn run_returns_success_for_query_command() {
        let cli = Cli::parse_from(["bce", "query"]);
        assert_eq!(run(cli), 1);
    }

    #[test]
    fn tool_spec_declares_only_query_posts_agent_capability() {
        let capability = TOOL_SPEC
            .agent_surface
            .expect("agent surface should be configured")
            .capabilities
            .first()
            .expect("query-posts capability should exist");

        assert_eq!(capability.name, "query-posts");
        assert_eq!(capability.commands, &[QUERY_COMMAND]);
        assert_eq!(
            capability.flags,
            &[QUERY_DB_FLAG, QUERY_LIMIT_FLAG, QUERY_OFFSET_FLAG]
        );
    }
}
