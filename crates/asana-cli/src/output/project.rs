//! Rendering helpers for project operations.

use crate::{
    error::Result,
    models::{Project, ProjectMember},
    output::ProjectOutputFormat,
};
use anyhow::Context;
use csv::WriterBuilder;
use serde::Serialize;
use serde_json::Value;
use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Style, object::Rows},
};

#[derive(Clone, Copy)]
enum TableStyleKind {
    Rounded,
    Plain,
    Markdown,
}

fn apply_style(table: &mut Table, style: TableStyleKind) {
    match style {
        TableStyleKind::Rounded => {
            table.with(Style::rounded());
        }
        TableStyleKind::Plain => {
            table.with(Style::modern());
        }
        TableStyleKind::Markdown => {
            table.with(Style::markdown());
        }
    }
}

/// Render a collection of projects in the requested format.
///
/// # Errors
///
/// Returns an error if JSON serialization or CSV writing fails.
pub fn render_project_list(
    projects: &[Project],
    format: ProjectOutputFormat,
    tty: bool,
) -> Result<String> {
    match format {
        ProjectOutputFormat::Json => Ok(serde_json::to_string_pretty(projects)?),
        ProjectOutputFormat::Csv => render_projects_csv(projects),
        ProjectOutputFormat::Markdown => {
            Ok(render_projects_table(projects, TableStyleKind::Markdown))
        }
        ProjectOutputFormat::Table => {
            let style = if tty {
                TableStyleKind::Rounded
            } else {
                TableStyleKind::Plain
            };
            Ok(render_projects_table(projects, style))
        }
    }
}

fn render_projects_table(projects: &[Project], style: TableStyleKind) -> String {
    let rows: Vec<ProjectRow> = projects.iter().map(ProjectRow::from).collect();
    let mut table = Table::new(rows);
    apply_style(&mut table, style);
    table.with(Modify::new(Rows::first()).with(Alignment::center()));
    table.to_string()
}

fn render_projects_csv(projects: &[Project]) -> Result<String> {
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(vec![]);
    for project in projects {
        wtr.serialize(ProjectRow::from(project))?;
    }
    let bytes = wtr.into_inner().context("failed to finalize CSV writer")?;
    Ok(String::from_utf8(bytes)?)
}

/// Render a single project detail payload.
///
/// # Errors
///
/// Returns an error if JSON serialization or CSV writing fails.
pub fn render_project_detail(
    project: &Project,
    format: ProjectOutputFormat,
    tty: bool,
) -> Result<String> {
    match format {
        ProjectOutputFormat::Json => Ok(serde_json::to_string_pretty(project)?),
        ProjectOutputFormat::Csv => render_detail_csv(project),
        ProjectOutputFormat::Markdown => Ok(render_detail_table(project, TableStyleKind::Markdown)),
        ProjectOutputFormat::Table => {
            let style = if tty {
                TableStyleKind::Rounded
            } else {
                TableStyleKind::Plain
            };
            Ok(render_detail_table(project, style))
        }
    }
}

fn render_detail_table(project: &Project, style: TableStyleKind) -> String {
    let mut rows = Vec::new();
    rows.push(KeyValueRow::new("GID", &project.gid));
    rows.push(KeyValueRow::new("Name", &project.name));
    if let Some(notes) = &project.notes {
        rows.push(KeyValueRow::new("Notes", notes));
    }
    rows.push(KeyValueRow::new("Archived", &project.archived.to_string()));
    if let Some(public) = project.public {
        rows.push(KeyValueRow::new("Public", &public.to_string()));
    }
    if let Some(workspace) = project.workspace.as_ref() {
        rows.push(KeyValueRow::new("Workspace", &workspace.label()));
    }
    if let Some(team) = project.team.as_ref() {
        rows.push(KeyValueRow::new("Team", &team.label()));
    }
    if let Some(owner) = project.owner.as_ref() {
        rows.push(KeyValueRow::new("Owner", &owner.label()));
    }
    if let Some(start_on) = project.start_on.as_ref() {
        rows.push(KeyValueRow::new("Start On", start_on));
    }
    if let Some(due_on) = project.due_on.as_ref() {
        rows.push(KeyValueRow::new("Due On", due_on));
    }
    if let Some(created_at) = project.created_at.as_ref() {
        rows.push(KeyValueRow::new("Created At", created_at));
    }
    if let Some(modified_at) = project.modified_at.as_ref() {
        rows.push(KeyValueRow::new("Modified At", modified_at));
    }
    if !project.members.is_empty() {
        let member_summary = project
            .members
            .iter()
            .map(|member| {
                format!(
                    "{} ({})",
                    member.user.label(),
                    member.role.as_ref().map_or_else(
                        || "member".to_string(),
                        |role| format!("{role:?}").to_ascii_lowercase()
                    )
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(KeyValueRow::new("Members", &member_summary));
    }
    if !project.custom_fields.is_empty() {
        for (key, value) in &project.custom_fields {
            rows.push(KeyValueRow::new(key, &humanize_value(value)));
        }
    }
    if !project.statuses.is_empty() {
        let summary = project
            .statuses
            .iter()
            .map(format_status)
            .collect::<Vec<_>>()
            .join("\n");
        rows.push(KeyValueRow::new("Status Updates", &summary));
    }

    let mut table = Table::new(rows);
    apply_style(&mut table, style);
    table.with(Modify::new(Rows::first()).with(Alignment::center()));
    table.to_string()
}

fn render_detail_csv(project: &Project) -> Result<String> {
    let mut wtr = WriterBuilder::new().has_headers(false).from_writer(vec![]);

    let mut push = |key: &str, value: &str| -> Result<()> {
        wtr.write_record([key, value])?;
        Ok(())
    };

    push("gid", &project.gid)?;
    push("name", &project.name)?;
    push("archived", &project.archived.to_string())?;
    if let Some(public) = project.public {
        push("public", &public.to_string())?;
    }
    if let Some(workspace) = project.workspace.as_ref() {
        push("workspace", &workspace.label())?;
    }
    if let Some(team) = project.team.as_ref() {
        push("team", &team.label())?;
    }
    if let Some(owner) = project.owner.as_ref() {
        push("owner", &owner.label())?;
    }
    if let Some(start_on) = project.start_on.as_ref() {
        push("start_on", start_on)?;
    }
    if let Some(due_on) = project.due_on.as_ref() {
        push("due_on", due_on)?;
    }
    if let Some(created_at) = project.created_at.as_ref() {
        push("created_at", created_at)?;
    }
    if let Some(modified_at) = project.modified_at.as_ref() {
        push("modified_at", modified_at)?;
    }

    if !project.members.is_empty() {
        let summary = project
            .members
            .iter()
            .map(|member| member.user.label())
            .collect::<Vec<_>>()
            .join("; ");
        push("members", &summary)?;
    }

    for (key, value) in &project.custom_fields {
        push(key, &humanize_value(value))?;
    }

    let bytes = wtr.into_inner().context("failed to finalize CSV writer")?;
    Ok(String::from_utf8(bytes)?)
}

fn humanize_value(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(items) => items
            .iter()
            .map(humanize_value)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(map) => map
            .iter()
            .map(|(k, v)| format!("{k}: {}", humanize_value(v)))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn format_status(status: &crate::models::ProjectStatus) -> String {
    let mut header_parts = Vec::new();
    if let Some(title) = status.title.as_ref() {
        header_parts.push(title.clone());
    }
    if let Some(created_at) = status.created_at.as_ref() {
        header_parts.push(created_at.clone());
    }
    let mut descriptor = if header_parts.is_empty() {
        status.gid.clone()
    } else {
        header_parts.join(" — ")
    };

    if let Some(author) = status.created_by.as_ref() {
        use std::fmt::Write;
        write!(descriptor, " [{}]", author.label()).unwrap();
    }

    if let Some(text) = status.text.as_ref() {
        descriptor.push_str(": ");
        descriptor.push_str(text);
    }

    descriptor
}

/// Render project members in the requested format.
///
/// # Errors
///
/// Returns an error if JSON serialization or CSV writing fails.
pub fn render_project_members(
    members: &[ProjectMember],
    format: ProjectOutputFormat,
    tty: bool,
) -> Result<String> {
    match format {
        ProjectOutputFormat::Json => Ok(serde_json::to_string_pretty(members)?),
        ProjectOutputFormat::Csv => render_members_csv(members),
        ProjectOutputFormat::Markdown => {
            Ok(render_members_table(members, TableStyleKind::Markdown))
        }
        ProjectOutputFormat::Table => {
            let style = if tty {
                TableStyleKind::Rounded
            } else {
                TableStyleKind::Plain
            };
            Ok(render_members_table(members, style))
        }
    }
}

fn render_members_table(members: &[ProjectMember], style: TableStyleKind) -> String {
    let rows: Vec<MemberRow> = members.iter().map(MemberRow::from).collect();
    let mut table = Table::new(rows);
    apply_style(&mut table, style);
    table.with(Modify::new(Rows::first()).with(Alignment::center()));
    table.to_string()
}

fn render_members_csv(members: &[ProjectMember]) -> Result<String> {
    let mut wtr = WriterBuilder::new().has_headers(true).from_writer(vec![]);
    for member in members {
        wtr.serialize(MemberRow::from(member))?;
    }
    let bytes = wtr.into_inner().context("failed to finalize CSV writer")?;
    Ok(String::from_utf8(bytes)?)
}

#[derive(Tabled, Serialize)]
struct ProjectRow {
    gid: String,
    name: String,
    workspace: String,
    owner: String,
    status: String,
    due_on: String,
    modified_at: String,
}

impl From<&Project> for ProjectRow {
    fn from(project: &Project) -> Self {
        Self {
            gid: project.gid.clone(),
            name: project.name.clone(),
            workspace: project.workspace.as_ref().map_or_else(
                || "-".into(),
                super::super::models::workspace::WorkspaceReference::label,
            ),
            owner: project.owner.as_ref().map_or_else(
                || "-".into(),
                super::super::models::user::UserReference::label,
            ),
            status: if project.archived {
                "archived".into()
            } else {
                "active".into()
            },
            due_on: project
                .due_on
                .as_ref()
                .map_or_else(|| "-".into(), ToOwned::to_owned),
            modified_at: project
                .modified_at
                .as_ref()
                .map_or_else(|| "-".into(), ToOwned::to_owned),
        }
    }
}

#[derive(Tabled, Serialize)]
struct MemberRow {
    gid: String,
    user: String,
    role: String,
}

impl From<&ProjectMember> for MemberRow {
    fn from(member: &ProjectMember) -> Self {
        Self {
            gid: member.gid.clone(),
            user: member.user.label(),
            role: member.role.as_ref().map_or_else(
                || "member".into(),
                |role| format!("{role:?}").to_ascii_lowercase(),
            ),
        }
    }
}

#[derive(Tabled)]
struct KeyValueRow {
    key: String,
    value: String,
}

impl KeyValueRow {
    fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.to_string(),
            value: value.to_string(),
        }
    }
}
