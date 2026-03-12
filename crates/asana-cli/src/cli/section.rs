//! Section CLI command implementations.

use super::build_api_client;
use crate::{
    api,
    config::Config,
    error::Result,
    models::{SectionCreateData, SectionCreateRequest},
};
use anyhow::Context;
use clap::{Args, Subcommand};
use std::io::{IsTerminal, stdout};
use tokio::runtime::Builder as RuntimeBuilder;

#[derive(Subcommand, Debug)]
pub enum SectionCommand {
    /// List sections in a project.
    List(SectionListArgs),
    /// Display detailed information about a section.
    Show(SectionShowArgs),
    /// Create a new section in a project.
    Create(SectionCreateArgs),
    /// List tasks in a section.
    Tasks(SectionTasksArgs),
}

#[derive(Args, Debug)]
pub struct SectionListArgs {
    /// Project identifier (gid) to list sections from.
    #[arg(long)]
    pub project: String,
    /// Output format (table, json, csv).
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Args, Debug)]
pub struct SectionShowArgs {
    /// Section identifier (gid).
    #[arg(value_name = "SECTION")]
    pub section: String,
    /// Additional fields to request from the API.
    #[arg(long, value_name = "FIELD")]
    pub fields: Vec<String>,
    /// Output format (table, json).
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Args, Debug)]
pub struct SectionCreateArgs {
    /// Section name.
    #[arg(long)]
    pub name: String,
    /// Project identifier (gid) to create the section in.
    #[arg(long)]
    pub project: String,
    /// Insert before this section gid.
    #[arg(long = "insert-before")]
    pub insert_before: Option<String>,
    /// Insert after this section gid.
    #[arg(long = "insert-after")]
    pub insert_after: Option<String>,
    /// Output format (table, json).
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Args, Debug)]
pub struct SectionTasksArgs {
    /// Section identifier (gid).
    #[arg(value_name = "SECTION")]
    pub section: String,
    /// Additional fields to request from the API.
    #[arg(long, value_name = "FIELD")]
    pub fields: Vec<String>,
    /// Output format (table, json, csv).
    #[arg(long)]
    pub output: Option<String>,
}

pub fn execute_section_command(cmd: SectionCommand, config: &Config) -> Result<()> {
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    runtime.block_on(async { execute_section_command_async(cmd, config).await })
}

async fn execute_section_command_async(cmd: SectionCommand, config: &Config) -> Result<()> {
    match cmd {
        SectionCommand::List(args) => list_sections(args, config).await,
        SectionCommand::Show(args) => show_section(args, config).await,
        SectionCommand::Create(args) => create_section(args, config).await,
        SectionCommand::Tasks(args) => list_section_tasks(args, config).await,
    }
}

async fn list_sections(args: SectionListArgs, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;
    let sections = api::list_sections(&client, &args.project).await?;

    let is_tty = stdout().is_terminal();
    let output_format = args
        .output
        .as_deref()
        .unwrap_or(if is_tty { "table" } else { "json" });

    match output_format {
        "json" => {
            let json = serde_json::to_string_pretty(&sections)?;
            println!("{json}");
        }
        "csv" => {
            println!("gid,name,project_gid,project_name");
            for section in sections {
                let project_gid = section.project.as_ref().map_or("", |p| p.gid.as_str());
                let project_name = section
                    .project
                    .as_ref()
                    .and_then(|p| p.name.as_deref())
                    .unwrap_or("");
                println!(
                    "{},{},{},{}",
                    section.gid, section.name, project_gid, project_name
                );
            }
        }
        _ => {
            if sections.is_empty() {
                println!("No sections found in project.");
            } else {
                println!("{:<20} {:<30} {:<20}", "GID", "NAME", "PROJECT");
                println!("{}", "-".repeat(72));
                for section in sections {
                    let project_label = section.project.as_ref().map_or_else(
                        || "N/A".to_string(),
                        super::super::models::section::SectionProjectReference::label,
                    );
                    println!(
                        "{:<20} {:<30} {:<20}",
                        section.gid, section.name, project_label
                    );
                }
            }
        }
    }

    Ok(())
}

async fn show_section(args: SectionShowArgs, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;
    let section = api::get_section(&client, &args.section, args.fields).await?;

    let is_tty = stdout().is_terminal();
    let output_format = args
        .output
        .as_deref()
        .unwrap_or(if is_tty { "table" } else { "json" });

    if output_format == "json" {
        let json = serde_json::to_string_pretty(&section)?;
        println!("{json}");
    } else {
        println!("Section: {}", section.name);
        println!("GID: {}", section.gid);
        if let Some(project) = &section.project {
            println!(
                "Project: {} ({})",
                project.name.as_deref().unwrap_or("N/A"),
                project.gid
            );
        }
        if let Some(created_at) = &section.created_at {
            println!("Created: {created_at}");
        }
    }

    Ok(())
}

async fn create_section(args: SectionCreateArgs, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let request = SectionCreateRequest {
        data: SectionCreateData {
            name: args.name,
            insert_before: args.insert_before,
            insert_after: args.insert_after,
        },
    };

    let section = api::create_section(&client, &args.project, request).await?;

    let is_tty = stdout().is_terminal();
    let output_format = args
        .output
        .as_deref()
        .unwrap_or(if is_tty { "table" } else { "json" });

    if output_format == "json" {
        let json = serde_json::to_string_pretty(&section)?;
        println!("{json}");
    } else {
        println!("Created section: {}", section.name);
        println!("GID: {}", section.gid);
        if let Some(project) = &section.project {
            println!(
                "Project: {} ({})",
                project.name.as_deref().unwrap_or("N/A"),
                project.gid
            );
        }
    }

    Ok(())
}

async fn list_section_tasks(args: SectionTasksArgs, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;
    let tasks = api::get_section_tasks(&client, &args.section, args.fields).await?;

    let is_tty = stdout().is_terminal();
    let output_format = args
        .output
        .as_deref()
        .unwrap_or(if is_tty { "table" } else { "json" });

    match output_format {
        "json" => {
            let json = serde_json::to_string_pretty(&tasks)?;
            println!("{json}");
        }
        "csv" => {
            println!("gid,name,completed,assignee");
            for task in tasks {
                let assignee = task.assignee.as_ref().map_or_else(
                    || "Unassigned".to_string(),
                    super::super::models::user::UserReference::label,
                );
                println!("{},{},{},{}", task.gid, task.name, task.completed, assignee);
            }
        }
        _ => {
            if tasks.is_empty() {
                println!("No tasks found in section.");
            } else {
                println!(
                    "{:<20} {:<40} {:<10} {:<20}",
                    "GID", "NAME", "STATUS", "ASSIGNEE"
                );
                println!("{}", "-".repeat(92));
                for task in tasks {
                    let status = if task.completed { "Done" } else { "Open" };
                    let assignee = task.assignee.as_ref().map_or_else(
                        || "Unassigned".to_string(),
                        super::super::models::user::UserReference::label,
                    );
                    println!(
                        "{:<20} {:<40} {:<10} {:<20}",
                        task.gid, task.name, status, assignee
                    );
                }
            }
        }
    }

    Ok(())
}
