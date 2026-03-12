//! Command-line interface entry points for the Asana CLI.

mod custom_field;
mod project;
mod section;
mod tag;
mod task;
mod user;
mod workspace;

use crate::api::{ApiClient, ApiError, AuthToken};
use crate::config::Config;
use crate::error::Result;
use anyhow::{Context, anyhow};
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use colored::Colorize;
use custom_field::CustomFieldCommand;
use project::ProjectCommand;
use secrecy::SecretString;
use section::SectionCommand;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;
use tag::TagCommand;
use task::TaskCommand;
use tokio::runtime::Builder as RuntimeBuilder;
use tracing::{debug, info};
use user::UserCommand;
use tftio_cli_common::{DoctorCheck, DoctorChecks, LicenseType, RepoInfo};
use workspace::WorkspaceCommand;

const MANPAGE_SOURCE: &str = include_str!("../../docs/man/asana-cli.1");

const VERSION: &str = match option_env!("CARGO_PKG_VERSION") {
    Some(version) => version,
    None => "unknown",
};

#[derive(Parser, Debug)]
#[command(name = "asana-cli")]
#[command(about = "An interface to the Asana API")]
#[command(version = VERSION)]
struct Cli {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show version information.
    Version,
    /// Show license information.
    License,
    /// Manage persisted configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Task operations.
    Task {
        #[command(subcommand)]
        command: Box<TaskCommand>,
    },
    /// Project operations.
    Project {
        #[command(subcommand)]
        command: Box<ProjectCommand>,
    },
    /// Section operations.
    Section {
        #[command(subcommand)]
        command: Box<SectionCommand>,
    },
    /// Tag operations.
    Tag {
        #[command(subcommand)]
        command: Box<TagCommand>,
    },
    /// Custom field operations.
    #[command(name = "custom-field")]
    CustomField {
        #[command(subcommand)]
        command: Box<CustomFieldCommand>,
    },
    /// Workspace operations.
    Workspace {
        #[command(subcommand)]
        command: Box<WorkspaceCommand>,
    },
    /// User operations.
    User {
        #[command(subcommand)]
        command: Box<UserCommand>,
    },
    /// Generate shell completion scripts.
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },
    /// Generate the man page (roff output).
    Manpage {
        /// Output directory for the generated man page (defaults to stdout).
        #[arg(long)]
        dir: Option<PathBuf>,
    },
    /// Check health and configuration.
    Doctor,
    /// Update to the latest version.
    Update {
        /// Specific version to install.
        #[arg(long)]
        version: Option<String>,
        /// Force update even if already up-to-date.
        #[arg(short, long)]
        force: bool,
        /// Custom installation directory.
        #[arg(long)]
        install_dir: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    /// Store configuration values.
    Set {
        #[command(subcommand)]
        command: ConfigSetCommand,
    },
    /// Display the current configuration (token redacted).
    Get,
    /// Validate the stored Personal Access Token against the Asana API.
    Test,
}

#[derive(Subcommand, Debug)]
enum ConfigSetCommand {
    /// Store the Personal Access Token.
    Token {
        /// Personal Access Token value; omit to be prompted securely.
        #[arg(long)]
        token: Option<String>,
    },
    /// Store the default workspace gid.
    Workspace {
        /// Workspace gid to use when none is supplied on the command line.
        #[arg(long, value_name = "GID")]
        workspace: Option<String>,
        /// Clear the stored default workspace.
        #[arg(long)]
        clear: bool,
    },
    /// Store the default assignee identifier.
    Assignee {
        /// Identifier (email or gid) that should replace the `me` alias.
        #[arg(long, value_name = "ID")]
        assignee: Option<String>,
        /// Clear the stored default assignee.
        #[arg(long)]
        clear: bool,
    },
    /// Store the default project identifier.
    Project {
        /// Project gid to use when none is supplied on the command line.
        #[arg(long, value_name = "GID")]
        project: Option<String>,
        /// Clear the stored default project.
        #[arg(long)]
        clear: bool,
    },
}

/// Parse and execute CLI commands, returning the desired process exit code.
///
/// # Errors
/// Returns an error when command execution fails prior to producing an exit code.
pub fn run() -> Result<i32> {
    let cli = Cli::parse();
    debug!(?cli, "parsed CLI arguments");

    let mut config = Config::load()?;
    debug!(
        config_path = %config.path().display(),
        "configuration handle prepared"
    );

    let exit_code = match cli.command {
        Commands::Version => {
            print_version();
            0
        }
        Commands::License => {
            print_license();
            0
        }
        Commands::Config { command } => {
            handle_config_command(command, &mut config)?;
            0
        }
        Commands::Task { command } => {
            task::handle_task_command(*command, &config)?;
            0
        }
        Commands::Project { command } => {
            handle_project_command(*command, &config)?;
            0
        }
        Commands::Section { command } => {
            handle_section_command(*command, &config)?;
            0
        }
        Commands::Tag { command } => {
            handle_tag_command(*command, &config)?;
            0
        }
        Commands::CustomField { command } => {
            handle_custom_field_command(*command, &config)?;
            0
        }
        Commands::Workspace { command } => {
            handle_workspace_command(*command, &config)?;
            0
        }
        Commands::User { command } => {
            handle_user_command(*command, &config)?;
            0
        }
        Commands::Completions { shell } => {
            tftio_cli_common::completions::generate_completions::<Cli>(shell);
            0
        }
        Commands::Manpage { dir } => {
            write_manpage(dir)?;
            0
        }
        Commands::Doctor => {
            struct AsanaCliDoctor;

            impl DoctorChecks for AsanaCliDoctor {
                fn repo_info() -> RepoInfo {
                    RepoInfo::new("tftio", "asana-cli")
                }

                fn current_version() -> &'static str {
                    VERSION
                }

                fn tool_checks(&self) -> Vec<DoctorCheck> {
                    crate::doctor::tool_specific_checks()
                }
            }

            let tool = AsanaCliDoctor;
            let exit = tftio_cli_common::doctor::run_doctor(&tool);
            info!(exit_code = exit, "doctor command completed");
            exit
        }
        Commands::Update {
            version: _,
            force: _,
            install_dir: _,
        } => {
            eprintln!(
                "Self-update feature has been removed. Please install the latest version manually."
            );
            eprintln!("Visit: https://github.com/tftio/asana-cli/releases");
            1
        }
    };

    Ok(exit_code)
}

fn print_version() {
    println!("{} {}", "asana-cli".green().bold(), VERSION);
}

fn print_license() {
    println!(
        "{}",
        tftio_cli_common::license::display_license("asana-cli", LicenseType::MIT)
    );
}

fn write_manpage(dir: Option<PathBuf>) -> Result<()> {
    if let Some(path) = dir {
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create manpage directory {}", path.display()))?;
        let output = path.join("asana-cli.1");
        let mut file = File::create(&output)
            .with_context(|| format!("failed to create manpage file {}", output.display()))?;
        write!(file, "{MANPAGE_SOURCE}")
            .map_err(|err| anyhow!("failed to write manpage: {err}"))?;
        println!("Man page written to {}", output.display());
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        write!(handle, "{MANPAGE_SOURCE}")
            .map_err(|err| anyhow!("failed to write manpage: {err}"))?;
    }

    Ok(())
}

fn handle_config_command(command: ConfigCommand, config: &mut Config) -> Result<()> {
    match command {
        ConfigCommand::Set { command } => handle_config_set(command, config),
        ConfigCommand::Get => {
            handle_config_get(config);
            Ok(())
        }
        ConfigCommand::Test => handle_config_test(config),
    }
}

fn handle_config_set(command: ConfigSetCommand, config: &mut Config) -> Result<()> {
    match command {
        ConfigSetCommand::Token { token } => {
            let value = match token {
                Some(value) => value,
                None => rpassword::prompt_password("Enter Personal Access Token: ")
                    .context("failed to read token from prompt")?,
            };

            if value.trim().is_empty() {
                return Err(anyhow!("token value cannot be empty"));
            }

            let secret = SecretString::new(value.into());
            config
                .store_personal_access_token(&secret)
                .context("failed to store Personal Access Token")?;
            println!("Personal Access Token stored in configuration file.");
            Ok(())
        }
        ConfigSetCommand::Workspace { workspace, clear } => {
            if clear {
                config
                    .set_default_workspace(None)
                    .context("failed to clear default workspace")?;
                println!("Default workspace cleared.");
                return Ok(());
            }

            let value = workspace
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow!("provide --workspace <gid> or use --clear"))?;

            config
                .set_default_workspace(Some(value.to_string()))
                .context("failed to store default workspace")?;
            println!("Default workspace stored in configuration file.");
            Ok(())
        }
        ConfigSetCommand::Assignee { assignee, clear } => {
            if clear {
                config
                    .set_default_assignee(None)
                    .context("failed to clear default assignee")?;
                println!("Default assignee cleared.");
                return Ok(());
            }

            let value = assignee
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow!("provide --assignee <id> or use --clear"))?;

            config
                .set_default_assignee(Some(value.to_string()))
                .context("failed to store default assignee")?;
            println!("Default assignee stored in configuration file.");
            Ok(())
        }
        ConfigSetCommand::Project { project, clear } => {
            if clear {
                config
                    .set_default_project(None)
                    .context("failed to clear default project")?;
                println!("Default project cleared.");
                return Ok(());
            }

            let value = project
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow!("provide --project <gid> or use --clear"))?;

            config
                .set_default_project(Some(value.to_string()))
                .context("failed to store default project")?;
            println!("Default project stored in configuration file.");
            Ok(())
        }
    }
}

fn handle_config_get(config: &Config) {
    println!("Configuration file: {}", config.path().display());
    println!("API base URL: {}", config.effective_api_base_url());
    println!(
        "Default workspace: {}",
        config
            .default_workspace()
            .filter(|workspace| !workspace.is_empty())
            .unwrap_or("not set")
    );
    println!(
        "Default assignee: {}",
        config
            .default_assignee()
            .filter(|assignee| !assignee.is_empty())
            .unwrap_or("not set")
    );
    println!(
        "Default project: {}",
        config
            .default_project()
            .filter(|project| !project.is_empty())
            .unwrap_or("not set")
    );

    if let Some(_token) = config.personal_access_token() {
        let status = if config.environment_token_available() {
            "provided via environment variable"
        } else if config.has_persisted_token() {
            "stored in configuration file"
        } else {
            "available"
        };
        println!("Personal Access Token: {status}");
    } else {
        println!("Personal Access Token: not set");
    }
}

fn handle_config_test(config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialise async runtime")?;

    runtime.block_on(async move {
        match client.get_current_user().await {
            Ok(payload) => {
                let user_name = payload
                    .get("data")
                    .and_then(|data| data.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or("unknown user");
                println!("Personal Access Token validated for {user_name}.");
                Ok(())
            }
            Err(ApiError::Authentication(_)) => Err(anyhow!(
                "authentication failed; verify your Personal Access Token"
            )),
            Err(ApiError::RateLimited { retry_after, .. }) => Err(anyhow!(
                "Asana rate limited the request. Retry after {:.1} seconds",
                retry_after.as_secs_f32()
            )),
            Err(ApiError::Offline { .. }) => Err(anyhow!(
                "offline mode enabled; disable offline mode to contact Asana"
            )),
            Err(err) => Err(anyhow!(err)),
        }
    })
}

pub(super) fn build_api_client(config: &Config) -> Result<ApiClient> {
    let token = config.personal_access_token().ok_or_else(|| {
        anyhow!("no Personal Access Token found; run `asana-cli config set token`")
    })?;

    let auth_token = AuthToken::new(token);
    let cache_dir = config.cache_dir().to_path_buf();

    let client = ApiClient::builder(auth_token)
        .base_url(config.effective_api_base_url().to_string())
        .cache_dir(cache_dir)
        .build()?;

    Ok(client)
}

fn handle_project_command(command: ProjectCommand, config: &Config) -> Result<()> {
    project::handle_project_command(command, config)
}

fn handle_section_command(command: SectionCommand, config: &Config) -> Result<()> {
    section::execute_section_command(command, config)
}

fn handle_tag_command(command: TagCommand, config: &Config) -> Result<()> {
    tag::handle_tag_command(command, config)
}

fn handle_custom_field_command(command: CustomFieldCommand, config: &Config) -> Result<()> {
    custom_field::handle_custom_field_command(command, config)
}

fn handle_workspace_command(command: WorkspaceCommand, config: &Config) -> Result<()> {
    workspace::handle_workspace_command(command, config)
}

fn handle_user_command(command: UserCommand, config: &Config) -> Result<()> {
    user::handle_user_command(command, config)
}
