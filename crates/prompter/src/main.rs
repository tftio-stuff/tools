//! Prompter CLI binary.
//!
//! Main entry point for the prompter command-line tool.

use prompter::{
    AppMode, Cli, init_scaffold, resolve_app_mode, run_list_stdout, run_render_stdout,
    run_tree_stdout, run_validate_stdout,
};
use tftio_cli_common::{
    AgentCapability, AgentDispatch, AgentSurfaceSpec, CommandSelector, LicenseType,
    StandardCommand, ToolSpec, command::run_standard_command, parse_with_agent_surface_from,
    workspace_tool,
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
)
.with_agent_surface(&AGENT_SURFACE);

const RENDER_PROMPTS_COMMAND: CommandSelector = CommandSelector::new(&["run"]);
const LIST_PROFILES_COMMAND: CommandSelector = CommandSelector::new(&["list"]);
const TREE_PROFILES_COMMAND: CommandSelector = CommandSelector::new(&["tree"]);
const VALIDATE_PROFILES_COMMAND: CommandSelector = CommandSelector::new(&["validate"]);

const RENDER_PROMPTS_CAPABILITY: AgentCapability =
    AgentCapability::minimal("render-prompts", &[RENDER_PROMPTS_COMMAND], &[]);
const LIST_PROFILES_CAPABILITY: AgentCapability =
    AgentCapability::minimal("list-profiles", &[LIST_PROFILES_COMMAND], &[]);
const TREE_PROFILES_CAPABILITY: AgentCapability =
    AgentCapability::minimal("tree-profiles", &[TREE_PROFILES_COMMAND], &[]);
const VALIDATE_PROFILES_CAPABILITY: AgentCapability =
    AgentCapability::minimal("validate-profiles", &[VALIDATE_PROFILES_COMMAND], &[]);

const AGENT_SURFACE: AgentSurfaceSpec = AgentSurfaceSpec::new(&[
    RENDER_PROMPTS_CAPABILITY,
    LIST_PROFILES_CAPABILITY,
    TREE_PROFILES_CAPABILITY,
    VALIDATE_PROFILES_CAPABILITY,
]);

fn main() {
    let mode = match parse_with_agent_surface_from::<Cli, _>(&TOOL_SPEC, std::env::args_os()) {
        Ok(AgentDispatch::Cli(cli)) => match resolve_app_mode(cli) {
            Ok(mode) => mode,
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(2);
            }
        },
        Ok(AgentDispatch::Printed(code)) => std::process::exit(code),
        Err(error) => error.exit(),
    };

    match mode {
        AppMode::Help => unreachable!("help exits during clap parsing"),
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
    }
}
