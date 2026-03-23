//! Shared agent-mode capability declarations and filtering helpers.

use std::collections::BTreeSet;

use clap::{Arg, Command};

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
    /// Human-readable summary for the capability.
    pub summary: &'static str,
    /// Command paths exposed by this capability.
    pub commands: &'static [CommandSelector],
    /// Long flags exposed by this capability.
    pub flags: &'static [FlagSelector],
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
            summary,
            commands,
            flags,
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

    let filtered = filter_command(command, spec.bin_name, spec.version, visible_capabilities(spec, ctx), &[]);
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

    let mut filtered = Command::new(name);

    if current_path.is_empty() {
        filtered = filtered.version(version);
    }

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

fn extend_path<'a>(current_path: &[&'a str], segment: &'a str) -> Vec<&'a str> {
    let mut next_path = current_path.to_vec();
    next_path.push(segment);
    next_path
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

    use clap::{Arg, Command};

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

    const AGENT_SURFACE: AgentSurfaceSpec =
        AgentSurfaceSpec::new(&[QUERY_CAPABILITY, STATUS_CAPABILITY]);

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
}
