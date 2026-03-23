//! Gator CLI entrypoint.

use gator::cli::{Cli, Command, MetaCommand};
use tftio_cli_common::{
    AgentCapability, AgentDispatch, AgentSurfaceSpec, CommandSelector, FatalCliError,
    FlagSelector, LicenseType, StandardCommand, StandardCommandMap, ToolSpec,
    command::maybe_run_standard_command_no_doctor, error::fatal_error, parse_with_agent_surface,
    run_with_fatal_handler, workspace_tool,
};

const RUN_AGENT_COMMAND: CommandSelector = CommandSelector::new(&[]);
const WORKDIR_FLAG: FlagSelector = FlagSelector::new(&[], "workdir");
const POLICY_FLAG: FlagSelector = FlagSelector::new(&[], "policy");
const NO_PROMPT_FLAG: FlagSelector = FlagSelector::new(&[], "no-prompt");
const DRY_RUN_FLAG: FlagSelector = FlagSelector::new(&[], "dry-run");
const JSON_FLAG: FlagSelector = FlagSelector::new(&[], "json");

const RUN_AGENT_CAPABILITY: AgentCapability = AgentCapability::new(
    "run-agent",
    "Launch an agent inside the gator sandbox",
    &[RUN_AGENT_COMMAND],
    &[WORKDIR_FLAG, POLICY_FLAG, NO_PROMPT_FLAG, DRY_RUN_FLAG, JSON_FLAG],
);

const AGENT_SURFACE: AgentSurfaceSpec = AgentSurfaceSpec::new(&[RUN_AGENT_CAPABILITY]);

const TOOL_SPEC: ToolSpec = workspace_tool(
    "gator",
    "Gator",
    env!("CARGO_PKG_VERSION"),
    LicenseType::MIT,
    true,
    false,
    false,
)
.with_agent_surface(&AGENT_SURFACE);

fn main() {
    match parse_with_agent_surface::<Cli>(&TOOL_SPEC) {
        Ok(AgentDispatch::Cli(cli)) => std::process::exit(run_with_fatal_handler(|| run(&cli))),
        Ok(AgentDispatch::Printed(code)) => std::process::exit(code),
        Err(error) => error.exit(),
    }
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
    use clap::Parser;

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

    #[test]
    fn tool_spec_declares_root_run_agent_surface() {
        let capability = TOOL_SPEC
            .agent_surface
            .expect("agent surface should exist")
            .capabilities
            .first()
            .expect("run-agent capability should exist");

        assert_eq!(capability.name, "run-agent");
        assert_eq!(capability.commands, &[RUN_AGENT_COMMAND]);
        assert_eq!(
            capability.flags,
            &[WORKDIR_FLAG, POLICY_FLAG, NO_PROMPT_FLAG, DRY_RUN_FLAG, JSON_FLAG]
        );
    }
}
