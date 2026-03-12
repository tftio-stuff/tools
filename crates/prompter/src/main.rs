//! Prompter CLI binary.
//!
//! Main entry point for the prompter command-line tool.

use std::env;

use clap::Parser;
use prompter::{
    AppMode, Cli, init_scaffold, parse_args_from, run_list_stdout, run_render_stdout,
    run_tree_stdout, run_validate_stdout,
};
use tftio_cli_common::LicenseType;

mod doctor;

fn parse_args() -> Result<AppMode, String> {
    let args: Vec<String> = env::args().collect();
    parse_args_from(args)
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
        AppMode::Help => {
            Cli::parse_from(["prompter", "--help"]);
        }
        AppMode::Version { json } => {
            if json {
                println!(r#"{{"version":"{}"}}"#, env!("CARGO_PKG_VERSION"));
            } else {
                println!("prompter {}", env!("CARGO_PKG_VERSION"));
            }
        }
        AppMode::License => {
            println!(
                "{}",
                tftio_cli_common::license::display_license("prompter", LicenseType::MIT)
            );
        }
        AppMode::Completions { shell } => {
            prompter::completions::generate(shell);
        }
        AppMode::Doctor { json } => {
            let exit_code = doctor::run_doctor_with_json(json);
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
