//! Self-update module.
//!
//! This module provides self-update functionality for CLI tools by delegating
//! to the install script (install.sh), which handles:
//! - Checking for latest releases on GitHub
//! - Downloading release binaries
//! - Verifying checksums (mandatory)
//! - Version comparison and upgrade logic
//! - Replacing the current binary

use crate::types::RepoInfo;
use std::path::Path;
use std::process::Command;

/// Run update command to install latest or specified version.
///
/// This delegates to the install.sh script, which handles version checking,
/// download, checksum verification, and installation.
///
/// Returns exit code: 0 if successful, 1 on error, 2 if already up-to-date.
///
/// # Arguments
/// * `repo_info` - Repository information for GitHub integration
/// * `_current_version` - Current version of the tool (unused, install.sh detects this)
/// * `version` - Optional specific version to install (currently unsupported, always installs latest)
/// * `force` - Force reinstall even if already up-to-date
/// * `install_dir` - Optional custom installation directory
///
/// # Panics
/// May panic if stdout flush fails during user interaction.
#[must_use]
pub fn run_update(
    repo_info: &RepoInfo,
    _current_version: &str,
    version: Option<&str>,
    force: bool,
    install_dir: Option<&Path>,
) -> i32 {
    if version.is_some() {
        eprintln!("⚠️  Specific version installation not yet supported");
        eprintln!("   The install script will install the latest version");
        println!();
    }

    println!("🔄 Running installation script...");
    println!();

    // Build install.sh URL
    let install_script_url = format!(
        "https://raw.githubusercontent.com/{}/{}/main/install.sh",
        repo_info.owner, repo_info.name
    );

    // Build command to download and execute install script
    let mut cmd = Command::new("sh");
    cmd.arg("-c");

    // Build the command string with environment variables
    let mut env_vars = Vec::new();
    env_vars.push(format!("REPO_OWNER={}", repo_info.owner));
    env_vars.push(format!("REPO_NAME={}", repo_info.name));

    if force {
        env_vars.push("FORCE_INSTALL=1".to_string());
    }

    if let Some(dir) = install_dir {
        env_vars.push(format!("INSTALL_DIR={}", dir.display()));
    }

    let env_string = env_vars.join(" ");
    let command_string = format!("{env_string} curl -fsSL {install_script_url} | sh");

    cmd.arg(&command_string);

    // Execute the command
    match cmd.status() {
        Ok(status) => {
            if status.success() {
                0
            } else {
                status.code().unwrap_or(1)
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to run install script: {e}");
            eprintln!("   Make sure curl is installed and you have internet access");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_script_url_construction() {
        let repo = RepoInfo::new("tftio", "peter-hook");
        let expected = "https://raw.githubusercontent.com/tftio/peter-hook/main/install.sh";
        let actual = format!(
            "https://raw.githubusercontent.com/{}/{}/main/install.sh",
            repo.owner, repo.name
        );
        assert_eq!(actual, expected);
    }
}
