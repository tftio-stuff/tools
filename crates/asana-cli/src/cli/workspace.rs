//! Workspace CLI command implementations.

use super::build_api_client;
use crate::{api, config::Config, error::Result, models::Workspace};
use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use std::io::{IsTerminal, stdout};
use tokio::runtime::Builder as RuntimeBuilder;

/// Primary `workspace` subcommands.
#[derive(Subcommand, Debug)]
pub enum WorkspaceCommand {
    /// List workspaces for the current user.
    List(WorkspaceListArgs),
    /// Display detailed information about a workspace.
    Show(WorkspaceShowArgs),
}

/// Arguments for `workspace list`.
#[derive(Args, Debug)]
pub struct WorkspaceListArgs {
    /// Maximum number of workspaces to retrieve.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Output format.
    #[arg(long, value_enum, default_value = "table")]
    pub format: WorkspaceOutputFormat,
}

/// Arguments for `workspace show`.
#[derive(Args, Debug)]
pub struct WorkspaceShowArgs {
    /// Workspace identifier.
    pub gid: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: WorkspaceOutputFormat,
}

/// Output format choices.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum WorkspaceOutputFormat {
    /// Human-readable table format.
    Table,
    /// JSON format.
    Json,
    /// Detailed human-readable format.
    Detail,
}

/// Parse and execute workspace commands.
///
/// # Errors
/// Returns an error when command execution fails prior to producing an exit code.
pub fn handle_workspace_command(command: WorkspaceCommand, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize async runtime")?;

    runtime.block_on(async move {
        match command {
            WorkspaceCommand::List(args) => list_workspaces_command(&client, args).await,
            WorkspaceCommand::Show(args) => show_workspace_command(&client, args).await,
        }
    })
}

async fn list_workspaces_command(client: &api::ApiClient, args: WorkspaceListArgs) -> Result<()> {
    let params = crate::models::WorkspaceListParams { limit: args.limit };

    let workspaces = api::list_workspaces(client, params).await?;

    if workspaces.is_empty() {
        println!("No workspaces found.");
        return Ok(());
    }

    match args.format {
        WorkspaceOutputFormat::Table => {
            if stdout().is_terminal() {
                println!(
                    "{:<20} {:<40} {}",
                    "GID".bold(),
                    "Name".bold(),
                    "Type".bold()
                );
                println!("{}", "─".repeat(80));
            }
            for workspace in &workspaces {
                let workspace_type = if workspace.is_organization {
                    "Organization"
                } else {
                    "Workspace"
                };

                if stdout().is_terminal() {
                    println!(
                        "{:<20} {:<40} {}",
                        workspace.gid, workspace.name, workspace_type
                    );
                } else {
                    println!("{}\t{}\t{}", workspace.gid, workspace.name, workspace_type);
                }
            }
            if stdout().is_terminal() {
                println!("\n{} workspaces listed.", workspaces.len());
            }
        }
        WorkspaceOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&workspaces)
                .context("failed to serialize workspaces to JSON")?;
            println!("{json}");
        }
        WorkspaceOutputFormat::Detail => {
            for (i, workspace) in workspaces.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                print_workspace_detail(workspace);
            }
        }
    }

    Ok(())
}

async fn show_workspace_command(client: &api::ApiClient, args: WorkspaceShowArgs) -> Result<()> {
    let workspace = api::get_workspace(client, &args.gid).await?;

    if args.format == WorkspaceOutputFormat::Json {
        let json = serde_json::to_string_pretty(&workspace)
            .context("failed to serialize workspace to JSON")?;
        println!("{json}");
    } else {
        print_workspace_detail(&workspace);
    }

    Ok(())
}

fn print_workspace_detail(workspace: &Workspace) {
    let gid = &workspace.gid;
    let name = &workspace.name;
    println!("GID: {gid}");
    println!("Name: {name}");
    println!(
        "Type: {}",
        if workspace.is_organization {
            "Organization"
        } else {
            "Workspace"
        }
    );

    if !workspace.email_domains.is_empty() {
        println!("Email Domains:");
        for domain in &workspace.email_domains {
            println!("  - {domain}");
        }
    }
}
