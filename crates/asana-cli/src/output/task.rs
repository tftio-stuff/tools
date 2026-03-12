//! Rendering helpers for task operations.

use crate::{
    error::Result,
    models::{CustomField, Task, UserReference},
    output::TaskOutputFormat,
};
use anyhow::Context;
use csv::WriterBuilder;
use serde::Serialize;
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

fn format_user_with_email(user: &UserReference) -> String {
    match (&user.name, &user.email) {
        (Some(name), Some(email)) if !email.is_empty() => format!("{name} <{email}>"),
        (Some(name), _) => name.clone(),
        (None, Some(email)) if !email.is_empty() => email.clone(),
        _ => user.gid.clone(),
    }
}

/// Render a collection of tasks in the requested format.
///
/// # Errors
///
/// Returns an error if serialization fails.
pub fn render_task_list(tasks: &[Task], format: TaskOutputFormat, tty: bool) -> Result<String> {
    match format {
        TaskOutputFormat::Json => Ok(serde_json::to_string_pretty(tasks)?),
        TaskOutputFormat::Csv => render_task_list_csv(tasks),
        TaskOutputFormat::Markdown => Ok(render_task_list_table(tasks, TableStyleKind::Markdown)),
        TaskOutputFormat::Table => {
            let style = if tty {
                TableStyleKind::Rounded
            } else {
                TableStyleKind::Plain
            };
            Ok(render_task_list_table(tasks, style))
        }
    }
}

fn render_task_list_table(tasks: &[Task], style: TableStyleKind) -> String {
    let rows: Vec<TaskRow> = tasks.iter().map(TaskRow::from).collect();
    let mut table = Table::new(rows);
    apply_style(&mut table, style);
    table.with(Modify::new(Rows::first()).with(Alignment::center()));
    table.to_string()
}

fn render_task_list_csv(tasks: &[Task]) -> Result<String> {
    let mut writer = WriterBuilder::new().has_headers(true).from_writer(vec![]);
    for task in tasks {
        writer.serialize(TaskRow::from(task))?;
    }
    let bytes = writer
        .into_inner()
        .context("failed to finalize CSV writer")?;
    Ok(String::from_utf8(bytes)?)
}

/// Render detailed task information.
///
/// # Errors
///
/// Returns an error if serialization fails.
pub fn render_task_detail(task: &Task, format: TaskOutputFormat, tty: bool) -> Result<String> {
    match format {
        TaskOutputFormat::Json => Ok(serde_json::to_string_pretty(task)?),
        TaskOutputFormat::Csv => render_task_detail_csv(task),
        TaskOutputFormat::Markdown => Ok(render_task_detail_table(task, TableStyleKind::Markdown)),
        TaskOutputFormat::Table => {
            let style = if tty {
                TableStyleKind::Rounded
            } else {
                TableStyleKind::Plain
            };
            Ok(render_task_detail_table(task, style))
        }
    }
}

fn render_task_detail_table(task: &Task, style: TableStyleKind) -> String {
    let mut rows = Vec::new();
    add_basic_task_rows(&mut rows, task);
    add_collection_rows(&mut rows, task);
    add_optional_content_rows(&mut rows, task);
    build_detail_table(rows, style)
}

fn add_basic_task_rows(rows: &mut Vec<KeyValueRow>, task: &Task) {
    rows.push(KeyValueRow::new("GID", &task.gid));
    rows.push(KeyValueRow::new("Name", &task.name));
    rows.push(KeyValueRow::new("Completed", task.completed.to_string()));
    if let Some(completed_at) = task.completed_at.as_ref() {
        rows.push(KeyValueRow::new("Completed At", completed_at));
    }
    if let Some(assignee) = task.assignee.as_ref() {
        rows.push(KeyValueRow::new(
            "Assignee",
            format_user_with_email(assignee),
        ));
    }
    if let Some(workspace) = task.workspace.as_ref() {
        rows.push(KeyValueRow::new("Workspace", workspace.label()));
    }
    if let Some(due_on) = task.due_on.as_ref() {
        rows.push(KeyValueRow::new("Due On", due_on));
    }
    if let Some(due_at) = task.due_at.as_ref() {
        rows.push(KeyValueRow::new("Due At", due_at));
    }
    if let Some(start_on) = task.start_on.as_ref() {
        rows.push(KeyValueRow::new("Start On", start_on));
    }
    if let Some(start_at) = task.start_at.as_ref() {
        rows.push(KeyValueRow::new("Start At", start_at));
    }
    if let Some(parent) = task.parent.as_ref() {
        rows.push(KeyValueRow::new("Parent", parent.label()));
    }
}

fn add_collection_rows(rows: &mut Vec<KeyValueRow>, task: &Task) {
    if !task.projects.is_empty() {
        let summary = task
            .projects
            .iter()
            .map(crate::models::task::TaskProjectReference::label)
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(KeyValueRow::new("Projects", &summary));
    }
    if !task.tags.is_empty() {
        let summary = task
            .tags
            .iter()
            .map(crate::models::task::TaskTagReference::label)
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(KeyValueRow::new("Tags", &summary));
    }
    if !task.followers.is_empty() {
        let summary = task
            .followers
            .iter()
            .map(format_user_with_email)
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(KeyValueRow::new("Followers", summary));
    }
    if !task.dependencies.is_empty() {
        let summary = task
            .dependencies
            .iter()
            .map(crate::models::task::TaskReference::label)
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(KeyValueRow::new("Depends On", &summary));
    }
    if !task.dependents.is_empty() {
        let summary = task
            .dependents
            .iter()
            .map(crate::models::task::TaskReference::label)
            .collect::<Vec<_>>()
            .join(", ");
        rows.push(KeyValueRow::new("Blocks", &summary));
    }
}

fn add_optional_content_rows(rows: &mut Vec<KeyValueRow>, task: &Task) {
    if let Some(permalink) = task.permalink_url.as_ref() {
        rows.push(KeyValueRow::new("Permalink", permalink));
    }
    if let Some(notes) = task.notes.as_ref()
        && !notes.trim().is_empty()
    {
        rows.push(KeyValueRow::new("Notes", notes));
    }
    if let Some(html_notes) = task.html_notes.as_ref()
        && !html_notes.trim().is_empty()
    {
        rows.push(KeyValueRow::new("HTML Notes", html_notes));
    }
    if !task.custom_fields.is_empty() {
        for field in &task.custom_fields {
            rows.push(KeyValueRow::new(&field.name, custom_field_display(field)));
        }
    }
    if !task.attachments.is_empty() {
        let summary = task
            .attachments
            .iter()
            .map(|attachment| {
                format!(
                    "{} ({})",
                    attachment.name,
                    attachment
                        .permanent_url
                        .as_deref()
                        .unwrap_or("no link available")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        rows.push(KeyValueRow::new("Attachments", &summary));
    }
}

fn build_detail_table(rows: Vec<KeyValueRow>, style: TableStyleKind) -> String {
    let mut table = Table::new(rows);
    apply_style(&mut table, style);
    table.with(Modify::new(Rows::first()).with(Alignment::center()));
    table.to_string()
}

fn render_task_detail_csv(task: &Task) -> Result<String> {
    let mut writer = WriterBuilder::new().has_headers(false).from_writer(vec![]);
    let mut push = |key: &str, value: &str| -> Result<()> {
        writer.write_record([key, value])?;
        Ok(())
    };

    push("gid", &task.gid)?;
    push("name", &task.name)?;
    push("completed", &task.completed.to_string())?;
    if let Some(completed_at) = task.completed_at.as_ref() {
        push("completed_at", completed_at)?;
    }
    if let Some(assignee) = task.assignee.as_ref() {
        let formatted = format_user_with_email(assignee);
        push("assignee", &formatted)?;
    }
    if let Some(workspace) = task.workspace.as_ref() {
        push("workspace", &workspace.label())?;
    }
    if let Some(due_on) = task.due_on.as_ref() {
        push("due_on", due_on)?;
    }
    if let Some(due_at) = task.due_at.as_ref() {
        push("due_at", due_at)?;
    }
    if let Some(start_on) = task.start_on.as_ref() {
        push("start_on", start_on)?;
    }
    if let Some(start_at) = task.start_at.as_ref() {
        push("start_at", start_at)?;
    }
    if let Some(parent) = task.parent.as_ref() {
        push("parent", &parent.label())?;
    }
    if !task.projects.is_empty() {
        let summary = task
            .projects
            .iter()
            .map(crate::models::task::TaskProjectReference::label)
            .collect::<Vec<_>>()
            .join(", ");
        push("projects", &summary)?;
    }
    if !task.tags.is_empty() {
        let summary = task
            .tags
            .iter()
            .map(crate::models::task::TaskTagReference::label)
            .collect::<Vec<_>>()
            .join(", ");
        push("tags", &summary)?;
    }
    if !task.followers.is_empty() {
        let summary = task
            .followers
            .iter()
            .map(format_user_with_email)
            .collect::<Vec<_>>()
            .join(", ");
        push("followers", &summary)?;
    }
    if !task.custom_fields.is_empty() {
        for field in &task.custom_fields {
            push(
                &format!("custom_field:{}", field.name),
                &custom_field_display(field),
            )?;
        }
    }
    if !task.dependencies.is_empty() {
        let summary = task
            .dependencies
            .iter()
            .map(crate::models::task::TaskReference::label)
            .collect::<Vec<_>>()
            .join(", ");
        push("depends_on", &summary)?;
    }
    if !task.dependents.is_empty() {
        let summary = task
            .dependents
            .iter()
            .map(crate::models::task::TaskReference::label)
            .collect::<Vec<_>>()
            .join(", ");
        push("blocks", &summary)?;
    }
    let bytes = writer
        .into_inner()
        .context("failed to finalize CSV writer")?;
    Ok(String::from_utf8(bytes)?)
}

fn custom_field_display(field: &CustomField) -> String {
    if let Some(display) = &field.display_value {
        return display.clone();
    }
    if let Some(text) = &field.text_value {
        return text.clone();
    }
    if let Some(number) = field.number_value {
        return number.to_string();
    }
    if let Some(enum_value) = &field.enum_value {
        return enum_value.name.clone();
    }
    if !field.multi_enum_values.is_empty() {
        return field
            .multi_enum_values
            .iter()
            .map(|value| value.name.clone())
            .collect::<Vec<_>>()
            .join(", ");
    }
    if let Some(date) = &field.date_value {
        let mut parts = Vec::new();
        if let Some(start_on) = &date.start_on {
            parts.push(format!("start: {start_on}"));
        }
        if let Some(due_on) = &date.due_on {
            parts.push(format!("due: {due_on}"));
        }
        if let Some(single) = &date.date {
            parts.push(single.clone());
        }
        if !parts.is_empty() {
            return parts.join(" | ");
        }
    }
    if field.extra.is_empty() {
        "n/a".into()
    } else {
        serde_json::to_string(&field.extra).unwrap_or_else(|_| "n/a".into())
    }
}

#[derive(Tabled, Serialize)]
struct TaskRow {
    /// Task identifier.
    #[tabled(rename = "GID")]
    gid: String,
    /// Task name.
    #[tabled(rename = "Name")]
    name: String,
    /// Completion flag.
    #[tabled(rename = "Done")]
    completed: String,
    /// Due date (all day).
    #[tabled(rename = "Due")]
    due_on: String,
    /// Assignee label.
    #[tabled(rename = "Assignee")]
    assignee: String,
    /// Primary project.
    #[tabled(rename = "Project")]
    project: String,
}

impl From<&Task> for TaskRow {
    fn from(task: &Task) -> Self {
        Self {
            gid: task.gid.clone(),
            name: task.name.clone(),
            completed: if task.completed {
                "yes".into()
            } else {
                "no".into()
            },
            due_on: task.due_on.clone().unwrap_or_else(|| "-".into()),
            assignee: task
                .assignee
                .as_ref()
                .map_or_else(|| "-".into(), crate::models::user::UserReference::label),
            project: task.projects.first().map_or_else(
                || "-".into(),
                crate::models::task::TaskProjectReference::label,
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
    fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}
