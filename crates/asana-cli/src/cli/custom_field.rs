//! Custom field CLI command implementations.

use super::build_api_client;
use crate::{api, config::Config, error::Result, models::CustomField};
use anyhow::Context;
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use std::io::{IsTerminal, stdout};
use tokio::runtime::Builder as RuntimeBuilder;

/// Primary `custom-field` subcommands.
#[derive(Subcommand, Debug)]
pub enum CustomFieldCommand {
    /// List custom fields in a workspace.
    List(CustomFieldListArgs),
    /// Display detailed information about a custom field.
    Show(CustomFieldShowArgs),
}

/// Arguments for `custom-field list`.
#[derive(Args, Debug)]
pub struct CustomFieldListArgs {
    /// Workspace identifier (required).
    #[arg(long)]
    pub workspace: Option<String>,
    /// Maximum number of custom fields to retrieve.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Output format.
    #[arg(long, value_enum, default_value = "table")]
    pub format: CustomFieldOutputFormat,
}

/// Arguments for `custom-field show`.
#[derive(Args, Debug)]
pub struct CustomFieldShowArgs {
    /// Custom field identifier.
    pub gid: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: CustomFieldOutputFormat,
}

/// Output format choices.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum CustomFieldOutputFormat {
    /// Human-readable table format.
    Table,
    /// JSON format.
    Json,
    /// Detailed human-readable format.
    Detail,
}

/// Parse and execute custom field commands.
///
/// # Errors
/// Returns an error when command execution fails prior to producing an exit code.
pub fn handle_custom_field_command(command: CustomFieldCommand, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize async runtime")?;

    runtime.block_on(async move {
        match command {
            CustomFieldCommand::List(args) => {
                list_custom_fields_command(&client, config, args).await
            }
            CustomFieldCommand::Show(args) => show_custom_field_command(&client, args).await,
        }
    })
}

async fn list_custom_fields_command(
    client: &api::ApiClient,
    config: &Config,
    args: CustomFieldListArgs,
) -> Result<()> {
    let workspace_gid = args
        .workspace
        .as_deref()
        .or_else(|| config.default_workspace())
        .context("workspace is required; provide --workspace or set default_workspace in config")?;

    let fields = api::list_custom_fields(client, workspace_gid, args.limit).await?;

    if fields.is_empty() {
        println!("No custom fields found in workspace {workspace_gid}.");
        return Ok(());
    }

    match args.format {
        CustomFieldOutputFormat::Table => {
            if stdout().is_terminal() {
                println!(
                    "{:<20} {:<30} {:<15} {}",
                    "GID".bold(),
                    "Name".bold(),
                    "Type".bold(),
                    "Description".bold()
                );
                println!("{}", "─".repeat(100));
            }
            for field in &fields {
                let description = field.description.as_deref().unwrap_or("");
                let desc_preview = if description.len() > 35 {
                    format!("{}...", &description[..35])
                } else {
                    description.to_string()
                };

                if stdout().is_terminal() {
                    println!(
                        "{:<20} {:<30} {:<15} {}",
                        field.gid,
                        field.name,
                        format!("{:?}", field.field_type),
                        desc_preview
                    );
                } else {
                    println!(
                        "{}\t{}\t{:?}\t{}",
                        field.gid, field.name, field.field_type, desc_preview
                    );
                }
            }
            if stdout().is_terminal() {
                println!("\n{} custom fields listed.", fields.len());
            }
        }
        CustomFieldOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&fields)
                .context("failed to serialize custom fields to JSON")?;
            println!("{json}");
        }
        CustomFieldOutputFormat::Detail => {
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                print_custom_field_detail(field);
            }
        }
    }

    Ok(())
}

async fn show_custom_field_command(
    client: &api::ApiClient,
    args: CustomFieldShowArgs,
) -> Result<()> {
    let field = api::get_custom_field(client, &args.gid).await?;

    match args.format {
        CustomFieldOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&field)
                .context("failed to serialize custom field to JSON")?;
            println!("{json}");
        }
        _ => {
            print_custom_field_detail(&field);
        }
    }

    Ok(())
}

fn print_custom_field_detail(field: &CustomField) {
    let gid = &field.gid;
    let name = &field.name;
    println!("GID: {gid}");
    println!("Name: {name}");
    println!("Type: {:?}", field.field_type);

    if let Some(ref description) = field.description {
        println!("Description: {description}");
    }

    if let Some(enabled) = field.enabled {
        println!("Enabled: {enabled}");
    }

    // Type-specific details
    match field.field_type {
        crate::models::CustomFieldType::Enum | crate::models::CustomFieldType::MultiEnum => {
            println!("\nEnum Options:");
            if let Some(options) = field.extra.get("enum_options")
                && let Some(options_array) = options.as_array()
            {
                for opt in options_array {
                    if let Some(opt_name) = opt.get("name").and_then(serde_json::Value::as_str) {
                        let opt_gid = opt
                            .get("gid")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("?");
                        let opt_enabled = opt
                            .get("enabled")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(true);
                        let status = if opt_enabled { "" } else { " (disabled)" };
                        println!("  - {opt_name} ({opt_gid}){status}");
                    }
                }
            }
        }
        crate::models::CustomFieldType::Number | crate::models::CustomFieldType::Percent => {
            if let Some(precision) = field
                .extra
                .get("precision")
                .and_then(serde_json::Value::as_i64)
            {
                println!("Precision: {precision}");
            }
        }
        crate::models::CustomFieldType::Currency => {
            if let Some(ref currency_code) = field.currency_code {
                println!("Currency: {currency_code}");
            }
        }
        _ => {}
    }
}
