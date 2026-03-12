//! Tag CLI command implementations.

use super::build_api_client;
use crate::{
    api,
    config::Config,
    error::Result,
    models::{Tag, TagColor, TagCreateBuilder, TagListParams, TagUpdateBuilder},
};
use anyhow::{Context, anyhow};
use clap::{Args, Subcommand, ValueEnum};
use colored::Colorize;
use std::io::{IsTerminal, stdout};
use tokio::runtime::Builder as RuntimeBuilder;

/// Primary `tag` subcommands.
#[derive(Subcommand, Debug)]
pub enum TagCommand {
    /// List tags in a workspace.
    List(TagListArgs),
    /// Display detailed information about a tag.
    Show(TagShowArgs),
    /// Create a new tag.
    Create(TagCreateArgs),
    /// Update an existing tag.
    Update(TagUpdateArgs),
    /// Delete a tag.
    Delete(TagDeleteArgs),
}

/// Arguments for `tag list`.
#[derive(Args, Debug)]
pub struct TagListArgs {
    /// Workspace identifier (required).
    #[arg(long)]
    pub workspace: Option<String>,
    /// Maximum number of tags to retrieve.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Output format.
    #[arg(long, value_enum, default_value = "table")]
    pub format: TagOutputFormat,
}

/// Arguments for `tag show`.
#[derive(Args, Debug)]
pub struct TagShowArgs {
    /// Tag identifier.
    pub gid: String,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: TagOutputFormat,
}

/// Arguments for `tag create`.
#[derive(Args, Debug)]
pub struct TagCreateArgs {
    /// Tag name (required).
    #[arg(long)]
    pub name: String,
    /// Workspace identifier (required).
    #[arg(long)]
    pub workspace: Option<String>,
    /// Tag color.
    #[arg(long, value_enum)]
    pub color: Option<TagColorArg>,
    /// Tag notes or description.
    #[arg(long)]
    pub notes: Option<String>,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: TagOutputFormat,
}

/// Arguments for `tag update`.
#[derive(Args, Debug)]
pub struct TagUpdateArgs {
    /// Tag identifier.
    pub gid: String,
    /// New tag name.
    #[arg(long)]
    pub name: Option<String>,
    /// New tag color.
    #[arg(long, value_enum)]
    pub color: Option<TagColorArg>,
    /// New tag notes.
    #[arg(long)]
    pub notes: Option<String>,
    /// Clear notes.
    #[arg(long)]
    pub clear_notes: bool,
    /// Output format.
    #[arg(long, value_enum, default_value = "detail")]
    pub format: TagOutputFormat,
}

/// Arguments for `tag delete`.
#[derive(Args, Debug)]
pub struct TagDeleteArgs {
    /// Tag identifier.
    pub gid: String,
    /// Skip confirmation prompt.
    #[arg(long)]
    pub yes: bool,
}

/// Output format for tag commands.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TagOutputFormat {
    /// Tabular listing.
    Table,
    /// Detailed view.
    Detail,
    /// JSON output.
    Json,
}

/// Tag color argument values.
#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum TagColorArg {
    DarkBlue,
    DarkBrown,
    DarkGreen,
    DarkOrange,
    DarkPink,
    DarkPurple,
    DarkRed,
    DarkTeal,
    DarkWarmGray,
    LightBlue,
    LightBrown,
    LightGreen,
    LightOrange,
    LightPink,
    LightPurple,
    LightRed,
    LightTeal,
    LightWarmGray,
}

impl From<TagColorArg> for TagColor {
    fn from(arg: TagColorArg) -> Self {
        match arg {
            TagColorArg::DarkBlue => Self::DarkBlue,
            TagColorArg::DarkBrown => Self::DarkBrown,
            TagColorArg::DarkGreen => Self::DarkGreen,
            TagColorArg::DarkOrange => Self::DarkOrange,
            TagColorArg::DarkPink => Self::DarkPink,
            TagColorArg::DarkPurple => Self::DarkPurple,
            TagColorArg::DarkRed => Self::DarkRed,
            TagColorArg::DarkTeal => Self::DarkTeal,
            TagColorArg::DarkWarmGray => Self::DarkWarmGray,
            TagColorArg::LightBlue => Self::LightBlue,
            TagColorArg::LightBrown => Self::LightBrown,
            TagColorArg::LightGreen => Self::LightGreen,
            TagColorArg::LightOrange => Self::LightOrange,
            TagColorArg::LightPink => Self::LightPink,
            TagColorArg::LightPurple => Self::LightPurple,
            TagColorArg::LightRed => Self::LightRed,
            TagColorArg::LightTeal => Self::LightTeal,
            TagColorArg::LightWarmGray => Self::LightWarmGray,
        }
    }
}

/// Execute a tag command.
pub fn handle_tag_command(command: TagCommand, config: &Config) -> Result<()> {
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialise async runtime")?;

    match command {
        TagCommand::List(args) => runtime.block_on(handle_tag_list(args, config)),
        TagCommand::Show(args) => runtime.block_on(handle_tag_show(args, config)),
        TagCommand::Create(args) => runtime.block_on(handle_tag_create(args, config)),
        TagCommand::Update(args) => runtime.block_on(handle_tag_update(args, config)),
        TagCommand::Delete(args) => runtime.block_on(handle_tag_delete(args, config)),
    }
}

async fn handle_tag_list(args: TagListArgs, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let workspace = args
        .workspace
        .or_else(|| config.default_workspace().map(String::from))
        .ok_or_else(|| anyhow!("workspace is required; provide --workspace or set a default"))?;

    let params = TagListParams {
        workspace,
        limit: args.limit,
    };

    let tags = api::list_tags(&client, params)
        .await
        .context("failed to list tags")?;

    match args.format {
        TagOutputFormat::Table => render_tag_table(&tags),
        TagOutputFormat::Detail => {
            for tag in &tags {
                render_tag_detail(tag);
                println!();
            }
        }
        TagOutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(&tags).context("failed to serialize tags to JSON")?;
            println!("{json}");
        }
    }

    Ok(())
}

async fn handle_tag_show(args: TagShowArgs, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let tag = api::get_tag(&client, &args.gid)
        .await
        .context("failed to retrieve tag")?;

    match args.format {
        TagOutputFormat::Detail | TagOutputFormat::Table => render_tag_detail(&tag),
        TagOutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(&tag).context("failed to serialize tag to JSON")?;
            println!("{json}");
        }
    }

    Ok(())
}

async fn handle_tag_create(args: TagCreateArgs, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let workspace = args
        .workspace
        .or_else(|| config.default_workspace().map(String::from))
        .ok_or_else(|| anyhow!("workspace is required; provide --workspace or set a default"))?;

    let mut builder = TagCreateBuilder::new(&args.name, workspace);

    if let Some(color) = args.color {
        builder = builder.color(color.into());
    }

    if let Some(notes) = args.notes {
        builder = builder.notes(notes);
    }

    let request = builder
        .build()
        .context("failed to build tag create request")?;

    let tag = api::create_tag(&client, request)
        .await
        .context("failed to create tag")?;

    match args.format {
        TagOutputFormat::Detail | TagOutputFormat::Table => {
            println!("{}", "Tag created successfully:".green().bold());
            render_tag_detail(&tag);
        }
        TagOutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(&tag).context("failed to serialize tag to JSON")?;
            println!("{json}");
        }
    }

    Ok(())
}

async fn handle_tag_update(args: TagUpdateArgs, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    let mut builder = TagUpdateBuilder::new();

    if let Some(name) = args.name {
        builder = builder.name(name);
    }

    if let Some(color) = args.color {
        builder = builder.color(color.into());
    }

    if args.clear_notes {
        builder = builder.clear_notes();
    } else if let Some(notes) = args.notes {
        builder = builder.notes(notes);
    }

    let request = builder
        .build()
        .context("failed to build tag update request")?;

    let tag = api::update_tag(&client, &args.gid, request)
        .await
        .context("failed to update tag")?;

    match args.format {
        TagOutputFormat::Detail | TagOutputFormat::Table => {
            println!("{}", "Tag updated successfully:".green().bold());
            render_tag_detail(&tag);
        }
        TagOutputFormat::Json => {
            let json =
                serde_json::to_string_pretty(&tag).context("failed to serialize tag to JSON")?;
            println!("{json}");
        }
    }

    Ok(())
}

async fn handle_tag_delete(args: TagDeleteArgs, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;

    if !args.yes {
        let tag = api::get_tag(&client, &args.gid)
            .await
            .context("failed to retrieve tag")?;

        println!("{}", "Tag to be deleted:".yellow().bold());
        println!("  GID: {}", tag.gid);
        println!("  Name: {}", tag.name);

        if !confirm_deletion()? {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }

    api::delete_tag(&client, &args.gid)
        .await
        .context("failed to delete tag")?;

    println!("{}", "Tag deleted successfully.".green().bold());

    Ok(())
}

fn render_tag_table(tags: &[Tag]) {
    if tags.is_empty() {
        println!("No tags found.");
        return;
    }

    let is_tty = stdout().is_terminal();

    if is_tty {
        println!(
            "{:<20} {:<30} {:<15} {}",
            "GID".bold(),
            "Name".bold(),
            "Color".bold(),
            "Workspace".bold()
        );
        println!("{}", "─".repeat(80));
    }

    for tag in tags {
        let color_str = tag.color.map_or_else(|| String::from("none"), format_color);

        let workspace_name = tag
            .workspace
            .as_ref()
            .and_then(|ws| ws.name.as_deref())
            .unwrap_or("unknown");

        if is_tty {
            println!(
                "{:<20} {:<30} {:<15} {}",
                tag.gid, tag.name, color_str, workspace_name
            );
        } else {
            println!(
                "{}\t{}\t{}\t{}",
                tag.gid, tag.name, color_str, workspace_name
            );
        }
    }

    if is_tty {
        println!("\n{} tags listed.", tags.len());
    }
}

fn render_tag_detail(tag: &Tag) {
    let is_tty = stdout().is_terminal();

    if is_tty {
        println!("{}", "Tag Details".bold().underline());
        println!("  {}: {}", "GID".bold(), tag.gid);
        println!("  {}: {}", "Name".bold(), tag.name);

        if let Some(color) = &tag.color {
            println!("  {}: {}", "Color".bold(), format_color(*color));
        }

        if let Some(notes) = &tag.notes
            && !notes.is_empty()
        {
            println!("  {}: {}", "Notes".bold(), notes);
        }

        if let Some(workspace) = &tag.workspace {
            println!(
                "  {}: {} ({})",
                "Workspace".bold(),
                workspace.label(),
                workspace.gid
            );
        }

        if !tag.followers.is_empty() {
            println!("  {}:", "Followers".bold());
            for follower in &tag.followers {
                println!("    - {}", follower.label());
            }
        }

        if let Some(created_at) = &tag.created_at {
            println!("  {}: {}", "Created".bold(), created_at);
        }

        if let Some(url) = &tag.permalink_url {
            println!("  {}: {}", "URL".bold(), url);
        }
    } else {
        println!("GID: {}", tag.gid);
        println!("Name: {}", tag.name);

        if let Some(color) = &tag.color {
            println!("Color: {}", format_color(*color));
        }

        if let Some(notes) = &tag.notes
            && !notes.is_empty()
        {
            println!("Notes: {notes}");
        }

        if let Some(workspace) = &tag.workspace {
            println!("Workspace: {} ({})", workspace.label(), workspace.gid);
        }

        if !tag.followers.is_empty() {
            println!("Followers:");
            for follower in &tag.followers {
                println!("  {}", follower.label());
            }
        }

        if let Some(created_at) = &tag.created_at {
            println!("Created: {created_at}");
        }

        if let Some(url) = &tag.permalink_url {
            println!("URL: {url}");
        }
    }
}

fn format_color(color: TagColor) -> String {
    match color {
        TagColor::DarkBlue => "dark-blue".to_string(),
        TagColor::DarkBrown => "dark-brown".to_string(),
        TagColor::DarkGreen => "dark-green".to_string(),
        TagColor::DarkOrange => "dark-orange".to_string(),
        TagColor::DarkPink => "dark-pink".to_string(),
        TagColor::DarkPurple => "dark-purple".to_string(),
        TagColor::DarkRed => "dark-red".to_string(),
        TagColor::DarkTeal => "dark-teal".to_string(),
        TagColor::DarkWarmGray => "dark-warm-gray".to_string(),
        TagColor::LightBlue => "light-blue".to_string(),
        TagColor::LightBrown => "light-brown".to_string(),
        TagColor::LightGreen => "light-green".to_string(),
        TagColor::LightOrange => "light-orange".to_string(),
        TagColor::LightPink => "light-pink".to_string(),
        TagColor::LightPurple => "light-purple".to_string(),
        TagColor::LightRed => "light-red".to_string(),
        TagColor::LightTeal => "light-teal".to_string(),
        TagColor::LightWarmGray => "light-warm-gray".to_string(),
        TagColor::Unknown => "unknown".to_string(),
    }
}

fn confirm_deletion() -> Result<bool> {
    use std::io::{self, Write};

    print!("Are you sure you want to delete this tag? [y/N] ");
    io::stdout().flush().context("failed to flush stdout")?;

    let mut response = String::new();
    io::stdin()
        .read_line(&mut response)
        .context("failed to read user input")?;

    let response = response.trim().to_lowercase();
    Ok(response == "y" || response == "yes")
}
