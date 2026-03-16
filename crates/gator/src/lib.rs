//! Gator -- agent sandbox harness.
//!
//! Wraps coding agents (Claude, Codex, Gemini) with macOS `sandbox-exec`
//! integration and prompter-based system prompt composition.

pub mod agent;
pub mod cli;
pub mod config;
pub mod prompt;
pub mod sandbox;
pub mod session;
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

    // Branch: session mode vs non-session mode
    let (workdir, extras, denies, wt_info) = if let Some(session_id) = &cli.session {
        // Session mode: contract is sole authority
        let sandbox = session::fetch_session_sandbox(session_id)?;
        let (workdir, extras, denies) = session::into_sandbox_parts(sandbox);
        let wt_info = worktree::WorktreeInfo::default(); // no auto-detection
        (workdir, extras, denies, wt_info)
    } else {
        // Non-session mode: implicit resolution
        let workdir = config::resolve_workdir(cli.workdir.as_deref())?;

        let mut safehouse_extras = config::load_safehouse_config(&workdir);

        // Load named policies
        let denies = if cli.policies.is_empty() {
            Vec::new()
        } else {
            let (policy_extras, policy_denies) =
                config::load_policies(&cli.policies, &workdir)?;
            safehouse_extras.rw.extend(policy_extras.rw);
            safehouse_extras.ro.extend(policy_extras.ro);
            policy_denies
        };

        let extras =
            config::merge_extra_dirs(safehouse_extras, &cli.add_dirs, &cli.add_dirs_ro);
        let wt_info = worktree::detect_worktrees(&workdir);
        (workdir, extras, denies, wt_info)
    };

    // 5. Assemble sandbox policy
    let policy = sandbox::assemble_policy(&workdir, &wt_info, &extras, &denies)
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
