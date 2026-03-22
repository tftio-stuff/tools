use clap::Parser;
use serde_json::json;
use std::io::Read;
use tftio_cli_common::{
    LicenseType, StandardCommand, ToolSpec, command::run_standard_command_no_doctor,
    error::print_error, render_response, render_response_with, workspace_tool,
};

use todoer::cli::{Cli, Command, MetaCommand, TaskCommand, TaskUpdateCommand};
use todoer::commands::{
    init::run_init,
    list::run_list,
    new::run_new,
    task::{
        NoteResult, ShowResult, StatusResult, UpdateStatusResult, run_note, run_show, run_status,
        run_update_status,
    },
};
use todoer::config::load_config;
use todoer::input::resolve_input;
use todoer::output::render_task_table;
use todoer::project::{
    find_project_file, load_project_name, resolve_init_project, resolve_project,
};

const TOOL_SPEC: ToolSpec = workspace_tool(
    "todoer",
    "Todoer",
    env!("CARGO_PKG_VERSION"),
    LicenseType::CC0,
    true,
    false,
    false,
);

fn main() {
    let cli = Cli::parse();
    let code = run(cli);
    std::process::exit(code);
}

fn run(cli: Cli) -> i32 {
    match cli.command {
        Command::Meta { command } => {
            let standard_command = match command {
                MetaCommand::Version { json } => StandardCommand::Version { json },
                MetaCommand::License => StandardCommand::License,
                MetaCommand::Completions { shell } => StandardCommand::Completions { shell },
            };
            run_standard_command_no_doctor::<Cli>(&TOOL_SPEC, &standard_command)
        }
        Command::Init { project, json } => {
            let config = match load_config() {
                Ok(config) => config,
                Err(error) => return print_error("init", json, &error.to_string()),
            };
            let cwd = match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(error) => return print_error("init", json, &error.to_string()),
            };
            let home = match dirs::home_dir() {
                Some(home) => home,
                None => return print_error("init", json, "no home dir"),
            };
            let git_name = git_repo_name(&cwd);
            let project =
                match resolve_init_project(project.as_deref(), &cwd, &home, git_name.as_deref()) {
                    Ok(project) => project,
                    Err(error) => return print_error("init", json, &error.to_string()),
                };

            match run_init(&config, &project) {
                Ok(result) => {
                    println!(
                        "{}",
                        render_response(
                            "init",
                            json,
                            json!({
                                "project": {"name": project.name, "key": project.key},
                                "db_path": result.db_path,
                                "schema_created": result.schema_created
                            }),
                            format!(
                                "project: {}\ndb: {}",
                                project.name,
                                result.db_path.display()
                            ),
                        )
                    );
                    0
                }
                Err(error) => print_error("init", json, &error.to_string()),
            }
        }
        Command::New {
            project,
            description,
            json,
        } => {
            let config = match load_config() {
                Ok(config) => config,
                Err(error) => return print_error("new", json, &error.to_string()),
            };
            let cwd = match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(error) => return print_error("new", json, &error.to_string()),
            };
            let home = match dirs::home_dir() {
                Some(home) => home,
                None => return print_error("new", json, "no home dir"),
            };
            let discovered = match discover_project_name(&cwd, &home) {
                Ok(discovered) => discovered,
                Err(error) => return print_error("new", json, &error.to_string()),
            };
            let git_name = git_repo_name(&cwd);
            let project = match resolve_project(
                project.as_deref(),
                discovered,
                &cwd,
                &home,
                git_name.as_deref(),
            ) {
                Ok(project) => project,
                Err(error) => return print_error("new", json, &error.to_string()),
            };
            let description = match read_input(&description) {
                Ok(description) => description,
                Err(error) => return print_error("new", json, &error.to_string()),
            };

            match run_new(&config, &project, &description) {
                Ok(result) => {
                    println!(
                        "{}",
                        render_response(
                            "new",
                            json,
                            json!({"task": result.task}),
                            format!(
                                "{} {} {}",
                                result.task.id,
                                result.task.status.as_str(),
                                result.task.description
                            ),
                        )
                    );
                    0
                }
                Err(error) => print_error("new", json, &error.to_string()),
            }
        }
        Command::List { project, all, json } => {
            let config = match load_config() {
                Ok(config) => config,
                Err(error) => return print_error("list", json, &error.to_string()),
            };
            let cwd = match std::env::current_dir() {
                Ok(cwd) => cwd,
                Err(error) => return print_error("list", json, &error.to_string()),
            };
            let home = match dirs::home_dir() {
                Some(home) => home,
                None => return print_error("list", json, "no home dir"),
            };
            let project = if all {
                None
            } else {
                let discovered = match discover_project_name(&cwd, &home) {
                    Ok(discovered) => discovered,
                    Err(error) => return print_error("list", json, &error.to_string()),
                };
                let git_name = git_repo_name(&cwd);
                match resolve_project(
                    project.as_deref(),
                    discovered,
                    &cwd,
                    &home,
                    git_name.as_deref(),
                ) {
                    Ok(project) => Some(project),
                    Err(error) => return print_error("list", json, &error.to_string()),
                }
            };

            match run_list(&config, project.as_ref(), all) {
                Ok(result) => {
                    println!(
                        "{}",
                        render_response(
                            "list",
                            json,
                            json!({"tasks": result.tasks}),
                            render_task_table(&result.tasks),
                        )
                    );
                    0
                }
                Err(error) => print_error("list", json, &error.to_string()),
            }
        }
        Command::Task { command, json } => match command {
            TaskCommand::Status { id } => {
                let config = match load_config() {
                    Ok(config) => config,
                    Err(error) => return print_error("task.status", json, &error.to_string()),
                };
                match run_status(&config, &id) {
                    Ok(result) => {
                        println!("{}", render_status_response(&result, json));
                        0
                    }
                    Err(error) => print_error("task.status", json, &error.to_string()),
                }
            }
            TaskCommand::Show { id } => {
                let config = match load_config() {
                    Ok(config) => config,
                    Err(error) => return print_error("task.show", json, &error.to_string()),
                };
                match run_show(&config, &id) {
                    Ok(result) => {
                        println!("{}", render_show_response(&result, json));
                        0
                    }
                    Err(error) => print_error("task.show", json, &error.to_string()),
                }
            }
            TaskCommand::Note { id, note } => {
                let config = match load_config() {
                    Ok(config) => config,
                    Err(error) => return print_error("task.note", json, &error.to_string()),
                };
                let note = match read_input(&note) {
                    Ok(note) => note,
                    Err(error) => return print_error("task.note", json, &error.to_string()),
                };
                match run_note(&config, &id, &note) {
                    Ok(result) => {
                        println!("{}", render_note_response(&result, json));
                        0
                    }
                    Err(error) => print_error("task.note", json, &error.to_string()),
                }
            }
            TaskCommand::Update { command } => match command {
                TaskUpdateCommand::Status { id, status } => {
                    let config = match load_config() {
                        Ok(config) => config,
                        Err(error) => {
                            return print_error("task.update.status", json, &error.to_string());
                        }
                    };
                    match run_update_status(&config, &id, status) {
                        Ok(result) => {
                            println!("{}", render_update_status_response(&result, json));
                            0
                        }
                        Err(error) => print_error("task.update.status", json, &error.to_string()),
                    }
                }
            },
        },
    }
}

fn discover_project_name(
    cwd: &std::path::Path,
    home: &std::path::Path,
) -> anyhow::Result<Option<String>> {
    if let Some(path) = find_project_file(cwd, home)? {
        let name = load_project_name(&path)?;
        return Ok(Some(name));
    }
    Ok(None)
}

fn read_input(arg: &str) -> anyhow::Result<String> {
    if arg == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        let trimmed = buf.trim_end_matches('\n').to_string();
        return resolve_input(arg, Some(trimmed));
    }
    resolve_input(arg, None)
}

fn git_repo_name(cwd: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(cwd)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() {
        return None;
    }
    std::path::Path::new(&path)
        .file_name()
        .and_then(|segment| segment.to_str())
        .map(std::string::ToString::to_string)
}

fn render_status_response(result: &StatusResult, json: bool) -> String {
    render_response_with(
        "task.status",
        json,
        json!({
            "description": result.description,
            "status": result.status,
            "created_at": result.created_at
        }),
        || {
            format!(
                "{}\n{}\n{}",
                result.description,
                result.status.as_str(),
                result.created_at
            )
        },
    )
}

fn render_show_response(result: &ShowResult, json: bool) -> String {
    render_response_with(
        "task.show",
        json,
        json!({
            "description": result.description,
            "status": result.status,
            "created_at": result.created_at,
            "notes": result.notes
        }),
        || {
            let note_lines = result
                .notes
                .iter()
                .map(|note| format!("- {}", note.note))
                .collect::<Vec<_>>()
                .join("\n");
            if note_lines.is_empty() {
                format!(
                    "{}\n{}\n{}",
                    result.description,
                    result.status.as_str(),
                    result.created_at
                )
            } else {
                format!(
                    "{}\n{}\n{}\n{}",
                    result.description,
                    result.status.as_str(),
                    result.created_at,
                    note_lines
                )
            }
        },
    )
}

fn render_note_response(result: &NoteResult, json: bool) -> String {
    render_response_with("task.note", json, json!({"note": result.note}), || {
        format!("note added: {}", result.note.id)
    })
}

fn render_update_status_response(result: &UpdateStatusResult, json: bool) -> String {
    render_response_with(
        "task.update.status",
        json,
        json!({"status": result.status}),
        || result.status.as_str().to_string(),
    )
}

#[cfg(test)]
mod tests {
    use todoer::commands::task::{ShowResult, StatusResult};
    use todoer::models::{Status, TaskNote};

    use super::{render_show_response, render_status_response};

    #[test]
    fn render_status_response_uses_text_payload_for_text_mode() {
        let rendered = render_status_response(
            &StatusResult {
                description: String::from("write tests"),
                status: Status::InProgress,
                created_at: String::from("2026-03-22T10:00:00Z"),
            },
            false,
        );

        assert_eq!(rendered, "write tests\nIN-PROGRESS\n2026-03-22T10:00:00Z");
    }

    #[test]
    fn render_show_response_wraps_json_output() {
        let rendered = render_show_response(
            &ShowResult {
                description: String::from("write tests"),
                status: Status::Completed,
                created_at: String::from("2026-03-22T10:00:00Z"),
                notes: vec![TaskNote {
                    id: 7,
                    task_id: String::from("task-1"),
                    created_at: String::from("2026-03-22T11:00:00Z"),
                    note: String::from("done"),
                }],
            },
            true,
        );

        assert!(rendered.contains("\"ok\":true"));
        assert!(rendered.contains("\"command\":\"task.show\""));
        assert!(rendered.contains("\"description\":\"write tests\""));
    }
}
