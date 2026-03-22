//! CLI entry point for the `bce` (`BlueSky` Comment Extractor) binary.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use directories::ProjectDirs;
use tftio_cli_common::{
    DoctorChecks, FatalCliError, LicenseType, StandardCommand, StandardCommandMap, ToolSpec,
    command::maybe_run_standard_command, error::fatal_error, parse_and_exit,
    progress::make_spinner, workspace_tool,
};

use bsky_comment_extractor::cli::Cli;

struct BceDoctor;

fn run(cli: Cli) -> i32 {
    match execute(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {e:#}");
            1
        }
    }
}

fn execute(cli: Cli) -> Result<()> {
    // 1. Validate BSKY_APP_PASSWORD is set
    if std::env::var("BSKY_APP_PASSWORD").is_err() {
        bail!(
            "BSKY_APP_PASSWORD not set. Create an app password at \
             https://bsky.app/settings/app-passwords"
        );
    }

    // 2. Resolve database path: --db flag or XDG default
    let db_path = match cli.db {
        Some(p) => p,
        None => default_db_path()?,
    };

    // 3. Create parent directories if they don't exist
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).context("failed to create database directory")?;
    }

    // 4. Parse --since date if provided
    let since = cli
        .since
        .as_deref()
        .map(|s| {
            dateparser::parse_with_timezone(s.trim(), &chrono::Utc)
                .map_err(|e| anyhow::anyhow!("failed to parse date '{s}': {e}"))
        })
        .transpose()?;

    // 5. Create spinner (only if TTY and not --quiet)
    let spinner = make_spinner(cli.quiet, &cli.handle);

    // 6. Build progress callback that updates the spinner
    let progress_cb = |count: u64| {
        if let Some(ref pb) = spinner {
            pb.set_message(format!(
                "Fetching posts for {}... {} records",
                &cli.handle, count
            ));
        }
    };

    // 7. Build and run tokio runtime
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to start async runtime")?;

    let result = runtime.block_on(bsky_comment_extractor::run_extraction(
        &cli.handle,
        &db_path,
        since,
        Some(&progress_cb),
    ));

    // 8. Clear spinner before printing summary or error
    if let Some(ref pb) = spinner {
        pb.finish_and_clear();
    }

    let summary = result?;

    // 9. Print completion summary to stdout
    println!(
        "Extracted {} posts for {} to {} ({} new, {} existing)",
        summary.count,
        cli.handle,
        db_path.display(),
        summary.new_count,
        summary.existing_count,
    );

    Ok(())
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
        // When quiet=true, make_spinner must return None regardless of TTY state.
        assert!(make_spinner(true, "alice.bsky.social").is_none());
    }

    #[test]
    fn test_db_path_default() {
        // default_db_path should return a path ending in bsky-posts.db within a bce-related directory.
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
