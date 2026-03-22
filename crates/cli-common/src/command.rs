//! Shared standard CLI commands.

use std::path::PathBuf;

use clap::CommandFactory;
use clap_complete::Shell;

use crate::{DoctorChecks, ToolSpec, display_license, generate_completions, run_doctor, update};

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

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;
    use crate::{LicenseType, RepoInfo};

    #[derive(Parser)]
    struct TestCli;

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

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum GlobalJsonMetaCommand {
        Version,
        License,
        Completions { shell: Shell },
    }

    impl_standard_command_map!(GlobalJsonMetaCommand, global_json);

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum FixedJsonMetaCommand {
        Version,
        License,
        Completions { shell: Shell },
        Doctor,
    }

    impl_standard_command_map!(FixedJsonMetaCommand, fixed_json = false, doctor);

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

    #[test]
    fn impl_standard_command_map_reads_json_from_version_field() {
        let command = map_standard_command(&VersionFieldMetaCommand::Version { json: true }, false);
        assert_eq!(command, StandardCommand::Version { json: true });
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
}
