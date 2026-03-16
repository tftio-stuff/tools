//! Agent-specific command building and execution.

use crate::cli::Agent;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

/// Write prompt text to a temp file, returning the handle.
///
/// The caller must keep the `NamedTempFile` alive until exec to prevent
/// cleanup. Since we exec, the file handle is inherited by the child.
///
/// # Errors
/// Returns an error if the temp file cannot be written.
pub fn write_prompt_tempfile(prompt: &str) -> Result<NamedTempFile, io::Error> {
    use std::io::Write;
    let mut f = NamedTempFile::new()?;
    f.write_all(prompt.as_bytes())?;
    f.flush()?;
    Ok(f)
}

/// Build the full command for `sandbox-exec`, including prompt injection.
///
/// Returns the `Command` ready for exec and any temp files that must
/// be kept alive until exec.
///
/// # Errors
/// Returns an error if prompt tempfile creation fails.
pub fn build_command(
    agent: &Agent,
    policy_path: &Path,
    prompt: Option<&str>,
    agent_args: &[String],
) -> Result<(Command, Vec<NamedTempFile>), io::Error> {
    let mut cmd = Command::new("sandbox-exec");
    cmd.arg("-f").arg(policy_path).arg("--");

    let mut tempfiles: Vec<NamedTempFile> = Vec::new();

    // Agent binary name
    cmd.arg(agent.to_string());

    // Inject prompt per agent's mechanism
    if let Some(prompt_text) = prompt {
        match agent {
            Agent::Claude => {
                cmd.arg("--append-system-prompt").arg(prompt_text);
            }
            Agent::Codex => {
                let f = write_prompt_tempfile(prompt_text)?;
                let path = f.path().to_path_buf();
                cmd.arg("-c")
                    .arg(format!("experimental_instructions_file={}", path.display()));
                tempfiles.push(f);
            }
            Agent::Gemini => {
                let f = write_prompt_tempfile(prompt_text)?;
                let path = f.path().to_path_buf();
                cmd.env("GEMINI_SYSTEM_MD", path);
                tempfiles.push(f);
            }
        }
    }

    // Forward remaining agent args
    cmd.args(agent_args);

    Ok((cmd, tempfiles))
}

/// Execute the sandboxed agent command (replaces current process).
///
/// # Errors
/// Returns the OS error if exec fails (this function does not return on success).
#[must_use]
pub fn exec_command(mut cmd: Command) -> io::Error {
    cmd.exec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_command_claude_no_prompt() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let (cmd, temps) = build_command(
            &Agent::Claude,
            tmp.path(),
            None,
            &["--help".to_owned()],
        )
        .unwrap();
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert!(args.contains(&"claude".to_owned()));
        assert!(args.contains(&"--help".to_owned()));
        assert!(temps.is_empty());
    }

    #[test]
    fn build_command_claude_with_prompt() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let (cmd, temps) = build_command(
            &Agent::Claude,
            tmp.path(),
            Some("system prompt"),
            &[],
        )
        .unwrap();
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert!(args.contains(&"--append-system-prompt".to_owned()));
        assert!(args.contains(&"system prompt".to_owned()));
        assert!(temps.is_empty());
    }

    #[test]
    fn build_command_codex_with_prompt() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let (cmd, temps) = build_command(
            &Agent::Codex,
            tmp.path(),
            Some("codex prompt"),
            &[],
        )
        .unwrap();
        assert_eq!(temps.len(), 1);
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert!(args
            .iter()
            .any(|a| a.starts_with("experimental_instructions_file=")));
    }

    #[test]
    fn build_command_gemini_with_prompt() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let (cmd, temps) = build_command(
            &Agent::Gemini,
            tmp.path(),
            Some("gemini prompt"),
            &[],
        )
        .unwrap();
        assert_eq!(temps.len(), 1);
        let envs: Vec<_> = cmd.get_envs().collect();
        assert!(envs.iter().any(|(k, _)| *k == "GEMINI_SYSTEM_MD"));
    }
}
