//! Shared standard CLI commands.

use std::path::PathBuf;

use clap::{CommandFactory, FromArgMatches};
use clap_complete::Shell;

use crate::{
    AgentDispatch, DoctorChecks, ToolSpec, display_license, generate_completions,
    parse_with_agent_surface, parse_with_agent_surface_from, run_doctor, update,
};

/// Shared doctorless adapter for tools that do not expose a doctor command.
pub struct NoDoctor;

impl DoctorChecks for NoDoctor {
    fn repo_info() -> crate::RepoInfo {
        crate::app::WORKSPACE_REPO
    }

    fn current_version() -> &'static str {
        "unknown"
    }
}

/// Shared metadata commands exposed by workspace binaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandardCommand {
    /// Show version information.
    Version {
        /// Whether to emit JSON instead of plain text.
        json: bool,
    },
    /// Show license text.
    License,
    /// Generate shell completion scripts.
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },
    /// Run health checks.
    Doctor,
    /// Run self-update.
    Update {
        /// Optional target version.
        version: Option<String>,
        /// Force the installation.
        force: bool,
        /// Optional install directory.
        install_dir: Option<PathBuf>,
    },
}

/// Map crate-local metadata commands onto the shared [`StandardCommand`] surface.
pub trait StandardCommandMap {
    /// Convert a local metadata command into a shared command.
    fn to_standard_command(&self, json: bool) -> StandardCommand;
}

/// Convert a crate-local metadata command into a shared [`StandardCommand`].
#[must_use]
pub fn map_standard_command<C>(command: &C, json: bool) -> StandardCommand
where
    C: StandardCommandMap + ?Sized,
{
    command.to_standard_command(json)
}

/// Run a mapped standard command when a tool exposes one.
#[must_use]
#[allow(clippy::single_option_map)]
pub fn maybe_run_standard_command<T, D, C>(
    spec: &ToolSpec,
    command: Option<&C>,
    json: bool,
    doctor: Option<&D>,
) -> Option<i32>
where
    T: CommandFactory,
    D: DoctorChecks,
    C: StandardCommandMap + ?Sized,
{
    command.map(|command| {
        run_standard_command::<T, D>(spec, &map_standard_command(command, json), doctor)
    })
}

/// Run a mapped standard command for a tool with no doctor support.
#[must_use]
#[allow(clippy::single_option_map)]
pub fn maybe_run_standard_command_no_doctor<T, C>(
    spec: &ToolSpec,
    command: Option<&C>,
    json: bool,
) -> Option<i32>
where
    T: CommandFactory,
    C: StandardCommandMap + ?Sized,
{
    command.map(|command| {
        run_standard_command_no_doctor::<T>(spec, &map_standard_command(command, json))
    })
}

/// Parse a clap CLI against the current normal or agent-filtered surface.
///
/// # Errors
///
/// Returns a `clap` error when parsing fails.
pub fn parse_command_with_agent_surface<T>(
    spec: &ToolSpec,
) -> Result<AgentDispatch<T>, clap::Error>
where
    T: CommandFactory + FromArgMatches,
{
    parse_with_agent_surface(spec)
}

/// Parse argv against the current normal or agent-filtered surface.
///
/// # Errors
///
/// Returns a `clap` error when parsing fails.
pub fn parse_command_with_agent_surface_from<T, I>(
    spec: &ToolSpec,
    argv: I,
) -> Result<AgentDispatch<T>, clap::Error>
where
    T: CommandFactory + FromArgMatches,
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString> + Clone,
{
    parse_with_agent_surface_from(spec, argv)
}

/// Parse argv and hand the typed CLI to a borrowed callback when parsing succeeds.
///
/// # Errors
///
/// Returns a `clap` error when parsing fails.
pub fn parse_command_ref_with_agent_surface_from<T, I, R, F>(
    spec: &ToolSpec,
    argv: I,
    run: F,
) -> Result<AgentDispatch<R>, clap::Error>
where
    T: CommandFactory + FromArgMatches,
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString> + Clone,
    F: FnOnce(&T) -> R,
{
    match parse_with_agent_surface_from(spec, argv)? {
        AgentDispatch::Cli(cli) => Ok(AgentDispatch::Cli(run(&cli))),
        AgentDispatch::Printed(code) => Ok(AgentDispatch::Printed(code)),
    }
}

fn render_version(spec: &ToolSpec, json: bool) -> String {
    if json {
        format!(r#"{{"version":"{}"}}"#, spec.version)
    } else {
        format!("{} {}", spec.bin_name, spec.version)
    }
}

fn render_license(spec: &ToolSpec) -> String {
    display_license(spec.bin_name, spec.license)
}

/// Execute a shared standard command.
#[must_use]
pub fn run_standard_command<T, D>(
    spec: &ToolSpec,
    command: &StandardCommand,
    doctor: Option<&D>,
) -> i32
where
    T: CommandFactory,
    D: DoctorChecks,
{
    match command {
        StandardCommand::Version { json } => {
            println!("{}", render_version(spec, *json));
            0
        }
        StandardCommand::License => {
            println!("{}", render_license(spec));
            0
        }
        StandardCommand::Completions { shell } => {
            generate_completions::<T>(*shell);
            0
        }
        StandardCommand::Doctor => {
            let Some(tool) = doctor else {
                eprintln!("doctor support not configured");
                return 1;
            };
            run_doctor(tool)
        }
        StandardCommand::Update {
            version,
            force,
            install_dir,
        } => update::run_update(
            &spec.repo,
            spec.version,
            version.as_deref(),
            *force,
            install_dir.as_deref(),
        ),
    }
}

/// Execute a shared standard command for a tool with no doctor support.
#[must_use]
pub fn run_standard_command_no_doctor<T>(spec: &ToolSpec, command: &StandardCommand) -> i32
where
    T: CommandFactory,
{
    run_standard_command::<T, NoDoctor>(spec, command, None)
}

/// Implement [`StandardCommandMap`] for a crate-local metadata enum that uses the
/// workspace-standard variant names.
#[macro_export]
macro_rules! impl_standard_command_map {
    ($type:ty, global_json $(,)?) => {
        impl $crate::command::StandardCommandMap for $type {
            fn to_standard_command(&self, json: bool) -> $crate::StandardCommand {
                match self {
                    Self::Version => $crate::StandardCommand::Version { json },
                    Self::License => $crate::StandardCommand::License,
                    Self::Completions { shell } => {
                        $crate::StandardCommand::Completions { shell: *shell }
                    }
                }
            }
        }
    };
    ($type:ty, global_json, doctor $(,)?) => {
        impl $crate::command::StandardCommandMap for $type {
            fn to_standard_command(&self, json: bool) -> $crate::StandardCommand {
                match self {
                    Self::Version => $crate::StandardCommand::Version { json },
                    Self::License => $crate::StandardCommand::License,
                    Self::Completions { shell } => {
                        $crate::StandardCommand::Completions { shell: *shell }
                    }
                    Self::Doctor => $crate::StandardCommand::Doctor,
                }
            }
        }
    };
    ($type:ty, global_json, doctor, update $(,)?) => {
        impl $crate::command::StandardCommandMap for $type {
            fn to_standard_command(&self, json: bool) -> $crate::StandardCommand {
                match self {
                    Self::Version => $crate::StandardCommand::Version { json },
                    Self::License => $crate::StandardCommand::License,
                    Self::Completions { shell } => {
                        $crate::StandardCommand::Completions { shell: *shell }
                    }
                    Self::Doctor => $crate::StandardCommand::Doctor,
                    Self::Update {
                        version,
                        force,
                        install_dir,
                    } => $crate::StandardCommand::Update {
                        version: version.clone(),
                        force: *force,
                        install_dir: install_dir.clone(),
                    },
                }
            }
        }
    };
    ($type:ty, field_json $(,)?) => {
        impl $crate::command::StandardCommandMap for $type {
            fn to_standard_command(&self, _json: bool) -> $crate::StandardCommand {
                match self {
                    Self::Version { json } => $crate::StandardCommand::Version { json: *json },
                    Self::License => $crate::StandardCommand::License,
                    Self::Completions { shell } => {
                        $crate::StandardCommand::Completions { shell: *shell }
                    }
                }
            }
        }
    };
    ($type:ty, fixed_json = $json:expr $(,)?) => {
        impl $crate::command::StandardCommandMap for $type {
            fn to_standard_command(&self, _json: bool) -> $crate::StandardCommand {
                match self {
                    Self::Version => $crate::StandardCommand::Version { json: $json },
                    Self::License => $crate::StandardCommand::License,
                    Self::Completions { shell } => {
                        $crate::StandardCommand::Completions { shell: *shell }
                    }
                }
            }
        }
    };
    ($type:ty, fixed_json = $json:expr, doctor $(,)?) => {
        impl $crate::command::StandardCommandMap for $type {
            fn to_standard_command(&self, _json: bool) -> $crate::StandardCommand {
                match self {
                    Self::Version => $crate::StandardCommand::Version { json: $json },
                    Self::License => $crate::StandardCommand::License,
                    Self::Completions { shell } => {
                        $crate::StandardCommand::Completions { shell: *shell }
                    }
                    Self::Doctor => $crate::StandardCommand::Doctor,
                }
            }
        }
    };
    ($type:ty, fixed_json = $json:expr, doctor, update $(,)?) => {
        impl $crate::command::StandardCommandMap for $type {
            fn to_standard_command(&self, _json: bool) -> $crate::StandardCommand {
                match self {
                    Self::Version => $crate::StandardCommand::Version { json: $json },
                    Self::License => $crate::StandardCommand::License,
                    Self::Completions { shell } => {
                        $crate::StandardCommand::Completions { shell: *shell }
                    }
                    Self::Doctor => $crate::StandardCommand::Doctor,
                    Self::Update {
                        version,
                        force,
                        install_dir,
                    } => $crate::StandardCommand::Update {
                        version: version.clone(),
                        force: *force,
                        install_dir: install_dir.clone(),
                    },
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use clap::{Parser, Subcommand};

    use super::*;
    use crate::{
        AGENT_TOKEN_ENV, AGENT_TOKEN_EXPECTED_ENV, AgentCapability, AgentDispatch,
        AgentSurfaceSpec, CommandSelector, FlagSelector, LicenseType, RepoInfo, workspace_tool,
    };

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    const QUERY_COMMAND: CommandSelector = CommandSelector::new(&["query"]);
    const QUERY_LIMIT_FLAG: FlagSelector = FlagSelector::new(&["query"], "limit");
    const QUERY_CAPABILITY: AgentCapability = AgentCapability::new(
        "query-posts",
        "Read paginated post records",
        &[QUERY_COMMAND],
        &[QUERY_LIMIT_FLAG],
    );
    const AGENT_SURFACE: AgentSurfaceSpec = AgentSurfaceSpec::new(&[QUERY_CAPABILITY]);

    #[derive(Parser)]
    struct TestCli;

    #[derive(Debug, Parser, PartialEq, Eq)]
    #[command(name = "tool")]
    struct ParseTestCli {
        #[command(subcommand)]
        command: ParseTestCommand,
    }

    #[derive(Debug, Subcommand, PartialEq, Eq)]
    enum ParseTestCommand {
        Query {
            #[arg(long)]
            limit: u32,
        },
        Admin,
    }

    struct TestDoctor;

    impl DoctorChecks for TestDoctor {
        fn repo_info() -> RepoInfo {
            RepoInfo::new("owner", "doctor-tool")
        }

        fn current_version() -> &'static str {
            "1.0.0"
        }
    }

    fn spec() -> ToolSpec {
        ToolSpec::new(
            "tool",
            "Tool",
            "1.2.3",
            LicenseType::MIT,
            RepoInfo::new("owner", "repo"),
            true,
            true,
            true,
        )
    }

    fn agent_spec() -> ToolSpec {
        workspace_tool("tool", "Tool", "1.2.3", LicenseType::MIT, true, true, true)
            .with_agent_surface(&AGENT_SURFACE)
    }

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[allow(unsafe_code)]
    fn set_tokens(presented: Option<&str>, expected: Option<&str>) {
        unsafe {
            std::env::remove_var(AGENT_TOKEN_ENV);
            std::env::remove_var(AGENT_TOKEN_EXPECTED_ENV);
            if let Some(presented) = presented {
                std::env::set_var(AGENT_TOKEN_ENV, presented);
            }
            if let Some(expected) = expected {
                std::env::set_var(AGENT_TOKEN_EXPECTED_ENV, expected);
            }
        }
    }

    #[test]
    fn version_json_contains_version_key() {
        let rendered = render_version(&spec(), true);
        assert!(rendered.contains("\"version\""));
    }

    #[test]
    fn license_render_uses_display_license_text() {
        let rendered = render_license(&spec());
        assert!(rendered.contains("MIT License"));
    }

    #[test]
    fn run_standard_command_version_returns_success() {
        let exit_code = run_standard_command::<TestCli, TestDoctor>(
            &spec(),
            &StandardCommand::Version { json: false },
            Some(&TestDoctor),
        );
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn run_standard_command_no_doctor_version_returns_success() {
        let exit_code = run_standard_command_no_doctor::<TestCli>(
            &spec(),
            &StandardCommand::Version { json: true },
        );
        assert_eq!(exit_code, 0);
    }

    #[allow(dead_code)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum GlobalJsonMetaCommand {
        Version,
        License,
        Completions { shell: Shell },
    }

    impl_standard_command_map!(GlobalJsonMetaCommand, global_json);

    #[allow(dead_code)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum FixedJsonMetaCommand {
        Version,
        License,
        Completions { shell: Shell },
        Doctor,
    }

    impl_standard_command_map!(FixedJsonMetaCommand, fixed_json = false, doctor);

    #[allow(dead_code)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum VersionFieldMetaCommand {
        Version { json: bool },
        License,
        Completions { shell: Shell },
    }

    impl_standard_command_map!(VersionFieldMetaCommand, field_json);

    #[test]
    fn impl_standard_command_map_uses_global_json_flag() {
        let command = map_standard_command(&GlobalJsonMetaCommand::Version, true);
        assert_eq!(command, StandardCommand::Version { json: true });
    }

    #[test]
    fn impl_standard_command_map_supports_fixed_json_and_doctor_variants() {
        let command = map_standard_command(&FixedJsonMetaCommand::Doctor, true);
        assert_eq!(command, StandardCommand::Doctor);
    }

    #[allow(dead_code)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum UpdateMetaCommand {
        Version,
        License,
        Completions {
            shell: Shell,
        },
        Doctor,
        Update {
            version: Option<String>,
            force: bool,
            install_dir: Option<PathBuf>,
        },
    }

    impl_standard_command_map!(UpdateMetaCommand, fixed_json = false, doctor, update);

    #[test]
    fn impl_standard_command_map_reads_json_from_version_field() {
        let command = map_standard_command(&VersionFieldMetaCommand::Version { json: true }, false);
        assert_eq!(command, StandardCommand::Version { json: true });
    }

    #[test]
    fn impl_standard_command_map_clones_update_payload() {
        let command = map_standard_command(
            &UpdateMetaCommand::Update {
                version: Some(String::from("1.0.0")),
                force: true,
                install_dir: Some(PathBuf::from("/tmp/install")),
            },
            false,
        );
        assert_eq!(
            command,
            StandardCommand::Update {
                version: Some(String::from("1.0.0")),
                force: true,
                install_dir: Some(PathBuf::from("/tmp/install")),
            }
        );
    }

    #[test]
    fn maybe_run_standard_command_no_doctor_executes_mapped_metadata_command() {
        let exit_code = maybe_run_standard_command_no_doctor::<TestCli, _>(
            &spec(),
            Some(&GlobalJsonMetaCommand::License),
            false,
        );
        assert_eq!(exit_code, Some(0));
    }

    #[test]
    fn maybe_run_standard_command_returns_none_without_metadata_command() {
        let exit_code = maybe_run_standard_command_no_doctor::<TestCli, GlobalJsonMetaCommand>(
            &spec(),
            None,
            false,
        );
        assert_eq!(exit_code, None);
    }

    #[test]
    fn parse_command_with_agent_surface_from_returns_owned_cli() {
        let _guard = env_lock();
        set_tokens(None, None);

        let parsed = parse_command_with_agent_surface_from::<ParseTestCli, _>(
            &agent_spec(),
            ["tool", "query", "--limit", "5"],
        )
        .expect("parse should succeed");

        assert_eq!(
            parsed,
            AgentDispatch::Cli(ParseTestCli {
                command: ParseTestCommand::Query { limit: 5 },
            })
        );
    }

    #[test]
    fn parse_command_ref_with_agent_surface_from_borrows_cli() {
        let _guard = env_lock();
        set_tokens(Some("shared-token"), Some("shared-token"));

        let parsed = parse_command_ref_with_agent_surface_from::<ParseTestCli, _, _, _>(
            &agent_spec(),
            ["tool", "query", "--limit", "7"],
            |cli| match cli.command {
                ParseTestCommand::Query { limit } => limit,
                ParseTestCommand::Admin => 0,
            },
        )
        .expect("parse should succeed");

        assert_eq!(parsed, AgentDispatch::Cli(7));
    }
}
