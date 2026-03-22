//! Gator CLI entrypoint.

use clap::Parser;
use gator::cli::{Cli, Command, MetaCommand};
use tftio_cli_common::{
    FatalCliError, LicenseType, StandardCommand, StandardCommandMap, ToolSpec,
    command::maybe_run_standard_command_no_doctor, error::fatal_error, parse_and_exit,
    workspace_tool,
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
    parse_and_exit(Cli::parse, |cli| run(&cli));
}

fn run(cli: &Cli) -> Result<i32, FatalCliError> {
    let json = cli.json;
    let meta_command = metadata_command(cli.command.as_ref());

    if let Some(exit_code) =
        maybe_run_standard_command_no_doctor::<Cli, _>(&TOOL_SPEC, meta_command.as_ref(), json)
    {
        return Ok(exit_code);
    }

    cli.validate()
        .map_err(|error| fatal_error("error", json, error))?;
    gator::run(cli).map_err(|error| fatal_error("error", json, error))?;
    Ok(0)
}

#[derive(Clone, Copy)]
struct GatorMetaCommand<'a>(&'a MetaCommand);

impl StandardCommandMap for GatorMetaCommand<'_> {
    fn to_standard_command(&self, json: bool) -> StandardCommand {
        match self.0 {
            MetaCommand::Version => StandardCommand::Version { json },
            MetaCommand::License => StandardCommand::License,
            MetaCommand::Completions { shell } => StandardCommand::Completions { shell: *shell },
        }
    }
}

fn metadata_command(command: Option<&Command>) -> Option<GatorMetaCommand<'_>> {
    command.map(|Command::Meta { command }| GatorMetaCommand(command))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_command_extracts_meta_subcommand() {
        let command = Command::Meta {
            command: MetaCommand::Version,
        };
        let metadata = metadata_command(Some(&command)).expect("meta command should extract");

        assert!(matches!(metadata.0, MetaCommand::Version));
    }

    #[test]
    fn run_returns_fatal_error_for_invalid_cli() {
        let cli = Cli::parse_from(["gator", "claude", "--session=abc", "--workdir=/tmp"]);
        let error = run(&cli).expect_err("invalid cli should produce a fatal error");

        assert_eq!(error.command(), "error");
        assert_eq!(error.message(), "--session is incompatible with: --workdir");
    }
}
