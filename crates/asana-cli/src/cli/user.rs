//! User CLI command implementations.

use super::build_api_client;
use crate::{api, config::Config, error::Result, models::User};
use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use std::io::{IsTerminal, stdout};
use tokio::runtime::Builder as RuntimeBuilder;

/// Primary `user` subcommands.
#[derive(Subcommand, Debug)]
pub enum UserCommand {
    /// List users in a workspace.
    List(UserListArgs),
    /// Display detailed information about a user.
    Show(UserShowArgs),
    /// Show current authenticated user.
    Me(UserMeArgs),
}

/// Arguments for `user list`.
#[derive(Args, Debug)]
pub struct UserListArgs {
    /// Workspace identifier (required).
    #[arg(long)]
    pub workspace: Option<String>,
    /// Maximum number of users to retrieve.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Output format.
    #[arg(long, value_enum, default_value = "table")]
    pub format: UserOutputFormat,
}

/// Arguments for `user show`.
#[derive(Args, Debug)]
pub struct UserShowArgs {
    /// User identifier.
    pub gid: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: UserOutputFormat,
}

/// Arguments for `user me`.
#[derive(Args, Debug)]
pub struct UserMeArgs {
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: UserOutputFormat,
}

/// Output format choices.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum UserOutputFormat {
    /// Human-readable table format.
    Table,
    /// JSON format.
    Json,
    /// Detailed human-readable format.
    Detail,
}

/// Parse and execute user commands.
///
/// # Errors
/// Returns an error when command execution fails prior to producing an exit code.
pub fn handle_user_command(command: UserCommand, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize async runtime")?;

    runtime.block_on(async move {
        match command {
            UserCommand::List(args) => list_users_command(&client, config, args).await,
            UserCommand::Show(args) => show_user_command(&client, args).await,
            UserCommand::Me(args) => show_current_user_command(&client, args).await,
        }
    })
}

async fn list_users_command(
    client: &api::ApiClient,
    config: &Config,
    args: UserListArgs,
) -> Result<()> {
    let workspace_gid = args
        .workspace
        .as_deref()
        .or_else(|| config.default_workspace())
        .context("workspace is required; provide --workspace or set default_workspace in config")?;

    let params = crate::models::UserListParams {
        workspace_gid: workspace_gid.to_string(),
        limit: args.limit,
    };

    let users = api::list_users(client, params).await?;

    if users.is_empty() {
        println!("No users found in workspace {workspace_gid}.");
        return Ok(());
    }

    match args.format {
        UserOutputFormat::Table => {
            if stdout().is_terminal() {
                println!(
                    "{:<20} {:<30} {}",
                    "GID".bold(),
                    "Name".bold(),
                    "Email".bold()
                );
                println!("{}", "─".repeat(80));
            }
            for user in &users {
                let email = user.email.as_deref().unwrap_or("N/A");

                if stdout().is_terminal() {
                    println!("{:<20} {:<30} {}", user.gid, user.name, email);
                } else {
                    println!("{}\t{}\t{}", user.gid, user.name, email);
                }
            }
            if stdout().is_terminal() {
                println!("\n{} users listed.", users.len());
            }
        }
        UserOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&users)
                .context("failed to serialize users to JSON")?;
            println!("{json}");
        }
        UserOutputFormat::Detail => {
            for (i, user) in users.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                print_user_detail(user);
            }
        }
    }

    Ok(())
}

async fn show_user_command(client: &api::ApiClient, args: UserShowArgs) -> Result<()> {
    let user = api::get_user(client, &args.gid).await?;

    if args.format == UserOutputFormat::Json {
        let json =
            serde_json::to_string_pretty(&user).context("failed to serialize user to JSON")?;
        println!("{json}");
    } else {
        print_user_detail(&user);
    }

    Ok(())
}

async fn show_current_user_command(client: &api::ApiClient, args: UserMeArgs) -> Result<()> {
    let user = api::get_current_user(client).await?;

    if args.format == UserOutputFormat::Json {
        let json =
            serde_json::to_string_pretty(&user).context("failed to serialize user to JSON")?;
        println!("{json}");
    } else {
        print_user_detail(&user);
    }

    Ok(())
}

fn print_user_detail(user: &User) {
    let gid = &user.gid;
    let name = &user.name;
    println!("GID: {gid}");
    println!("Name: {name}");

    if let Some(ref email) = user.email {
        println!("Email: {email}");
    }

    if !user.workspaces.is_empty() {
        println!("\nWorkspaces:");
        for workspace in &user.workspaces {
            let ws_name = workspace.name.as_deref().unwrap_or(&workspace.gid);
            println!("  - {} ({})", ws_name, workspace.gid);
        }
    }

    if let Some(ref photo) = user.photo
        && let Some(ref url) = photo.image_128x128
    {
        println!("\nPhoto: {url}");
    }
}
