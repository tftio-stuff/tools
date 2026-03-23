//! unvenv - Python venv detector CLI
//!
//! Scans the current Git working tree for non-ignored `pyvenv.cfg` files
//! and exits with error status if any are found, preventing accidental
//! commits of Python virtual environments.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use git2::Repository;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process,
};
use tftio_cli_common::{
    AgentCapability, AgentDispatch, AgentSurfaceSpec, CommandSelector, DoctorCheck,
    DoctorChecks, LicenseType, RepoInfo, StandardCommand, ToolSpec,
    command::run_standard_command, parse_with_agent_surface, run_with_display_error_handler,
    workspace_tool,
};
use walkdir::WalkDir;

/// Application version from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Information extracted from a pyvenv.cfg file
#[derive(Debug)]
struct VenvInfo {
    path: PathBuf,
    home: Option<String>,
    version: Option<String>,
    include_system_site_packages: Option<String>,
}

/// Python virtual environment detector CLI
#[derive(Parser)]
#[command(name = "unvenv")]
#[command(about = "Python virtual environment detector CLI")]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show version information
    Version,
    /// Show license information
    License,
    /// Scan for unignored Python virtual environments (default)
    Scan,
    /// Generate shell completion scripts
    Completions {
        /// Shell type (bash, zsh, fish, etc.)
        shell: clap_complete::Shell,
    },
    /// Check health and configuration
    Doctor,
    /// Update to the latest version
    Update {
        /// Specific version to install (defaults to latest)
        #[arg(long)]
        version: Option<String>,
        /// Force update even if already at target version
        #[arg(long)]
        force: bool,
        /// Custom installation directory
        #[arg(long)]
        install_dir: Option<PathBuf>,
    },
}

struct UnvenvTool;

impl DoctorChecks for UnvenvTool {
    fn repo_info() -> RepoInfo {
        RepoInfo::new("tftio", "unvenv")
    }

    fn current_version() -> &'static str {
        VERSION
    }

    fn tool_checks(&self) -> Vec<DoctorCheck> {
        let mut checks = Vec::new();

        // Check if in git repository
        if let Ok(repo) = Repository::discover(".") {
            if repo.is_bare() {
                checks.push(DoctorCheck::fail(
                    "Git repository check",
                    "In bare Git repository - unvenv works best with regular repositories",
                ));
            } else if let Some(workdir) = repo.workdir() {
                checks.push(DoctorCheck::pass(format!(
                    "Git repository: {}",
                    workdir.display()
                )));
            }
        }

        checks
    }
}

const TOOL_SPEC: ToolSpec = workspace_tool(
    "unvenv",
    "unvenv",
    VERSION,
    LicenseType::MIT,
    false,
    true,
    true,
)
.with_agent_surface(&AGENT_SURFACE);

const SCAN_COMMAND: CommandSelector = CommandSelector::new(&["scan"]);
const SCAN_VENVS_CAPABILITY: AgentCapability = AgentCapability::minimal(
    "scan-venvs",
    &[SCAN_COMMAND],
    &[],
)
.with_output("plain text on stdout with a non-zero exit when unignored pyvenv.cfg files are found")
.with_constraints("reads the current working tree only");
const AGENT_SURFACE: AgentSurfaceSpec = AgentSurfaceSpec::new(&[SCAN_VENVS_CAPABILITY]);

fn main() {
    let is_tty = tftio_cli_common::output::is_tty();
    match parse_with_agent_surface::<Cli>(&TOOL_SPEC) {
        Ok(AgentDispatch::Cli(cli)) => {
            let exit_code = run_with_display_error_handler("unvenv", false, || run(cli, is_tty));
            process::exit(exit_code);
        }
        Ok(AgentDispatch::Printed(code)) => process::exit(code),
        Err(error) => error.exit(),
    }
}

fn run(cli: Cli, is_tty: bool) -> Result<i32> {
    let tool = UnvenvTool;

    match cli.command {
        Some(Commands::Version) => Ok(run_standard_command::<Cli, UnvenvTool>(
            &TOOL_SPEC,
            &StandardCommand::Version { json: false },
            Some(&tool),
        )),
        Some(Commands::License) => Ok(run_standard_command::<Cli, UnvenvTool>(
            &TOOL_SPEC,
            &StandardCommand::License,
            Some(&tool),
        )),
        Some(Commands::Scan) | None => {
            // Default behavior: scan for venv files
            scan_for_venvs(is_tty)
        }
        Some(Commands::Completions { shell }) => Ok(run_standard_command::<Cli, UnvenvTool>(
            &TOOL_SPEC,
            &StandardCommand::Completions { shell },
            Some(&tool),
        )),
        Some(Commands::Doctor) => Ok(run_standard_command::<Cli, UnvenvTool>(
            &TOOL_SPEC,
            &StandardCommand::Doctor,
            Some(&tool),
        )),
        Some(Commands::Update {
            version,
            force,
            install_dir,
        }) => Ok(run_standard_command::<Cli, UnvenvTool>(
            &TOOL_SPEC,
            &StandardCommand::Update {
                version,
                force,
                install_dir,
            },
            Some(&tool),
        )),
    }
}

fn scan_for_venvs(is_tty: bool) -> Result<i32> {
    let workdir = std::env::current_dir().context("Failed to get current directory")?;
    scan_for_venvs_in_dir(&workdir, is_tty)
}

/// Scan a specific directory for unignored Python virtual environments
fn scan_for_venvs_in_dir(workdir: &Path, is_tty: bool) -> Result<i32> {
    // Try to discover Git repository for ignore checking, but don't require it
    let repo = Repository::discover(workdir).ok();

    // Find all pyvenv.cfg files in the directory tree
    let mut unignored_venvs = Vec::new();

    for entry in WalkDir::new(workdir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip .git directory
            e.file_name().to_str() != Some(".git")
        })
    {
        let entry = entry.context("Failed to read directory entry")?;

        // Check if this is a pyvenv.cfg file
        if entry.file_name() == "pyvenv.cfg" && entry.file_type().is_file() {
            let full_path = entry.path();

            // Get path relative to current workdir
            let rel_path = full_path
                .strip_prefix(workdir)
                .context("Failed to create relative path")?;

            // Check if file is ignored by Git (if we have a repo)
            let is_ignored = if let Some(ref repo) = repo {
                // Skip bare repositories
                if repo.is_bare() {
                    false
                } else {
                    repo.status_should_ignore(rel_path)
                        .context("Failed to check Git ignore status")?
                }
            } else {
                // No Git repo, so treat as not ignored
                false
            };

            if !is_ignored {
                // Parse the pyvenv.cfg file
                let venv_info = parse_pyvenv_cfg(full_path, rel_path)?;
                unignored_venvs.push(venv_info);
            }
        }
    }

    // Handle results
    if unignored_venvs.is_empty() {
        // No unignored venv files found
        Ok(0)
    } else {
        // Found unignored venv files - print helpful output and exit with error
        print_violation_report(&unignored_venvs, is_tty);
        Ok(2)
    }
}

/// Parse a pyvenv.cfg file to extract useful metadata
fn parse_pyvenv_cfg(full_path: &Path, rel_path: &Path) -> Result<VenvInfo> {
    let content = fs::read_to_string(full_path)
        .with_context(|| format!("Failed to read {}", rel_path.display()))?;

    let mut fields = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            fields.insert(key.to_string(), value.to_string());
        }
    }

    Ok(VenvInfo {
        path: rel_path.to_path_buf(),
        home: fields.get("home").cloned(),
        version: fields.get("version").cloned(),
        include_system_site_packages: fields.get("include-system-site-packages").cloned(),
    })
}

/// Print a helpful report about policy violations
#[allow(clippy::too_many_lines)]
fn print_violation_report(venvs: &[VenvInfo], is_tty: bool) {
    if is_tty {
        println!(
            "{} Found Python virtual environment files that are not ignored by Git!",
            "WARNING:".yellow().bold()
        );
        println!();
        println!("Python virtual environments should not be committed to version control.");
        println!("They contain system-specific paths and can be large and unnecessary.");
        println!();

        println!(
            "{}",
            "Found the following unignored pyvenv.cfg files:".bold()
        );
        println!();

        for venv in venvs {
            let normalized_path = venv.path.to_string_lossy().replace('\\', "/");
            println!("  📁 {}", normalized_path.cyan());

            if let Some(home) = &venv.home {
                println!("     Python home: {home}");
            }
            if let Some(version) = &venv.version {
                println!("     Python version: {version}");
            }
            if let Some(include_sys) = &venv.include_system_site_packages {
                println!("     Include system packages: {include_sys}");
            }
            println!();
        }

        // Suggest gitignore entries
        let mut suggested_ignores = std::collections::HashSet::new();
        for venv in venvs {
            if let Some(parent) = venv.path.parent() {
                if let Some(dir_name) = parent.file_name() {
                    if let Some(dir_str) = dir_name.to_str() {
                        suggested_ignores.insert(format!("{dir_str}/"));
                    }
                }
            }
        }

        if !suggested_ignores.is_empty() {
            println!("{}", "Suggested .gitignore entries:".bold());
            println!();
            for ignore_entry in suggested_ignores {
                println!("  {}", ignore_entry.green());
            }
            println!();
        }

        println!("To fix this issue:");
        println!("1. Add the virtual environment directories to your .gitignore file");
        println!("2. If already committed, remove them from the index:");
        for venv in venvs {
            if let Some(parent) = venv.path.parent() {
                println!(
                    "   {}",
                    format!("git rm -r --cached {}", parent.display()).yellow()
                );
            }
        }
        println!("2. If already committed, remove them from the index:");
        for venv in venvs {
            if let Some(parent) = venv.path.parent() {
                println!(
                    "   {}",
                    format!("git rm -r --cached {}", parent.display()).yellow()
                );
            }
        }
    } else {
        // Non-TTY output: plain text without colors or decorations
        println!("WARNING: Found Python virtual environment files that are not ignored by Git!");
        println!();
        println!("Python virtual environments should not be committed to version control.");
        println!();

        println!("Found the following unignored pyvenv.cfg files:");
        for venv in venvs {
            let normalized_path = venv.path.to_string_lossy().replace('\\', "/");
            println!("  {normalized_path}");
            if let Some(home) = &venv.home {
                println!("    Python home: {home}");
            }
            if let Some(version) = &venv.version {
                println!("    Python version: {version}");
            }
            if let Some(include_sys) = &venv.include_system_site_packages {
                println!("    Include system packages: {include_sys}");
            }
        }
        println!();

        // Suggest gitignore entries
        let mut suggested_ignores = std::collections::HashSet::new();
        for venv in venvs {
            if let Some(parent) = venv.path.parent() {
                if let Some(dir_name) = parent.file_name() {
                    if let Some(dir_str) = dir_name.to_str() {
                        suggested_ignores.insert(format!("{dir_str}/"));
                    }
                }
            }
        }

        if !suggested_ignores.is_empty() {
            println!("Suggested .gitignore entries:");
            for ignore_entry in suggested_ignores {
                println!("  {ignore_entry}");
            }
            println!();
        }

        println!("To fix this issue:");
        println!("1. Add the virtual environment directories to your .gitignore file");
        println!("2. If already committed, remove them from the index:");
        for venv in venvs {
            if let Some(parent) = venv.path.parent() {
                println!("   git rm -r --cached {}", parent.display());
            }
        }
    }
    println!("3. Commit the .gitignore changes");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_version_constant() {
        #[allow(clippy::len_zero)]
        {
            assert!(VERSION.len() > 0); // Check VERSION has content
        }
        assert!(VERSION.chars().next().unwrap().is_ascii_digit());
    }

    #[test]
    fn test_parse_pyvenv_cfg() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyvenv_path = temp_dir.path().join("pyvenv.cfg");

        let content = r"home = /usr/bin
include-system-site-packages = false
version = 3.9.7
";
        fs::write(&pyvenv_path, content)?;

        let info = parse_pyvenv_cfg(&pyvenv_path, Path::new("test/pyvenv.cfg"))?;

        assert_eq!(info.home, Some("/usr/bin".to_string()));
        assert_eq!(info.version, Some("3.9.7".to_string()));
        assert_eq!(info.include_system_site_packages, Some("false".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_empty_pyvenv_cfg() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyvenv_path = temp_dir.path().join("pyvenv.cfg");

        fs::write(&pyvenv_path, "")?;

        let info = parse_pyvenv_cfg(&pyvenv_path, Path::new("test/pyvenv.cfg"))?;

        assert_eq!(info.home, None);
        assert_eq!(info.version, None);
        assert_eq!(info.include_system_site_packages, None);

        Ok(())
    }

    #[test]
    fn test_parse_pyvenv_cfg_with_comments() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyvenv_path = temp_dir.path().join("pyvenv.cfg");

        let content = r"# This is a comment
home = /opt/python
# Another comment
version = 3.10.1
# Trailing comment
";
        fs::write(&pyvenv_path, content)?;

        let info = parse_pyvenv_cfg(&pyvenv_path, Path::new("test/pyvenv.cfg"))?;

        assert_eq!(info.home, Some("/opt/python".to_string()));
        assert_eq!(info.version, Some("3.10.1".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_pyvenv_cfg_with_whitespace() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyvenv_path = temp_dir.path().join("pyvenv.cfg");

        let content = r"  home   =   /usr/local/bin
  version  =  3.11.0
";
        fs::write(&pyvenv_path, content)?;

        let info = parse_pyvenv_cfg(&pyvenv_path, Path::new("test/pyvenv.cfg"))?;

        assert_eq!(info.home, Some("/usr/local/bin".to_string()));
        assert_eq!(info.version, Some("3.11.0".to_string()));

        Ok(())
    }

    #[test]
    fn test_print_violation_report_tty() {
        let venvs = vec![VenvInfo {
            path: PathBuf::from("venv/pyvenv.cfg"),
            home: Some("/usr/bin".to_string()),
            version: Some("3.9.0".to_string()),
            include_system_site_packages: Some("false".to_string()),
        }];

        // Should not panic
        print_violation_report(&venvs, true);
    }

    #[test]
    fn test_print_violation_report_non_tty() {
        let venvs = vec![VenvInfo {
            path: PathBuf::from("venv/pyvenv.cfg"),
            home: Some("/usr/bin".to_string()),
            version: Some("3.9.0".to_string()),
            include_system_site_packages: None,
        }];

        // Should not panic
        print_violation_report(&venvs, false);
    }

    #[test]
    fn test_print_violation_report_multiple_venvs() {
        let venvs = vec![
            VenvInfo {
                path: PathBuf::from("venv1/pyvenv.cfg"),
                home: Some("/usr/bin".to_string()),
                version: Some("3.9.0".to_string()),
                include_system_site_packages: Some("true".to_string()),
            },
            VenvInfo {
                path: PathBuf::from("venv2/pyvenv.cfg"),
                home: None,
                version: None,
                include_system_site_packages: None,
            },
        ];

        // Should not panic with multiple venvs
        print_violation_report(&venvs, true);
        print_violation_report(&venvs, false);
    }

    #[test]
    fn test_venv_info_creation() {
        let venv = VenvInfo {
            path: PathBuf::from("test/pyvenv.cfg"),
            home: Some("/usr/bin".to_string()),
            version: Some("3.9.0".to_string()),
            include_system_site_packages: Some("false".to_string()),
        };

        assert_eq!(venv.path, PathBuf::from("test/pyvenv.cfg"));
        assert_eq!(venv.home, Some("/usr/bin".to_string()));
        assert_eq!(venv.version, Some("3.9.0".to_string()));
        assert_eq!(venv.include_system_site_packages, Some("false".to_string()));
    }

    #[test]
    fn test_parse_pyvenv_cfg_malformed_lines() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyvenv_path = temp_dir.path().join("pyvenv.cfg");

        // File with lines that don't have = separator
        let content = r"home = /usr/bin
this line has no equals sign
version = 3.9.0
another malformed line
";
        fs::write(&pyvenv_path, content)?;

        let info = parse_pyvenv_cfg(&pyvenv_path, Path::new("test/pyvenv.cfg"))?;

        // Should still parse valid lines
        assert_eq!(info.home, Some("/usr/bin".to_string()));
        assert_eq!(info.version, Some("3.9.0".to_string()));

        Ok(())
    }

    #[test]
    fn test_scan_for_venvs_no_violations() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Initialize git repo — clear git env vars so `GIT_DIR` set by hooks doesn't hijack init
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_OBJECT_DIRECTORY")
            .env_remove("GIT_COMMON_DIR")
            .output()?;

        // Create .gitignore
        fs::write(temp_dir.path().join(".gitignore"), "venv/\n")?;

        // Create ignored venv
        let venv_dir = temp_dir.path().join("venv");
        fs::create_dir(&venv_dir)?;
        fs::write(venv_dir.join("pyvenv.cfg"), "home = /usr/bin\n")?;

        // Scan should return 0 (no violations)
        let result = scan_for_venvs_in_dir(temp_dir.path(), false)?;
        assert_eq!(result, 0, "Should return 0 when all venvs are ignored");

        Ok(())
    }

    #[test]
    fn test_scan_for_venvs_with_violations() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Initialize git repo — clear git env vars so `GIT_DIR` set by hooks doesn't hijack init
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_OBJECT_DIRECTORY")
            .env_remove("GIT_COMMON_DIR")
            .output()?;

        // Create unignored venv
        let venv_dir = temp_dir.path().join("venv");
        fs::create_dir(&venv_dir)?;
        fs::write(venv_dir.join("pyvenv.cfg"), "home = /usr/bin\n")?;

        // Scan should return 2 (policy violation)
        let result = scan_for_venvs_in_dir(temp_dir.path(), false)?;
        assert_eq!(result, 2, "Should return 2 when unignored venvs found");

        Ok(())
    }

    #[test]
    fn test_parse_pyvenv_cfg_missing_file() {
        let result = parse_pyvenv_cfg(Path::new("/nonexistent/pyvenv.cfg"), Path::new("test.cfg"));
        assert!(result.is_err(), "Should return error for missing file");
    }

    #[test]
    fn test_parse_pyvenv_cfg_with_special_characters() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyvenv_path = temp_dir.path().join("pyvenv.cfg");

        // File with special characters and Unicode
        let content = "home = /usr/bin/python🐍\nversion = 3.9.0\n";
        fs::write(&pyvenv_path, content)?;

        let info = parse_pyvenv_cfg(&pyvenv_path, Path::new("test/pyvenv.cfg"))?;

        assert_eq!(info.home, Some("/usr/bin/python🐍".to_string()));
        assert_eq!(info.version, Some("3.9.0".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_pyvenv_cfg_only_equals() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyvenv_path = temp_dir.path().join("pyvenv.cfg");

        // File with line that's only an equals sign
        let content = "=\nhome = /usr/bin\n";
        fs::write(&pyvenv_path, content)?;

        let info = parse_pyvenv_cfg(&pyvenv_path, Path::new("test/pyvenv.cfg"))?;

        // Should still parse valid lines
        assert_eq!(info.home, Some("/usr/bin".to_string()));

        Ok(())
    }

    #[test]
    fn test_parse_pyvenv_cfg_multiple_equals() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyvenv_path = temp_dir.path().join("pyvenv.cfg");

        // Line with multiple = signs
        let content = "home = /usr/bin = something\nversion = 3.9.0\n";
        fs::write(&pyvenv_path, content)?;

        let info = parse_pyvenv_cfg(&pyvenv_path, Path::new("test/pyvenv.cfg"))?;

        // split_once should only split on first =
        assert_eq!(info.home, Some("/usr/bin = something".to_string()));
        assert_eq!(info.version, Some("3.9.0".to_string()));

        Ok(())
    }

    #[test]
    fn test_venv_info_with_none_values() {
        let venv = VenvInfo {
            path: PathBuf::from("test/pyvenv.cfg"),
            home: None,
            version: None,
            include_system_site_packages: None,
        };

        assert_eq!(venv.path, PathBuf::from("test/pyvenv.cfg"));
        assert_eq!(venv.home, None);
        assert_eq!(venv.version, None);
        assert_eq!(venv.include_system_site_packages, None);
    }

    #[test]
    fn test_print_violation_report_empty_venvs() {
        // Test with empty vector - should not panic
        let venvs: Vec<VenvInfo> = vec![];
        print_violation_report(&venvs, true);
        print_violation_report(&venvs, false);
    }

    #[test]
    fn test_parse_pyvenv_cfg_blank_lines_only() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let pyvenv_path = temp_dir.path().join("pyvenv.cfg");

        // File with only blank lines
        let content = "\n\n\n\n";
        fs::write(&pyvenv_path, content)?;

        let info = parse_pyvenv_cfg(&pyvenv_path, Path::new("test/pyvenv.cfg"))?;

        assert_eq!(info.home, None);
        assert_eq!(info.version, None);
        assert_eq!(info.include_system_site_packages, None);

        Ok(())
    }
}
