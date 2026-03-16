//! Gator -- agent sandbox harness.
//!
//! Wraps coding agents (Claude, Codex, Gemini) with macOS `sandbox-exec`
//! integration and prompter-based system prompt composition.

pub mod agent;
pub mod cli;
pub mod config;
pub mod prompt;
pub mod sandbox;
pub mod worktree;

use cli::Cli;

/// Run the gator harness.
///
/// # Errors
/// Returns an error string if any step fails.
pub fn run(cli: &Cli) -> Result<(), String> {
    // 1. Prepend clankers to PATH
    if let Some(home) = dirs::home_dir() {
        let clankers = home.join(".local/clankers/bin");
        if clankers.is_dir() {
            let path = std::env::var("PATH").unwrap_or_default();
            // SAFETY: gator is single-threaded and sets PATH before spawning anything.
            unsafe {
                std::env::set_var("PATH", format!("{}:{path}", clankers.display()));
            }
        }
    }

    // 2. Resolve workdir
    let workdir = config::resolve_workdir(cli.workdir.as_deref())?;

    // 3. Load .safehouse + merge CLI extra dirs
    let safehouse_extras = config::load_safehouse_config(&workdir);
    let extras = config::merge_extra_dirs(safehouse_extras, &cli.add_dirs, &cli.add_dirs_ro);

    // 4. Detect worktrees
    let wt_info = worktree::detect_worktrees(&workdir);

    // 5. Assemble sandbox policy
    let policy = sandbox::assemble_policy(&workdir, &wt_info, &extras, &[])
        .map_err(|e| format!("policy assembly failed: {e}"))?;

    // 6. Dry-run: print policy and exit
    if cli.dry_run {
        eprint!("{policy}");
        return Ok(());
    }

    // 7. Compose prompt (if not --no-prompt)
    let prompt = if cli.no_prompt {
        None
    } else {
        Some(prompt::compose_prompt(&cli.profiles)?)
    };

    // 8. Write policy to temp file
    let policy_file = tempfile::Builder::new()
        .prefix("gator-policy-")
        .suffix(".sb")
        .tempfile()
        .map_err(|e| format!("cannot create policy tempfile: {e}"))?;

    std::fs::write(policy_file.path(), &policy)
        .map_err(|e| format!("cannot write policy: {e}"))?;

    // 9. Build and exec command
    let (cmd, _tempfiles) = agent::build_command(
        &cli.agent,
        policy_file.path(),
        prompt.as_deref(),
        &cli.agent_args,
    )
    .map_err(|e| format!("cannot build command: {e}"))?;

    // exec replaces the process -- only returns on error
    let err = agent::exec_command(cmd);
    Err(format!("exec failed: {err}"))
}
