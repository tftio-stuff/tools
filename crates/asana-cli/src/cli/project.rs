//! Project CLI command implementations.

use super::build_api_client;
use crate::{
    api::{self, ApiClient},
    config::Config,
    error::Result,
    filters,
    models::{
        MemberPermission, Project, ProjectCreateData, ProjectCreateRequest, ProjectFilter,
        ProjectListParams, ProjectUpdateData, ProjectUpdateRequest,
    },
    output::{
        ProjectOutputFormat,
        project::{render_project_detail, render_project_list, render_project_members},
    },
    templates,
};
use anyhow::{Context, anyhow, bail};
use clap::{Args, Subcommand};
use dialoguer::{Confirm, Input};
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::{IsTerminal, stdout};
use tokio::runtime::Builder as RuntimeBuilder;
use tracing::warn;

#[derive(Subcommand, Debug)]
pub enum ProjectCommand {
    /// List projects with optional filtering.
    List(ProjectListArgs),
    /// Display detailed information about a project.
    Show(ProjectShowArgs),
    /// Create a new project.
    Create(ProjectCreateArgs),
    /// Update an existing project.
    Update(ProjectUpdateArgs),
    /// Delete a project.
    Delete(ProjectDeleteArgs),
    /// Manage project members.
    Members {
        #[command(subcommand)]
        command: ProjectMembersCommand,
    },
}

#[derive(Args, Debug)]
pub struct ProjectListArgs {
    /// Workspace identifier to filter by.
    #[arg(long)]
    pub workspace: Option<String>,
    /// Team identifier to filter by.
    #[arg(long)]
    pub team: Option<String>,
    /// Filter projects by archived flag.
    #[arg(long)]
    pub archived: Option<bool>,
    /// Sort field (`name`, `created_at`, `modified_at`).
    #[arg(long)]
    pub sort: Option<String>,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<ProjectOutputFormat>,
    /// Inline filter expressions (field=value, field!=value, field~regex, field:substring).
    #[arg(long = "filter", value_name = "EXPR")]
    pub filters: Vec<String>,
    /// Include filters saved to disk.
    #[arg(long = "filter-saved", value_name = "NAME")]
    pub filter_saved: Vec<String>,
    /// Persist the provided filter expressions for reuse.
    #[arg(long = "save-filter", value_name = "NAME")]
    pub save_filter: Option<String>,
    /// Maximum number of projects to retrieve.
    #[arg(long)]
    pub limit: Option<usize>,
    /// Additional fields to request from the API.
    #[arg(long, value_name = "FIELD")]
    pub fields: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ProjectShowArgs {
    /// Project identifier (gid) or name when --by-name is supplied.
    #[arg(value_name = "PROJECT")]
    pub project: String,
    /// Treat the project argument as a name instead of gid.
    #[arg(long)]
    pub by_name: bool,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<ProjectOutputFormat>,
    /// Additional fields to request from the API.
    #[arg(long, value_name = "FIELD")]
    pub fields: Vec<String>,
    /// Include members in the output.
    #[arg(long)]
    pub include_members: bool,
    /// Number of recent status updates to show (0 to disable).
    #[arg(long = "status-limit", default_value_t = 3)]
    pub status_limit: usize,
}

#[derive(Args, Debug)]
pub struct ProjectCreateArgs {
    /// Project name (required unless --interactive or template supplies it).
    #[arg(long)]
    pub name: Option<String>,
    /// Workspace identifier.
    #[arg(long)]
    pub workspace: Option<String>,
    /// Team identifier.
    #[arg(long)]
    pub team: Option<String>,
    /// Project notes/description.
    #[arg(long)]
    pub notes: Option<String>,
    /// Project color slug.
    #[arg(long)]
    pub color: Option<String>,
    /// Start date (YYYY-MM-DD).
    #[arg(long = "start-on")]
    pub start_on: Option<String>,
    /// Due date (YYYY-MM-DD).
    #[arg(long = "due-on")]
    pub due_on: Option<String>,
    /// Owner identifier (gid or email).
    #[arg(long)]
    pub owner: Option<String>,
    /// Visibility flag.
    #[arg(long)]
    pub public: Option<bool>,
    /// Template name or path.
    #[arg(long)]
    pub template: Option<String>,
    /// Additional members to add (gid or email).
    #[arg(long = "member", value_name = "USER")]
    pub members: Vec<String>,
    /// Custom field assignments in KEY=VALUE form.
    #[arg(long = "custom-field", value_name = "KEY=VALUE")]
    pub custom_fields: Vec<String>,
    /// Template variables in KEY=VALUE form.
    #[arg(long = "var", value_name = "KEY=VALUE")]
    pub vars: Vec<String>,
    /// Prompt for missing values interactively.
    #[arg(long)]
    pub interactive: bool,
    /// Output format override.
    #[arg(long, value_enum)]
    pub output: Option<ProjectOutputFormat>,
}

#[derive(Args, Debug)]
pub struct ProjectUpdateArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub notes: Option<String>,
    #[arg(long)]
    pub color: Option<String>,
    #[arg(long = "start-on")]
    pub start_on: Option<String>,
    #[arg(long = "due-on")]
    pub due_on: Option<String>,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long, conflicts_with = "unarchive")]
    pub archive: bool,
    #[arg(long)]
    pub unarchive: bool,
    #[arg(long)]
    pub public: Option<bool>,
    #[arg(long, value_enum)]
    pub output: Option<ProjectOutputFormat>,
}

#[derive(Args, Debug)]
pub struct ProjectDeleteArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    /// Skip confirmation prompts.
    #[arg(long)]
    pub force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProjectTarget {
    /// Project identifier (gid) or name when --by-name is provided.
    #[arg(value_name = "PROJECT")]
    pub project: String,
    /// Treat the project argument as a name.
    #[arg(long)]
    pub by_name: bool,
}

#[derive(Subcommand, Debug)]
pub enum ProjectMembersCommand {
    /// List project members.
    List(ProjectMembersListArgs),
    /// Add members to the project.
    Add(ProjectMembersAddArgs),
    /// Remove members from the project.
    Remove(ProjectMembersRemoveArgs),
    /// Update an existing member's role.
    Update(ProjectMembersUpdateArgs),
}

#[derive(Args, Debug)]
pub struct ProjectMembersListArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    #[arg(long, value_enum)]
    pub output: Option<ProjectOutputFormat>,
}

#[derive(Args, Debug)]
pub struct ProjectMembersAddArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    #[arg(required = true, value_name = "USER")]
    pub members: Vec<String>,
    #[arg(long, value_enum)]
    pub role: Option<MemberPermission>,
}

#[derive(Args, Debug)]
pub struct ProjectMembersRemoveArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    #[arg(required = true, value_name = "USER")]
    pub members: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ProjectMembersUpdateArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    #[arg(long)]
    pub membership: Option<String>,
    #[arg(long)]
    pub member: Option<String>,
    #[arg(long, value_enum)]
    pub role: MemberPermission,
}

pub fn handle_project_command(command: ProjectCommand, config: &Config) -> Result<()> {
    let client = build_api_client(config)?;
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialise async runtime")?;

    runtime.block_on(async {
        match command {
            ProjectCommand::List(args) => list_projects_command(&client, config, args).await?,
            ProjectCommand::Show(args) => show_project_command(&client, config, args).await?,
            ProjectCommand::Create(args) => create_project_command(&client, config, args).await?,
            ProjectCommand::Update(args) => update_project_command(&client, config, args).await?,
            ProjectCommand::Delete(args) => delete_project_command(&client, config, args).await?,
            ProjectCommand::Members { command } => match command {
                ProjectMembersCommand::List(args) => {
                    project_members_list(&client, config, args).await?;
                }
                ProjectMembersCommand::Add(args) => {
                    project_members_add(&client, config, args).await?;
                }
                ProjectMembersCommand::Remove(args) => {
                    project_members_remove(&client, config, args).await?;
                }
                ProjectMembersCommand::Update(args) => {
                    project_member_update(&client, config, args).await?;
                }
            },
        }
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

async fn list_projects_command(
    client: &ApiClient,
    config: &Config,
    args: ProjectListArgs,
) -> Result<()> {
    let mut params = ProjectListParams {
        workspace: args.workspace,
        team: args.team,
        archived: args.archived,
        limit: args.limit,
        ..ProjectListParams::default()
    };
    params.sort = filters::parse_sort(args.sort.as_deref())?;
    if !args.fields.is_empty() {
        params.fields.extend(args.fields.into_iter());
    }

    let mut all_filters = filters::parse_filters(&args.filters)?;
    for name in &args.filter_saved {
        let mut saved = filters::load_saved_filters(config, name)?;
        all_filters.append(&mut saved);
    }
    params.filters = all_filters;

    if let Some(name) = args.save_filter.as_ref() {
        filters::save_filters(config, name, &args.filters)?;
        println!(
            "Saved filter '{name}' with {} expression(s).",
            args.filters.len()
        );
    }

    let projects = api::list_projects(client, params).await?;
    let format = determine_output(args.output);
    let rendered = render_project_list(&projects, format, stdout().is_terminal())?;
    println!("{rendered}");
    Ok(())
}

async fn show_project_command(
    client: &ApiClient,
    _config: &Config,
    args: ProjectShowArgs,
) -> Result<()> {
    let format = determine_output(args.output);
    let fields = if args.fields.is_empty() {
        vec![
            "gid".to_string(),
            "name".to_string(),
            "notes".to_string(),
            "color".to_string(),
            "archived".to_string(),
            "public".to_string(),
            "workspace.name".to_string(),
            "workspace.gid".to_string(),
            "team.name".to_string(),
            "owner.name".to_string(),
            "owner.gid".to_string(),
            "due_on".to_string(),
            "start_on".to_string(),
            "created_at".to_string(),
            "modified_at".to_string(),
        ]
    } else {
        args.fields.clone()
    };

    let mut project = if args.by_name {
        let located = find_project_by_name(client, &args.project).await?;
        api::get_project(client, &located.gid, fields.clone()).await?
    } else {
        api::get_project(client, &args.project, fields.clone()).await?
    };

    if args.include_members
        && let Ok(members) = api::list_members(client, &project.gid).await
    {
        project.members = members.members;
    }

    if args.status_limit > 0 {
        match api::list_statuses(client, &project.gid, Some(args.status_limit)).await {
            Ok(statuses) => project.statuses = statuses,
            Err(err) => warn!("failed to fetch project statuses: {err}"),
        }
    }

    let rendered = render_project_detail(&project, format, stdout().is_terminal())?;
    println!("{rendered}");
    Ok(())
}

async fn create_project_command(
    client: &ApiClient,
    config: &Config,
    args: ProjectCreateArgs,
) -> Result<()> {
    let mut data = if let Some(identifier) = args.template.as_deref() {
        let template = templates::resolve_project_template(config, identifier)?;
        println!("Using template '{}'.", template.name);
        template.project.clone()
    } else {
        ProjectCreateData::default()
    };

    if let Some(name) = args.name {
        data.name = name;
    }
    if let Some(workspace) = args.workspace {
        data.workspace = Some(workspace);
    }
    if let Some(team) = args.team {
        data.team = Some(team);
    }
    if let Some(notes) = args.notes {
        data.notes = Some(notes);
    }
    if let Some(color) = args.color {
        data.color = Some(color);
    }
    if let Some(start_on) = args.start_on {
        data.start_on = Some(start_on);
    }
    if let Some(due_on) = args.due_on {
        data.due_on = Some(due_on);
    }
    if let Some(owner) = args.owner {
        data.owner = Some(owner);
    }
    if let Some(public) = args.public {
        data.public = Some(public);
    }

    if !args.members.is_empty() {
        data.members.extend(args.members.clone());
    }

    if !args.custom_fields.is_empty() {
        let fields = parse_custom_fields(&args.custom_fields)?;
        data.custom_fields.extend(fields);
    }

    let vars = parse_variables(&args.vars)?;

    if args.interactive {
        ensure_tty()?;
        interactive_populate(&mut data)?;
    }

    data = templates::apply_template_variables(data, &vars);
    validate_create_payload(&data)?;

    let request = ProjectCreateRequest { data };
    let mut project = api::create_project(client, request).await?;
    if let Ok(members) = api::list_members(client, &project.gid).await {
        project.members = members.members;
    }

    let format = determine_output(args.output);
    let rendered = render_project_detail(&project, format, stdout().is_terminal())?;
    println!("{rendered}");
    println!("Project URL: {}", project_url(&project));
    Ok(())
}

async fn update_project_command(
    client: &ApiClient,
    config: &Config,
    args: ProjectUpdateArgs,
) -> Result<()> {
    let project = resolve_project_reference(client, config, &args.target).await?;
    let mut data = ProjectUpdateData::default();

    if let Some(name) = args.name {
        data.name = Some(name);
    }
    if let Some(notes) = args.notes {
        data.notes = Some(notes);
    }
    if let Some(color) = args.color {
        data.color = Some(color);
    }
    if let Some(start_on) = args.start_on {
        data.start_on = Some(start_on);
    }
    if let Some(due_on) = args.due_on {
        data.due_on = Some(due_on);
    }
    if let Some(owner) = args.owner {
        data.owner = Some(owner);
    }
    if args.archive {
        data.archived = Some(true);
    }
    if args.unarchive {
        data.archived = Some(false);
    }
    if let Some(public) = args.public {
        data.public = Some(public);
    }

    if data.is_empty() {
        bail!("no updates specified; supply at least one field to change");
    }

    let mut project =
        api::update_project(client, &project.gid, ProjectUpdateRequest { data }).await?;
    if let Ok(members) = api::list_members(client, &project.gid).await {
        project.members = members.members;
    }

    let format = determine_output(args.output);
    let rendered = render_project_detail(&project, format, stdout().is_terminal())?;
    println!("{rendered}");
    Ok(())
}

async fn delete_project_command(
    client: &ApiClient,
    config: &Config,
    args: ProjectDeleteArgs,
) -> Result<()> {
    let project = resolve_project_reference(client, config, &args.target).await?;

    if !args.force {
        ensure_tty()?;
        let prompt = format!("Delete project '{}' ({})?", project.name, project.gid);
        let proceed = Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()?;
        if !proceed {
            println!("Aborted");
            return Ok(());
        }
    }

    api::delete_project(client, &project.gid).await?;
    println!("Deleted project '{}' ({})", project.name, project.gid);
    Ok(())
}

async fn project_members_list(
    client: &ApiClient,
    config: &Config,
    args: ProjectMembersListArgs,
) -> Result<()> {
    let project = resolve_project_reference(client, config, &args.target).await?;
    let members = api::list_members(client, &project.gid).await?;
    let format = determine_output(args.output);
    let rendered = render_project_members(&members.members, format, stdout().is_terminal())?;
    println!("{rendered}");
    Ok(())
}

async fn project_members_add(
    client: &ApiClient,
    config: &Config,
    args: ProjectMembersAddArgs,
) -> Result<()> {
    let project = resolve_project_reference(client, config, &args.target).await?;
    api::add_members(client, &project.gid, args.members.clone(), args.role).await?;
    println!(
        "Added {} member(s) to '{}'.",
        args.members.len(),
        project.name
    );
    Ok(())
}

async fn project_members_remove(
    client: &ApiClient,
    config: &Config,
    args: ProjectMembersRemoveArgs,
) -> Result<()> {
    let project = resolve_project_reference(client, config, &args.target).await?;
    api::remove_members(client, &project.gid, args.members.clone()).await?;
    println!(
        "Removed {} member(s) from '{}'.",
        args.members.len(),
        project.name
    );
    Ok(())
}

async fn project_member_update(
    client: &ApiClient,
    config: &Config,
    args: ProjectMembersUpdateArgs,
) -> Result<()> {
    let project = resolve_project_reference(client, config, &args.target).await?;
    let membership_gid = if let Some(membership) = args.membership.clone() {
        membership
    } else if let Some(member) = args.member.clone() {
        find_membership_by_member(client, &project.gid, &member).await?
    } else {
        bail!("provide either --membership or --member to identify the member to update");
    };

    let updated = api::update_member(client, &membership_gid, args.role).await?;
    println!(
        "Updated member {} to role {:?}.",
        updated.user.label(),
        updated.role.unwrap_or(MemberPermission::Member)
    );
    Ok(())
}

fn determine_output(value: Option<ProjectOutputFormat>) -> ProjectOutputFormat {
    value.unwrap_or_else(|| {
        if stdout().is_terminal() {
            ProjectOutputFormat::Table
        } else {
            ProjectOutputFormat::Json
        }
    })
}

fn ensure_tty() -> Result<()> {
    if !stdout().is_terminal() {
        bail!("interactive mode requires an interactive terminal");
    }
    Ok(())
}

fn interactive_populate(data: &mut ProjectCreateData) -> Result<()> {
    if data.name.trim().is_empty() {
        data.name = Input::new().with_prompt("Project name").interact_text()?;
    }
    if data.workspace.is_none() {
        let workspace = Input::new().with_prompt("Workspace gid").interact_text()?;
        data.workspace = Some(workspace);
    }
    if data.notes.as_deref().is_none_or(str::is_empty) {
        let notes: String = Input::new()
            .with_prompt("Notes (optional)")
            .allow_empty(true)
            .interact_text()?;
        if !notes.is_empty() {
            data.notes = Some(notes);
        }
    }
    Ok(())
}

fn parse_custom_fields(entries: &[String]) -> Result<BTreeMap<String, Value>> {
    let mut map = BTreeMap::new();
    for entry in entries {
        let (key, value) = entry
            .split_once('=')
            .ok_or_else(|| anyhow!("invalid custom field '{entry}'; expected KEY=VALUE"))?;
        let parsed = serde_json::from_str::<Value>(value)
            .unwrap_or_else(|_| Value::String(value.to_string()));
        map.insert(key.trim().to_string(), parsed);
    }
    Ok(map)
}

fn parse_variables(entries: &[String]) -> Result<BTreeMap<String, String>> {
    let mut vars = BTreeMap::new();
    for entry in entries {
        let (key, value) = entry
            .split_once('=')
            .ok_or_else(|| anyhow!("invalid var '{entry}'; expected KEY=VALUE"))?;
        vars.insert(key.trim().to_string(), value.trim().to_string());
    }
    Ok(vars)
}

fn validate_create_payload(data: &ProjectCreateData) -> Result<()> {
    if data.name.trim().is_empty() {
        bail!("project name is required after applying template variables");
    }
    if data.name.contains("{{") {
        bail!("project name still contains unresolved template variables");
    }
    if data
        .workspace
        .as_deref()
        .is_none_or(|value| value.trim().is_empty() || value.contains("{{"))
    {
        bail!("workspace is required after applying template variables");
    }
    Ok(())
}

async fn resolve_project_reference(
    client: &ApiClient,
    _config: &Config,
    target: &ProjectTarget,
) -> Result<Project> {
    if target.by_name {
        find_project_by_name(client, &target.project).await
    } else {
        let fields = vec![
            "gid".to_string(),
            "name".to_string(),
            "workspace.gid".to_string(),
        ];
        api::get_project(client, &target.project, fields)
            .await
            .with_context(|| format!("failed to fetch project {}", target.project))
    }
}

async fn find_project_by_name(client: &ApiClient, name: &str) -> Result<Project> {
    let params = ProjectListParams {
        limit: Some(1),
        filters: vec![ProjectFilter::Equals("name".into(), name.into())],
        ..ProjectListParams::default()
    };
    let mut results = api::list_projects(client, params).await?;
    results
        .pop()
        .ok_or_else(|| anyhow!("project named '{name}' not found"))
}

async fn find_membership_by_member(
    client: &ApiClient,
    project_gid: &str,
    identifier: &str,
) -> Result<String> {
    let members = api::list_members(client, project_gid).await?;
    for member in members.members {
        if member.gid == identifier
            || member.user.gid == identifier
            || member
                .user
                .email
                .as_deref()
                .is_some_and(|email| email.eq_ignore_ascii_case(identifier))
            || member.user.label().eq_ignore_ascii_case(identifier)
        {
            return Ok(member.gid);
        }
    }
    bail!(
        "member '{identifier}' not found in project {project_gid}; specify --membership with the membership gid"
    )
}

fn project_url(project: &Project) -> String {
    project.workspace.as_ref().map_or_else(
        || format!("https://app.asana.com/0/{}/", project.gid),
        |workspace| format!("https://app.asana.com/0/{}/{}/", workspace.gid, project.gid),
    )
}
