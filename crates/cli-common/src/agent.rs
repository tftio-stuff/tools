//! Shared agent-mode capability declarations and filtering helpers.

use std::{collections::BTreeSet, ffi::OsString};

use clap::{
    Arg, ArgAction, Command, CommandFactory, FromArgMatches,
    error::ErrorKind,
};

use crate::ToolSpec;

/// Environment variable containing the presented agent token.
pub const AGENT_TOKEN_ENV: &str = "TFTIO_AGENT_TOKEN";

/// Environment variable containing the expected agent token.
pub const AGENT_TOKEN_EXPECTED_ENV: &str = "TFTIO_AGENT_TOKEN_EXPECTED";

/// Shared process-level agent-mode context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentModeContext {
    /// Whether agent mode is active for the current process.
    pub active: bool,
}

impl AgentModeContext {
    /// Detect whether agent mode is active for the current process.
    #[must_use]
    pub fn detect() -> Self {
        let presented = std::env::var(AGENT_TOKEN_ENV).ok();
        let expected = std::env::var(AGENT_TOKEN_EXPECTED_ENV).ok();

        Self {
            active: matches!((presented, expected), (Some(presented), Some(expected)) if presented == expected),
        }
    }
}

/// Declarative capability surface for a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentSurfaceSpec {
    /// Named capabilities visible to the agent surface.
    pub capabilities: &'static [AgentCapability],
}

impl AgentSurfaceSpec {
    /// Create a new [`AgentSurfaceSpec`].
    #[must_use]
    pub const fn new(capabilities: &'static [AgentCapability]) -> Self {
        Self { capabilities }
    }
}

/// Declarative capability group for agent-mode visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentCapability {
    /// Stable capability name.
    pub name: &'static str,
    /// Optional human-readable summary for the capability.
    pub summary: Option<&'static str>,
    /// Command paths exposed by this capability.
    pub commands: &'static [CommandSelector],
    /// Long flags exposed by this capability.
    pub flags: &'static [FlagSelector],
    /// Optional example invocations.
    pub examples: Option<&'static [&'static str]>,
    /// Optional output contract prose.
    pub output: Option<&'static str>,
    /// Optional constraints prose.
    pub constraints: Option<&'static str>,
}

impl AgentCapability {
    /// Create a new [`AgentCapability`].
    #[must_use]
    pub const fn new(
        name: &'static str,
        summary: &'static str,
        commands: &'static [CommandSelector],
        flags: &'static [FlagSelector],
    ) -> Self {
        Self {
            name,
            summary: Some(summary),
            commands,
            flags,
            examples: None,
            output: None,
            constraints: None,
        }
    }

    /// Create a new [`AgentCapability`] without optional prose metadata.
    #[must_use]
    pub const fn minimal(
        name: &'static str,
        commands: &'static [CommandSelector],
        flags: &'static [FlagSelector],
    ) -> Self {
        Self {
            name,
            summary: None,
            commands,
            flags,
            examples: None,
            output: None,
            constraints: None,
        }
    }

    /// Attach example invocations.
    #[must_use]
    pub const fn with_examples(self, examples: &'static [&'static str]) -> Self {
        Self {
            examples: Some(examples),
            ..self
        }
    }

    /// Attach output contract prose.
    #[must_use]
    pub const fn with_output(self, output: &'static str) -> Self {
        Self {
            output: Some(output),
            ..self
        }
    }

    /// Attach constraints prose.
    #[must_use]
    pub const fn with_constraints(self, constraints: &'static str) -> Self {
        Self {
            constraints: Some(constraints),
            ..self
        }
    }
}

/// Declarative selector for a command path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSelector {
    /// Path segments for the selected command.
    pub path: &'static [&'static str],
}

impl CommandSelector {
    /// Create a new [`CommandSelector`].
    #[must_use]
    pub const fn new(path: &'static [&'static str]) -> Self {
        Self { path }
    }
}

/// Declarative selector for a long flag on a command path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlagSelector {
    /// Command path that owns the flag.
    pub command_path: &'static [&'static str],
    /// Long flag name without the leading `--`.
    pub long: &'static str,
}

impl FlagSelector {
    /// Create a new [`FlagSelector`].
    #[must_use]
    pub const fn new(command_path: &'static [&'static str], long: &'static str) -> Self {
        Self { command_path, long }
    }
}

/// Shared parse result for agent-aware entrypoints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentDispatch<T> {
    /// Continue with the parsed CLI value.
    Cli(T),
    /// A shared agent inspection path printed output and chose an exit code.
    Printed(i32),
}

/// Error returned when rendering a single agent capability view fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSkillError {
    message: String,
}

impl AgentSkillError {
    /// Create a new [`AgentSkillError`].
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AgentSkillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AgentSkillError {}

/// Return the capabilities visible in the current agent-mode context.
#[must_use]
pub fn visible_capabilities<'a>(
    spec: &'a ToolSpec,
    ctx: &AgentModeContext,
) -> &'a [AgentCapability] {
    if ctx.active {
        spec.agent_surface.map_or(&[], |surface| surface.capabilities)
    } else {
        &[]
    }
}

/// Apply the visible agent surface to a `clap` command tree.
pub fn apply_agent_surface(command: &mut Command, spec: &ToolSpec, ctx: &AgentModeContext) {
    if !ctx.active {
        return;
    }

    ensure_agent_inspection_args(command);

    let filtered = filter_command(
        command,
        spec.bin_name,
        spec.version,
        visible_capabilities(spec, ctx),
        &[],
    );
    *command = filtered;
}

fn filter_command(
    command: &Command,
    name: &'static str,
    version: &'static str,
    capabilities: &[AgentCapability],
    current_path: &[&str],
) -> Command {
    let allowed_flags = allowed_flags(capabilities, current_path);
    let allowed_subcommands = allowed_subcommands(capabilities, current_path);
    let mut filtered = clone_command_metadata(command, name, version, current_path.is_empty());

    for arg in command
        .get_arguments()
        .filter(|arg| should_keep_arg(arg, current_path, &allowed_flags))
        .cloned()
    {
        filtered = filtered.arg(arg);
    }

    for subcommand_name in &allowed_subcommands {
        if let Some(subcommand) = command.find_subcommand(subcommand_name) {
            let next_path = extend_path(current_path, subcommand_name);
            filtered = filtered.subcommand(filter_command(
                subcommand,
                subcommand_name,
                version,
                capabilities,
                &next_path,
            ));
        }
    }

    filtered
}

fn clone_command_metadata(
    command: &Command,
    name: &'static str,
    version: &'static str,
    include_version: bool,
) -> Command {
    let mut filtered = Command::new(name);

    if let Some(display_name) = command.get_display_name() {
        filtered = filtered.display_name(display_name.to_owned());
    }
    if include_version {
        filtered = filtered.version(version);
    }
    if let Some(about) = command.get_about() {
        filtered = filtered.about(about.clone());
    }
    if let Some(long_about) = command.get_long_about() {
        filtered = filtered.long_about(long_about.clone());
    }
    if let Some(before_help) = command.get_before_help() {
        filtered = filtered.before_help(before_help.clone());
    }
    if let Some(after_help) = command.get_after_help() {
        filtered = filtered.after_help(after_help.clone());
    }
    if command.is_disable_help_flag_set() {
        filtered = filtered.disable_help_flag(true);
    }
    if command.is_disable_help_subcommand_set() {
        filtered = filtered.disable_help_subcommand(true);
    }
    if command.is_disable_colored_help_set() {
        filtered = filtered.disable_colored_help(true);
    }
    if command.is_flatten_help_set() {
        filtered = filtered.flatten_help(true);
    }
    if let Some(bin_name) = command.get_bin_name() {
        filtered.set_bin_name(bin_name.to_owned());
    }

    filtered
}

fn allowed_flags(
    capabilities: &[AgentCapability],
    current_path: &[&str],
) -> BTreeSet<&'static str> {
    let mut flags = BTreeSet::new();

    for capability in capabilities {
        for selector in capability.flags {
            if selector.command_path == current_path {
                flags.insert(selector.long);
            }
        }
    }

    flags
}

fn allowed_subcommands(
    capabilities: &[AgentCapability],
    current_path: &[&str],
) -> BTreeSet<&'static str> {
    let mut subcommands = BTreeSet::new();

    for capability in capabilities {
        for selector in capability.commands {
            if selector.path.starts_with(current_path) && selector.path.len() > current_path.len() {
                subcommands.insert(selector.path[current_path.len()]);
            }
        }
    }

    subcommands
}

fn should_keep_arg(arg: &Arg, current_path: &[&str], allowed_flags: &BTreeSet<&str>) -> bool {
    if is_shared_agent_flag(arg) {
        return true;
    }

    if arg.is_positional() {
        return !current_path.is_empty();
    }

    arg.get_long()
        .is_some_and(|long| allowed_flags.contains(long))
}

fn is_shared_agent_flag(arg: &Arg) -> bool {
    matches!(arg.get_long(), Some("agent-help" | "agent-skill"))
}

fn ensure_agent_inspection_args(command: &mut Command) {
    if !command
        .get_arguments()
        .any(|arg| arg.get_long() == Some("agent-help"))
    {
        *command = command.clone().arg(
            Arg::new("agent-help")
                .long("agent-help")
                .help("Print the visible agent command surface")
                .hide(true)
                .global(true)
                .action(ArgAction::SetTrue),
        );
    }

    if !command
        .get_arguments()
        .any(|arg| arg.get_long() == Some("agent-skill"))
    {
        *command = command.clone().arg(
            Arg::new("agent-skill")
                .long("agent-skill")
                .help("Print the visible agent capability contract")
                .hide(true)
                .global(true)
                .value_name("NAME"),
        );
    }
}

fn extend_path<'a>(current_path: &[&'a str], segment: &'a str) -> Vec<&'a str> {
    let mut next_path = current_path.to_vec();
    next_path.push(segment);
    next_path
}

/// Parse argv against the normal or agent-filtered surface for a clap CLI.
///
/// # Errors
///
/// Returns a `clap` error when parsing fails or when an unknown agent capability is requested.
pub fn parse_with_agent_surface_from<T, I>(
    spec: &ToolSpec,
    argv: I,
) -> Result<AgentDispatch<T>, clap::Error>
where
    T: CommandFactory + FromArgMatches,
    I: IntoIterator,
    I::Item: Into<OsString> + Clone,
{
    let ctx = AgentModeContext::detect();
    let mut command = T::command();
    ensure_agent_inspection_args(&mut command);

    if ctx.active {
        apply_agent_surface(&mut command, spec, &ctx);
    }

    match command.try_get_matches_from_mut(argv) {
        Ok(mut matches) => {
            if matches.get_flag("agent-help") {
                println!("{}", render_agent_help(spec, &ctx));
                return Ok(AgentDispatch::Printed(0));
            }

            if let Some(name) = matches.get_one::<String>("agent-skill") {
                let text = render_agent_skill(spec, &ctx, name)
                    .map_err(|error| command.error(ErrorKind::InvalidValue, error.to_string()))?;
                println!("{text}");
                return Ok(AgentDispatch::Printed(0));
            }

            T::from_arg_matches_mut(&mut matches).map(AgentDispatch::Cli)
        }
        Err(error) => Err(sanitize_agent_parse_error(error)),
    }
}

/// Parse process argv against the normal or agent-filtered surface for a clap CLI.
///
/// # Errors
///
/// Returns a `clap` error when parsing fails or when an unknown agent capability is requested.
pub fn parse_with_agent_surface<T>(spec: &ToolSpec) -> Result<AgentDispatch<T>, clap::Error>
where
    T: CommandFactory + FromArgMatches,
{
    parse_with_agent_surface_from(spec, std::env::args_os())
}

fn sanitize_agent_parse_error(mut error: clap::Error) -> clap::Error {
    let rendered = error.to_string();
    if rendered.contains("Did you mean") {
        error = clap::Error::raw(error.kind(), strip_suggestion_lines(&rendered));
    }
    error
}

fn strip_suggestion_lines(rendered: &str) -> String {
    rendered
        .lines()
        .filter(|line| !line.contains("Did you mean"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render the visible agent capability surface as structured text.
#[must_use]
pub fn render_agent_help(spec: &ToolSpec, ctx: &AgentModeContext) -> String {
    let capabilities = visible_capabilities(spec, ctx);
    let capability_lines = if capabilities.is_empty() {
        String::from("- none")
    } else {
        capabilities
            .iter()
            .map(|capability| {
                format!(
                    "- {}: {}",
                    capability.name,
                    capability_summary(capability)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let argument_lines = render_surface_arguments(capabilities);

    format!(
        "tool:\n- {}\nmode:\n- {}\ncapabilities:\n{}\narguments:\n{}\noutput:\n- structured plain text for the visible command surface\nconstraints:\n- output is limited to the currently visible surface",
        spec.bin_name,
        if ctx.active { "agent" } else { "human" },
        capability_lines,
        argument_lines,
    )
}

/// Render the visible contract for one agent capability.
///
/// # Errors
///
/// Returns [`AgentSkillError`] when the capability name is not visible in the current context.
pub fn render_agent_skill(
    spec: &ToolSpec,
    ctx: &AgentModeContext,
    name: &str,
) -> Result<String, AgentSkillError> {
    let capability = visible_capabilities(spec, ctx)
        .iter()
        .find(|capability| capability.name == name)
        .ok_or_else(|| AgentSkillError::new(format!("unknown agent capability: {name}")))?;

    Ok(format!(
        "tool:\n- {}\ncapability:\n- {}\nsummary:\n- {}\ncommands:\n{}\nflags:\n{}\nexamples:\n{}\noutput:\n- {}\nconstraints:\n- {}",
        spec.bin_name,
        capability.name,
        capability_summary(capability),
        render_command_lines(capability),
        render_flag_lines(capability),
        render_example_lines(capability),
        capability_output(capability),
        capability_constraints(capability),
    ))
}

fn capability_summary(capability: &AgentCapability) -> String {
    if let Some(summary) = capability.summary {
        return String::from(summary);
    }

    if let Some(primary_command) = capability.commands.first() {
        return format!(
            "Use {} via {}",
            capability.name.replace('-', " "),
            primary_command.path.join(" ")
        );
    }

    format!("Use {}", capability.name.replace('-', " "))
}

fn capability_output(capability: &AgentCapability) -> String {
    capability.output.map_or_else(
        || {
            if let Some(primary_command) = capability.commands.first() {
                format!(
                    "output follows the existing CLI contract for {}",
                    primary_command.path.join(" ")
                )
            } else {
                String::from("output follows the existing CLI contract")
            }
        },
        String::from,
    )
}

fn capability_constraints(capability: &AgentCapability) -> String {
    capability.constraints.map_or_else(
        || String::from("existing command validation and auth rules still apply"),
        String::from,
    )
}

fn render_example_lines(capability: &AgentCapability) -> String {
    capability.examples.map_or_else(
        || String::from("- none declared"),
        |examples| {
            if examples.is_empty() {
                String::from("- none declared")
            } else {
                examples
                    .iter()
                    .map(|example| format!("- {example}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        },
    )
}

fn render_surface_arguments(capabilities: &[AgentCapability]) -> String {
    let mut lines = vec![
        String::from("- --agent-help"),
        String::from("- --agent-skill <NAME>"),
    ];

    for capability in capabilities {
        for command in capability.commands {
            lines.push(format!("- command {}", command.path.join(" ")));
        }
        for flag in capability.flags {
            lines.push(format!(
                "- {} --{}",
                flag.command_path.join(" "),
                flag.long
            ));
        }
    }

    lines.join("\n")
}

fn render_command_lines(capability: &AgentCapability) -> String {
    if capability.commands.is_empty() {
        String::from("- none declared")
    } else {
        capability
            .commands
            .iter()
            .map(|selector| format!("- {}", selector.path.join(" ")))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn render_flag_lines(capability: &AgentCapability) -> String {
    if capability.flags.is_empty() {
        String::from("- none declared")
    } else {
        capability
            .flags
            .iter()
            .map(|selector| format!("- {} --{}", selector.command_path.join(" "), selector.long))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use clap::{Arg, Args, Command, Parser, Subcommand};

    use super::*;
    use crate::{LicenseType, RepoInfo, ToolSpec};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    const QUERY_COMMAND: CommandSelector = CommandSelector::new(&["query"]);
    const STATUS_COMMAND: CommandSelector = CommandSelector::new(&["status"]);
    const QUERY_LIMIT_FLAG: FlagSelector = FlagSelector::new(&["query"], "limit");
    const QUERY_OFFSET_FLAG: FlagSelector = FlagSelector::new(&["query"], "offset");

    const QUERY_CAPABILITY: AgentCapability = AgentCapability::new(
        "query-posts",
        "Read paginated post records",
        &[QUERY_COMMAND],
        &[QUERY_LIMIT_FLAG, QUERY_OFFSET_FLAG],
    );

    const STATUS_CAPABILITY: AgentCapability = AgentCapability::new(
        "inspect-status",
        "Inspect current status",
        &[STATUS_COMMAND],
        &[],
    );
    const QUERY_MINIMAL_CAPABILITY: AgentCapability =
        AgentCapability::minimal("query-minimal", &[QUERY_COMMAND], &[QUERY_LIMIT_FLAG]);

    const AGENT_SURFACE: AgentSurfaceSpec =
        AgentSurfaceSpec::new(&[QUERY_CAPABILITY, STATUS_CAPABILITY]);
    const MINIMAL_AGENT_SURFACE: AgentSurfaceSpec =
        AgentSurfaceSpec::new(&[QUERY_MINIMAL_CAPABILITY]);

    fn spec() -> ToolSpec {
        ToolSpec::new(
            "tool",
            "Tool",
            "1.2.3",
            LicenseType::MIT,
            RepoInfo::new("owner", "repo"),
            true,
            false,
            true,
        )
        .with_agent_surface(&AGENT_SURFACE)
    }

    fn minimal_spec() -> ToolSpec {
        ToolSpec::new(
            "tool",
            "Tool",
            "1.2.3",
            LicenseType::MIT,
            RepoInfo::new("owner", "repo"),
            true,
            false,
            true,
        )
        .with_agent_surface(&MINIMAL_AGENT_SURFACE)
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
    fn agent_mode_activation_is_inactive_without_presented_token() {
        let _guard = env_lock();
        set_tokens(None, Some("expected"));

        let ctx = AgentModeContext::detect();

        assert!(!ctx.active);
    }

    #[test]
    fn agent_mode_activation_is_inactive_without_expected_token() {
        let _guard = env_lock();
        set_tokens(Some("presented"), None);

        let ctx = AgentModeContext::detect();

        assert!(!ctx.active);
    }

    #[test]
    fn agent_mode_activation_is_inactive_on_exact_string_mismatch() {
        let _guard = env_lock();
        set_tokens(Some("presented"), Some("expected"));

        let ctx = AgentModeContext::detect();

        assert!(!ctx.active);
    }

    #[test]
    fn agent_mode_activation_is_active_on_exact_string_match() {
        let _guard = env_lock();
        set_tokens(Some("shared-token"), Some("shared-token"));

        let ctx = AgentModeContext::detect();

        assert!(ctx.active);
    }

    #[test]
    fn agent_mode_activation_preserves_capability_declarations() {
        let spec = spec();
        let capability = spec
            .agent_surface
            .expect("agent surface present")
            .capabilities
            .first()
            .expect("capability present");

        assert_eq!(capability.name, "query-posts");
        assert_eq!(capability.commands[0].path, ["query"]);
        assert_eq!(capability.flags[0].command_path, ["query"]);
        assert_eq!(capability.flags[0].long, "limit");
        assert_eq!(capability.flags[1].long, "offset");
    }

    #[test]
    fn capability_policy_removes_undeclared_subcommand() {
        let mut command = sample_command();

        apply_agent_surface(&mut command, &spec(), &AgentModeContext { active: true });

        assert!(command.find_subcommand("query").is_some());
        assert!(command.find_subcommand("status").is_some());
        assert!(command.find_subcommand("admin").is_none());
    }

    #[test]
    fn capability_policy_removes_undeclared_flag() {
        let mut command = sample_command();

        apply_agent_surface(&mut command, &spec(), &AgentModeContext { active: true });

        let query = command.find_subcommand("query").expect("query present");
        assert!(query.get_arguments().any(|arg| arg.get_long() == Some("limit")));
        assert!(query.get_arguments().any(|arg| arg.get_long() == Some("offset")));
        assert!(!query.get_arguments().any(|arg| arg.get_long() == Some("secret")));
    }

    #[test]
    fn capability_policy_returns_empty_surface_without_declared_capabilities() {
        let spec = ToolSpec::new(
            "tool",
            "Tool",
            "1.2.3",
            LicenseType::MIT,
            RepoInfo::new("owner", "repo"),
            true,
            false,
            true,
        );
        let mut command = sample_command();

        apply_agent_surface(&mut command, &spec, &AgentModeContext { active: true });

        assert!(visible_capabilities(&spec, &AgentModeContext { active: true }).is_empty());
        assert!(command.find_subcommand("query").is_none());
        assert!(command.find_subcommand("status").is_none());
        assert!(command.find_subcommand("admin").is_none());
        assert!(
            command
                .get_arguments()
                .any(|arg| arg.get_long() == Some("agent-help"))
        );
        assert!(
            command
                .get_arguments()
                .any(|arg| arg.get_long() == Some("agent-skill"))
        );
    }

    fn sample_command() -> Command {
        Command::new("tool")
            .arg(Arg::new("agent-help").long("agent-help"))
            .arg(Arg::new("agent-skill").long("agent-skill").value_name("NAME"))
            .subcommand(
                Command::new("query")
                    .arg(Arg::new("limit").long("limit"))
                    .arg(Arg::new("offset").long("offset"))
                    .arg(Arg::new("secret").long("secret")),
            )
            .subcommand(Command::new("status"))
            .subcommand(Command::new("admin").arg(Arg::new("danger").long("danger")))
    }

    #[derive(Debug, Parser, PartialEq, Eq)]
    #[command(name = "tool")]
    struct AgentTestCli {
        #[command(subcommand)]
        command: Option<AgentTestCommand>,
    }

    #[derive(Debug, Subcommand, PartialEq, Eq)]
    enum AgentTestCommand {
        Query(QueryArgs),
        Status,
        Admin(AdminArgs),
    }

    #[derive(Debug, Args, PartialEq, Eq)]
    struct QueryArgs {
        #[arg(long)]
        limit: Option<u32>,
        #[arg(long)]
        offset: Option<u32>,
        #[arg(long)]
        secret: bool,
    }

    #[derive(Debug, Args, PartialEq, Eq)]
    struct AdminArgs {
        #[arg(long)]
        danger: bool,
    }

    #[test]
    fn agent_surface_redaction_rejects_hidden_command_and_flag() {
        let _guard = env_lock();
        set_tokens(Some("shared-token"), Some("shared-token"));
        let spec = spec();

        let hidden_command_error =
            parse_with_agent_surface_from::<AgentTestCli, _>(&spec, ["tool", "admin"])
                .expect_err("hidden subcommand should be rejected")
                .to_string();
        assert!(hidden_command_error.contains("unrecognized subcommand"));

        let hidden_command_typo_error =
            parse_with_agent_surface_from::<AgentTestCli, _>(&spec, ["tool", "admni"])
                .expect_err("hidden subcommand typo should not leak suggestions")
                .to_string();
        assert!(hidden_command_typo_error.contains("unrecognized subcommand"));
        assert!(!hidden_command_typo_error.contains("Did you mean"));

        let hidden_flag_error = parse_with_agent_surface_from::<AgentTestCli, _>(
            &spec,
            ["tool", "query", "--secre"],
        )
        .expect_err("hidden flag typo should be rejected")
        .to_string();
        assert!(hidden_flag_error.contains("unexpected argument"));
        assert!(!hidden_flag_error.contains("--secret"));
        assert!(!hidden_flag_error.contains("Did you mean"));
    }

    #[test]
    fn agent_surface_redaction_help_omits_hidden_entries() {
        let _guard = env_lock();
        set_tokens(Some("shared-token"), Some("shared-token"));
        let spec = spec();

        let long_help = parse_with_agent_surface_from::<AgentTestCli, _>(&spec, ["tool", "--help"])
            .expect_err("help should short-circuit through clap")
            .to_string();
        assert!(long_help.contains("query"));
        assert!(long_help.contains("status"));
        assert!(!long_help.contains("admin"));
        assert!(!long_help.contains("--secret"));

        let help_subcommand = parse_with_agent_surface_from::<AgentTestCli, _>(&spec, ["tool", "help"])
            .expect_err("help subcommand should short-circuit through clap")
            .to_string();
        assert!(help_subcommand.contains("query"));
        assert!(help_subcommand.contains("status"));
        assert!(!help_subcommand.contains("admin"));
        assert!(!help_subcommand.contains("--secret"));
    }

    #[test]
    fn agent_surface_redaction_preserves_human_mode_surface() {
        let _guard = env_lock();
        set_tokens(None, None);
        let spec = spec();

        let admin = parse_with_agent_surface_from::<AgentTestCli, _>(
            &spec,
            ["tool", "admin", "--danger"],
        )
        .expect("human mode should keep the full command tree");
        assert_eq!(
            admin,
            AgentDispatch::Cli(AgentTestCli {
                command: Some(AgentTestCommand::Admin(AdminArgs { danger: true })),
            })
        );

        let query = parse_with_agent_surface_from::<AgentTestCli, _>(
            &spec,
            ["tool", "query", "--secret"],
        )
        .expect("human mode should keep hidden flags available");
        assert_eq!(
            query,
            AgentDispatch::Cli(AgentTestCli {
                command: Some(AgentTestCommand::Query(QueryArgs {
                    limit: None,
                    offset: None,
                    secret: true,
                })),
            })
        );
    }

    #[test]
    fn agent_surface_redaction_agent_flags_short_circuit() {
        let _guard = env_lock();
        set_tokens(Some("shared-token"), Some("shared-token"));
        let spec = spec();

        let help = parse_with_agent_surface_from::<AgentTestCli, _>(&spec, ["tool", "--agent-help"])
            .expect("agent help should print and exit");
        assert_eq!(help, AgentDispatch::Printed(0));

        let skill = parse_with_agent_surface_from::<AgentTestCli, _>(
            &spec,
            ["tool", "--agent-skill", "query-posts"],
        )
        .expect("agent skill should print and exit");
        assert_eq!(skill, AgentDispatch::Printed(0));
    }

    #[test]
    fn agent_help_render_sections_are_structured_and_redacted() {
        let rendered = render_agent_help(&spec(), &AgentModeContext { active: true });

        let section_positions = [
            rendered.find("tool:\n").expect("tool section"),
            rendered.find("mode:\n").expect("mode section"),
            rendered
                .find("capabilities:\n")
                .expect("capabilities section"),
            rendered.find("arguments:\n").expect("arguments section"),
            rendered.find("output:\n").expect("output section"),
            rendered
                .find("constraints:\n")
                .expect("constraints section"),
        ];
        assert!(section_positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(rendered.contains("query-posts"));
        assert!(rendered.contains("inspect-status"));
        assert!(!rendered.contains("admin"));
        assert!(!rendered.contains("--secret"));
    }

    #[test]
    fn agent_help_render_skill_output_is_single_capability_only() {
        let rendered = render_agent_skill(&spec(), &AgentModeContext { active: true }, "query-posts")
            .expect("capability should render");

        let section_positions = [
            rendered.find("tool:\n").expect("tool section"),
            rendered
                .find("capability:\n")
                .expect("capability section"),
            rendered.find("summary:\n").expect("summary section"),
            rendered.find("commands:\n").expect("commands section"),
            rendered.find("flags:\n").expect("flags section"),
            rendered.find("examples:\n").expect("examples section"),
            rendered.find("output:\n").expect("output section"),
            rendered
                .find("constraints:\n")
                .expect("constraints section"),
        ];
        assert!(section_positions.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(rendered.contains("query-posts"));
        assert!(!rendered.contains("inspect-status"));
        assert!(rendered.contains("query"));
        assert!(rendered.contains("--limit"));
        assert!(rendered.contains("--offset"));
    }

    #[test]
    fn agent_help_render_unknown_skill_is_bounded() {
        let error = render_agent_skill(&spec(), &AgentModeContext { active: true }, "missing")
            .expect_err("unknown capability should fail");

        assert_eq!(error.to_string(), "unknown agent capability: missing");
        assert!(!error.to_string().contains("query-posts"));
        assert!(!error.to_string().contains("inspect-status"));
    }

    #[test]
    fn agent_help_render_fills_missing_prose_metadata() {
        let rendered = render_agent_skill(
            &minimal_spec(),
            &AgentModeContext { active: true },
            "query-minimal",
        )
        .expect("minimal capability should render");

        assert!(rendered.contains("capability:\n- query-minimal"));
        assert!(rendered.contains("summary:\n-"));
        assert!(rendered.contains("commands:\n- query"));
        assert!(rendered.contains("flags:\n- query --limit"));
        assert!(rendered.contains("examples:\n- none declared"));
        assert!(rendered.contains("output:\n-"));
        assert!(rendered.contains("constraints:\n-"));
    }
}
