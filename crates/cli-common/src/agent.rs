//! Shared agent-mode capability declarations and filtering helpers.

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

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, OnceLock};

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
}
