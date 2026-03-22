//! CLI entry point for the `bce` (`BlueSky` Comment Extractor) binary.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser};
use directories::ProjectDirs;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};

use bsky_comment_extractor::cli::{Cli, Command as CliCommand, FetchArgs, QueryArgs};
use bsky_comment_extractor::db::{count_posts, open_existing_db, query_posts};
use bsky_comment_extractor::error::ExtractorError;
use bsky_comment_extractor::models::QueryEnvelope;

fn main() {
    let cli = Cli::parse();
    let code = run(cli);
    std::process::exit(code);
}

fn run(cli: Cli) -> i32 {
    if cli.agent_help {
        print_agent_help();
        return 0;
    }

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

fn print_agent_help() {
    println!(
        "# bce agent help\nstatus: pending_phase_06\ncommands:\n  - fetch\n  - query\nmessage: Full agent reference lands in Phase 6."
    );
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

/// Create an `indicatif` spinner for progress display.
///
/// Returns `None` if `quiet` is true or stdout is not a terminal.
fn make_spinner(quiet: bool, handle: &str) -> Option<ProgressBar> {
    if quiet || !tftio_cli_common::output::is_tty() {
        return None;
    }
    let pb = ProgressBar::with_draw_target(None, ProgressDrawTarget::stderr());
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("\u{2801}\u{2802}\u{2804}\u{2840}\u{2880}\u{2820}\u{2810}\u{2808} ")
            .template("{spinner:.green} {msg}")
            .expect("valid spinner template"),
    );
    pb.set_message(format!("Fetching posts for {handle}... 0 records"));
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    Some(pb)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
