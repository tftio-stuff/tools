//! Prompter CLI binary.
//!
//! Main entry point for the prompter command-line tool.

use std::env;

use clap::Parser;
use prompter::{
    AppMode, Cli, agent_doc, init_scaffold, parse_args_from, run_list_stdout, run_render_stdout,
    run_tree_stdout, run_validate_stdout,
};
use tftio_cli_common::{
    AgentDocRequest, LicenseType, StandardCommand, ToolSpec, command::run_standard_command,
    detect_agent_doc_request, render_agent_help_yaml, render_agent_skill, workspace_tool,
};

mod doctor;

const TOOL_SPEC: ToolSpec = workspace_tool(
    "prompter",
    "Prompter",
    env!("CARGO_PKG_VERSION"),
    LicenseType::MIT,
    true,
    true,
    false,
);

enum EntryPoint {
    AgentDoc(AgentDocRequest),
    Mode(AppMode),
}

fn parse_args() -> Result<EntryPoint, String> {
    let raw_args = env::args_os().collect::<Vec<_>>();
    if let Some(request) = detect_agent_doc_request(&raw_args) {
        return Ok(EntryPoint::AgentDoc(request));
    }

    let args = raw_args
        .into_iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    parse_args_from(args).map(EntryPoint::Mode)
}

fn print_agent_doc(request: AgentDocRequest) {
    let doc = agent_doc();
    let rendered = match request {
        AgentDocRequest::Help => render_agent_help_yaml(&doc),
        AgentDocRequest::Skill => render_agent_skill(&doc),
    };
    print!("{rendered}");
}

fn main() {
    let mode = match parse_args() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    match mode {
        EntryPoint::AgentDoc(request) => {
            print_agent_doc(request);
        }
        EntryPoint::Mode(mode) => match mode {
        AppMode::Help => {
            Cli::parse_from(["prompter", "--help"]);
        }
        AppMode::Version { json } => {
            let _ = run_standard_command::<Cli, doctor::PrompterDoctor>(
                &TOOL_SPEC,
                &StandardCommand::Version { json },
                Some(&doctor::PrompterDoctor),
            );
        }
        AppMode::License => {
            let _ = run_standard_command::<Cli, doctor::PrompterDoctor>(
                &TOOL_SPEC,
                &StandardCommand::License,
                Some(&doctor::PrompterDoctor),
            );
        }
        AppMode::Completions { shell } => {
            prompter::completions::generate(shell);
        }
        AppMode::Doctor { json } => {
            let exit_code = if json {
                doctor::run_doctor(true)
            } else {
                run_standard_command::<Cli, doctor::PrompterDoctor>(
                    &TOOL_SPEC,
                    &StandardCommand::Doctor,
                    Some(&doctor::PrompterDoctor),
                )
            };
            std::process::exit(exit_code);
        }
        AppMode::Init => {
            if let Err(e) = init_scaffold() {
                eprintln!("Init failed: {e}");
                std::process::exit(1);
            }
        }
        AppMode::List { config, json } => {
            if let Err(e) = run_list_stdout(config.as_deref(), json) {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        AppMode::Tree { config, json } => {
            if let Err(e) = run_tree_stdout(config.as_deref(), json) {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        AppMode::Validate { config, json } => match run_validate_stdout(config.as_deref(), json) {
            Ok(()) => {
                if !json {
                    println!("All profiles valid");
                }
            }
            Err(errs) => {
                if json {
                    eprintln!(r#"{{"error":"{}"}}"#, errs.replace('"', "\\\""));
                } else {
                    eprintln!("Validation errors:\n{errs}");
                }
                std::process::exit(1);
            }
        },
        AppMode::Run {
            profiles,
            separator,
            pre_prompt,
            post_prompt,
            config,
            json,
        } => {
            if let Err(e) = run_render_stdout(
                &profiles,
                separator.as_deref(),
                pre_prompt.as_deref(),
                post_prompt.as_deref(),
                config.as_deref(),
                json,
            ) {
                if json {
                    eprintln!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\""));
                } else {
                    eprintln!("{e}");
                }
                std::process::exit(1);
            }
        }
    },
    }
}
