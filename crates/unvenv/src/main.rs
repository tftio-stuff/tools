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
    AgentArgument, AgentCommand, AgentConfigFile, AgentDoc, AgentDocRequest,
    AgentExample, AgentFailureMode, AgentOperatorMistake, AgentOutputShape, AgentPath,
    AgentSection, AgentTool, AgentUsage, DoctorCheck, DoctorChecks, LicenseType, RepoInfo,
    detect_agent_doc_request, render_agent_help_yaml, render_agent_skill,
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
    /// Show the top-level canonical YAML agent reference.
    #[arg(long, hide = true)]
    agent_help: bool,

    /// Show the top-level Claude skill document.
    #[arg(long, hide = true)]
    agent_skill: bool,

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

fn main() {
    let exit_code = match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            1
        }
    };
    process::exit(exit_code);
}

fn run() -> Result<i32> {
    let raw_args = std::env::args_os().collect::<Vec<_>>();
    if let Some(request) = detect_agent_doc_request(&raw_args) {
        print_agent_doc(request);
        return Ok(0);
    }

    let cli = Cli::parse_from(raw_args);

    if cli.agent_help {
        print_agent_doc(AgentDocRequest::Help);
        return Ok(0);
    }

    if cli.agent_skill {
        print_agent_doc(AgentDocRequest::Skill);
        return Ok(0);
    }

    // Check if stdout is a TTY for decoration
    let is_tty = tftio_cli_common::output::is_tty();

    match cli.command {
        Some(Commands::Version) => {
            if is_tty {
                println!("{} {}", "unvenv".green().bold(), VERSION);
            } else {
                println!("unvenv {VERSION}");
            }
            Ok(0)
        }
        Some(Commands::License) => {
            println!(
                "{}",
                tftio_cli_common::license::display_license("unvenv", LicenseType::MIT)
            );
            Ok(0)
        }
        Some(Commands::Scan) | None => {
            // Default behavior: scan for venv files
            scan_for_venvs(is_tty)
        }
        Some(Commands::Completions { shell }) => {
            tftio_cli_common::completions::generate_completions::<Cli>(shell);
            Ok(0)
        }
        Some(Commands::Doctor) => Ok(tftio_cli_common::doctor::run_doctor(&UnvenvTool)),
        Some(Commands::Update {
            version,
            force,
            install_dir,
        }) => Ok(tftio_cli_common::update::run_update(
            &UnvenvTool::repo_info(),
            UnvenvTool::current_version(),
            version.as_deref(),
            force,
            install_dir.as_deref(),
        )),
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

fn build_agent_doc() -> AgentDoc {
    AgentDoc {
        schema_version: text("1.0"),
        tool: AgentTool {
            name: text("unvenv"),
            binary: text("unvenv"),
            summary: text(
                "Scan a repository for Python virtual environments that are not ignored by Git.",
            ),
        },
        usage: AgentUsage {
            invocation: text("unvenv [--agent-help|--agent-skill] [COMMAND]"),
            notes: vec![
                text("Running `unvenv` with no subcommand is equivalent to `unvenv scan`."),
                text("`unvenv --agent-help` and `unvenv --agent-skill` are top-level only; `unvenv scan --agent-help` is invalid."),
                text("Exit code 2 means policy violation: at least one unignored `pyvenv.cfg` file was detected."),
            ],
        },
        shared_sections: vec![AgentSection {
            title: text("Top-level agent-doc contract"),
            content: text(
                "Agent-doc invocations print to stdout, exit 0, and are hidden from normal human help output.",
            ),
        }],
        commands: vec![
            AgentCommand {
                name: text("scan"),
                summary: text(
                    "Walk the current repository tree, ignore `.git`, respect Git ignore rules when possible, and report unignored virtual environments.",
                ),
                usage: text("unvenv scan"),
                arguments: vec![],
                output_shapes: vec![
                    AgentOutputShape {
                        name: text("scan_success"),
                        format: text("exit 0"),
                        description: text(
                            "No unignored virtual environments were found in the scanned tree.",
                        ),
                    },
                    AgentOutputShape {
                        name: text("scan_violation"),
                        format: text("stdout report + exit code 2"),
                        description: text(
                            "Lists each discovered `pyvenv.cfg`, metadata parsed from it, suggested `.gitignore` entries, and cleanup guidance.",
                        ),
                    },
                ],
            },
            AgentCommand {
                name: text("doctor"),
                summary: text("Run shared CLI health checks plus repository discovery checks."),
                usage: text("unvenv doctor"),
                arguments: vec![],
                output_shapes: vec![AgentOutputShape {
                    name: text("doctor_report"),
                    format: text("stdout text"),
                    description: text("Shared doctor output showing repository and installation health."),
                }],
            },
            AgentCommand {
                name: text("completions"),
                summary: text("Generate shell completion scripts."),
                usage: text("unvenv completions <SHELL>"),
                arguments: vec![positional_arg(
                    "shell",
                    "Shell name accepted by clap_complete, such as `bash`, `zsh`, or `fish`.",
                    true,
                )],
                output_shapes: vec![AgentOutputShape {
                    name: text("completion_script"),
                    format: text("stdout text"),
                    description: text("Shell completion script suitable for redirecting into a file."),
                }],
            },
            AgentCommand {
                name: text("update"),
                summary: text("Install a newer release of `unvenv` using the shared updater."),
                usage: text("unvenv update [--version <SEMVER>] [--force] [--install-dir <PATH>]"),
                arguments: vec![
                    flag_arg("--version", "Install a specific release instead of the latest version.", false),
                    flag_arg("--force", "Reinstall even when already at the target version.", false),
                    flag_arg("--install-dir", "Override the destination directory for the installed binary.", false),
                ],
                output_shapes: vec![AgentOutputShape {
                    name: text("update_report"),
                    format: text("stdout text"),
                    description: text("Shared updater progress and final installation status."),
                }],
            },
        ],
        arguments: vec![
            flag_arg("--agent-help", "Print this canonical YAML reference document.", false),
            flag_arg("--agent-skill", "Print the same content as a Claude skill document.", false),
        ],
        environment_variables: vec![],
        config_files: vec![
            AgentConfigFile {
                path: text(".gitignore"),
                purpose: text("Git ignore rules determine whether a detected virtual environment is reported."),
            },
            AgentConfigFile {
                path: text("~/.config/unvenv"),
                purpose: text("No dedicated config file exists today; runtime behavior is driven by the current working tree and CLI flags."),
            },
        ],
        default_paths: vec![
            AgentPath {
                name: text("scan root"),
                path: text("current working directory"),
                purpose: text("The starting directory for repository discovery and filesystem walking."),
            },
            AgentPath {
                name: text("virtualenv marker"),
                path: text("**/pyvenv.cfg"),
                purpose: text("Every detected environment is identified by this file."),
            },
        ],
        output_shapes: vec![
            AgentOutputShape {
                name: text("scan_violation"),
                format: text("stdout report + exit code 2"),
                description: text("Human-readable report listing each unignored environment and recommended cleanup."),
            },
            AgentOutputShape {
                name: text("scan_success"),
                format: text("exit 0"),
                description: text("Silent success when no policy violations are found."),
            },
        ],
        examples: vec![
            AgentExample {
                name: text("default scan"),
                command: text("unvenv"),
                description: text("Scan the current directory tree for unignored Python virtual environments."),
            },
            AgentExample {
                name: text("explicit scan"),
                command: text("unvenv scan"),
                description: text("Equivalent to the default invocation but explicit in scripts."),
            },
            AgentExample {
                name: text("generate zsh completions"),
                command: text("unvenv completions zsh"),
                description: text("Emit a zsh completion script to stdout."),
            },
            AgentExample {
                name: text("pin an update"),
                command: text("unvenv update --version 1.8.0 --install-dir ~/.local/bin"),
                description: text("Install a specific version into a custom directory."),
            },
        ],
        failure_modes: vec![
            AgentFailureMode {
                name: text("exit code 2"),
                symptom: text("At least one unignored `pyvenv.cfg` file was detected."),
                resolution: text("Add the environment directory to `.gitignore`, remove committed copies from the index, and rerun the scan."),
            },
            AgentFailureMode {
                name: text("repository discovery fallback"),
                symptom: text("No Git repository is found for the current directory."),
                resolution: text("`unvenv` still scans the filesystem, but it cannot consult Git ignore rules, so every `pyvenv.cfg` file is treated as unignored."),
            },
            AgentFailureMode {
                name: text("filesystem read error"),
                symptom: text("Directory walking or file reads fail and the process exits with an error."),
                resolution: text("Inspect stderr, confirm the working tree is readable, and retry."),
            },
        ],
        operator_mistakes: vec![
            AgentOperatorMistake {
                name: text("Running `unvenv scan --agent-help`"),
                symptom: text("The hidden agent-doc flags are rejected when placed after a subcommand."),
                correction: text("Use `unvenv --agent-help` or `unvenv --agent-skill` at the top level."),
            },
            AgentOperatorMistake {
                name: text("Ignoring only `pyvenv.cfg`"),
                symptom: text("The marker file is ignored but the rest of the environment directory is still tracked."),
                correction: text("Ignore the entire environment directory such as `venv/`, not just the config file."),
            },
            AgentOperatorMistake {
                name: text("Assuming exit code 0 means a repository exists"),
                symptom: text("A non-repository directory can still return success if no `pyvenv.cfg` files are found."),
                correction: text("Treat repository discovery and policy cleanliness as separate facts; use `unvenv doctor` when you need repository diagnostics."),
            },
        ],
        constraints: vec![
            text("The scanner skips `.git` directories and does not follow symlinks."),
            text("Only `pyvenv.cfg` markers are used for detection."),
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
