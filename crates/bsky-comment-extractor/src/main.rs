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

use bsky_comment_extractor::cli::{Cli, Command};

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
    parse_and_exit(Cli::parse, run);
}

fn run(cli: Cli) -> Result<i32, FatalCliError> {
    let doctor = BceDoctor;
    let command = cli.command.as_ref().map(BceMetadataCommand);
    if let Some(exit_code) =
        maybe_run_standard_command::<Cli, BceDoctor, _>(&TOOL_SPEC, command.as_ref(), false, Some(&doctor))
    {
        return Ok(exit_code);
    }

    execute(cli)
        .map(|()| 0)
        .map_err(|error| fatal_error("extract", false, format!("{error:#}")))
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
        }
    }
}

fn execute(cli: Cli) -> Result<()> {
    let handle = cli
        .handle
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("handle is required"))?;

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
    let spinner = make_spinner(!cli.quiet, &format!("Fetching posts for {handle}... 0 records"));

    // 6. Build progress callback that updates the spinner
    let progress_cb = |count: u64| {
        if let Some(ref pb) = spinner {
            pb.set_message(format!(
                "Fetching posts for {}... {} records",
                handle, count
            ));
        }
    };

    // 7. Build and run tokio runtime
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to start async runtime")?;

    let result = runtime.block_on(bsky_comment_extractor::run_extraction(
        handle,
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
        handle,
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
        assert!(make_spinner(false, "Fetching posts for alice.bsky.social... 0 records").is_none());
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
