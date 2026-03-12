use clap::Parser;
use serde_json::json;
use std::io::Read;

use todoer::cli::{Cli, Command, TaskCommand, TaskUpdateCommand};
use todoer::commands::{
    init::run_init,
    list::run_list,
    new::run_new,
    task::{run_note, run_show, run_status, run_update_status},
};
use todoer::config::load_config;
use todoer::input::resolve_input;
use todoer::output::{err_response, ok_response, render_task_table};
use todoer::project::{
    find_project_file, load_project_name, resolve_init_project, resolve_project,
};

fn main() {
    let cli = Cli::parse();
    let code = run(cli);
    std::process::exit(code);
}

fn run(cli: Cli) -> i32 {
    match cli.command {
        Command::Init { project, json } => {
            let config = match load_config() {
                Ok(c) => c,
                Err(e) => return print_error("init", json, e),
            };
            let cwd = match std::env::current_dir() {
                Ok(c) => c,
                Err(e) => return print_error("init", json, e.into()),
            };
            let home = match dirs::home_dir() {
                Some(h) => h,
                None => return print_error("init", json, anyhow::anyhow!("no home dir")),
            };
            let git_name = git_repo_name(&cwd);
            let proj =
                match resolve_init_project(project.as_deref(), &cwd, &home, git_name.as_deref()) {
                    Ok(p) => p,
                    Err(e) => return print_error("init", json, e),
                };
            match run_init(&config, &proj) {
                Ok(result) => {
                    if json {
                        let out = ok_response(
                            "init",
                            json!({
                                "project": {"name": proj.name, "key": proj.key},
                                "db_path": result.db_path,
                                "schema_created": result.schema_created
                            }),
                        );
                        println!("{}", out);
                    } else {
                        println!("project: {}", proj.name);
                        println!("db: {}", result.db_path.display());
                    }
                    0
                }
                Err(e) => print_error("init", json, e),
            }
        }
        Command::New {
            project,
            description,
            json,
        } => {
            let config = match load_config() {
                Ok(c) => c,
                Err(e) => return print_error("new", json, e),
            };
            let cwd = match std::env::current_dir() {
                Ok(c) => c,
                Err(e) => return print_error("new", json, e.into()),
            };
            let home = match dirs::home_dir() {
                Some(h) => h,
                None => return print_error("new", json, anyhow::anyhow!("no home dir")),
            };
            let discovered = match discover_project_name(&cwd, &home) {
                Ok(d) => d,
                Err(e) => return print_error("new", json, e),
            };
            let git_name = git_repo_name(&cwd);
            let proj = match resolve_project(
                project.as_deref(),
                discovered,
                &cwd,
                &home,
                git_name.as_deref(),
            ) {
                Ok(p) => p,
                Err(e) => return print_error("new", json, e),
            };
            let desc = match read_input(&description) {
                Ok(d) => d,
                Err(e) => return print_error("new", json, e),
            };
            match run_new(&config, &proj, &desc) {
                Ok(result) => {
                    if json {
                        let out = ok_response("new", json!({"task": result.task}));
                        println!("{}", out);
                    } else {
                        println!(
                            "{} {} {}",
                            result.task.id,
                            result.task.status.as_str(),
                            result.task.description
                        );
                    }
                    0
                }
                Err(e) => print_error("new", json, e),
            }
        }
        Command::List { project, all, json } => {
            let config = match load_config() {
                Ok(c) => c,
                Err(e) => return print_error("list", json, e),
            };
            let cwd = match std::env::current_dir() {
                Ok(c) => c,
                Err(e) => return print_error("list", json, e.into()),
            };
            let home = match dirs::home_dir() {
                Some(h) => h,
                None => return print_error("list", json, anyhow::anyhow!("no home dir")),
            };
            let proj = if all {
                None
            } else {
                let discovered = match discover_project_name(&cwd, &home) {
                    Ok(d) => d,
                    Err(e) => return print_error("list", json, e),
                };
                let git_name = git_repo_name(&cwd);
                match resolve_project(
                    project.as_deref(),
                    discovered,
                    &cwd,
                    &home,
                    git_name.as_deref(),
                ) {
                    Ok(p) => Some(p),
                    Err(e) => return print_error("list", json, e),
                }
            };
            match run_list(&config, proj.as_ref(), all) {
                Ok(result) => {
                    if json {
                        let out = ok_response("list", json!({"tasks": result.tasks}));
                        println!("{}", out);
                    } else {
                        let table = render_task_table(&result.tasks);
                        print!("{}", table);
                    }
                    0
                }
                Err(e) => print_error("list", json, e),
            }
        }
        Command::Task { command, json } => match command {
            TaskCommand::Status { id } => {
                let config = match load_config() {
                    Ok(c) => c,
                    Err(e) => return print_error("task.status", json, e),
                };
                match run_status(&config, &id) {
                    Ok(result) => {
                        if json {
                            let out = ok_response(
                                "task.status",
                                json!({
                                    "description": result.description,
                                    "status": result.status,
                                    "created_at": result.created_at
                                }),
                            );
                            println!("{}", out);
                        } else {
                            println!(
                                "{}\n{}\n{}",
                                result.description,
                                result.status.as_str(),
                                result.created_at
                            );
                        }
                        0
                    }
                    Err(e) => print_error("task.status", json, e),
                }
            }
            TaskCommand::Show { id } => {
                let config = match load_config() {
                    Ok(c) => c,
                    Err(e) => return print_error("task.show", json, e),
                };
                match run_show(&config, &id) {
                    Ok(result) => {
                        if json {
                            let out = ok_response(
                                "task.show",
                                json!({
                                    "description": result.description,
                                    "status": result.status,
                                    "created_at": result.created_at,
                                    "notes": result.notes
                                }),
                            );
                            println!("{}", out);
                        } else {
                            println!(
                                "{}\n{}\n{}",
                                result.description,
                                result.status.as_str(),
                                result.created_at
                            );
                            for note in result.notes {
                                println!("- {}", note.note);
                            }
                        }
                        0
                    }
                    Err(e) => print_error("task.show", json, e),
                }
            }
            TaskCommand::Note { id, note } => {
                let config = match load_config() {
                    Ok(c) => c,
                    Err(e) => return print_error("task.note", json, e),
                };
                let note = match read_input(&note) {
                    Ok(n) => n,
                    Err(e) => return print_error("task.note", json, e),
                };
                match run_note(&config, &id, &note) {
                    Ok(result) => {
                        if json {
                            let out = ok_response("task.note", json!({"note": result.note}));
                            println!("{}", out);
                        } else {
                            println!("note added: {}", result.note.id);
                        }
                        0
                    }
                    Err(e) => print_error("task.note", json, e),
                }
            }
            TaskCommand::Update { command } => match command {
                TaskUpdateCommand::Status { id, status } => {
                    let config = match load_config() {
                        Ok(c) => c,
                        Err(e) => return print_error("task.update.status", json, e),
                    };
                    match run_update_status(&config, &id, status) {
                        Ok(result) => {
                            if json {
                                let out = ok_response(
                                    "task.update.status",
                                    json!({"status": result.status}),
                                );
                                println!("{}", out);
                            } else {
                                println!("{}", result.status.as_str());
                            }
                            0
                        }
                        Err(e) => print_error("task.update.status", json, e),
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
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

fn print_error(command: &str, json: bool, err: anyhow::Error) -> i32 {
    if json {
        let out = err_response(command, "ERROR", &err.to_string(), json!({}));
        println!("{}", out);
    } else {
        eprintln!("{}", err);
    }
    1
}
