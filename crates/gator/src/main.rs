//! Gator CLI entrypoint.

use clap::Parser;
use gator::cli::{Cli, Command, MetaCommand};
use tftio_cli_common::{
    LicenseType, StandardCommand, ToolSpec, command::run_standard_command_no_doctor,
    error::print_error, workspace_tool,
};

const TOOL_SPEC: ToolSpec = workspace_tool(
    "gator",
    "Gator",
    env!("CARGO_PKG_VERSION"),
    LicenseType::MIT,
    true,
    false,
    false,
);

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    if let Some(Command::Meta { command }) = &cli.command {
        let standard_command = match command {
            MetaCommand::Version => StandardCommand::Version { json },
            MetaCommand::License => StandardCommand::License,
            MetaCommand::Completions { shell } => StandardCommand::Completions { shell: *shell },
        };
        std::process::exit(run_standard_command_no_doctor::<Cli>(
            &TOOL_SPEC,
            &standard_command,
        ));
    }

    if let Err(e) = cli.validate() {
        let _ = print_error("error", json, &e);
        std::process::exit(1);
    }

    if let Err(e) = gator::run(&cli) {
        let _ = print_error("error", json, &e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_command_extracts_meta_subcommand() {
        let command = Command::Meta {
            command: MetaCommand::Version,
        };

        assert!(matches!(
            metadata_command(Some(&command)),
            Some(MetaCommand::Version)
        ));
    }

    #[test]
    fn run_returns_fatal_error_for_invalid_cli() {
        let cli = Cli::parse_from(["gator", "claude", "--session=abc", "--workdir=/tmp"]);
        let error = run(cli).expect_err("invalid cli should produce a fatal error");

        assert_eq!(error.command(), "error");
        assert_eq!(error.message(), "--session is incompatible with: --workdir");
    }
}
