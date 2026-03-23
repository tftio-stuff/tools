//! Shared CLI application metadata.

use crate::{LicenseType, RepoInfo};

/// Default repository metadata for binaries that ship from the shared tools workspace.
pub const WORKSPACE_REPO: RepoInfo = RepoInfo::new("tftio-stuff", "tools");

/// Shared metadata for a CLI binary.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    /// Binary name shown in version output and help text.
    pub bin_name: &'static str,
    /// Human-readable tool name.
    pub display_name: &'static str,
    /// Binary version.
    pub version: &'static str,
    /// License rendered by the shared license command.
    pub license: LicenseType,
    /// Repository metadata used by update helpers.
    pub repo: RepoInfo,
    /// Whether the tool supports JSON output in base commands.
    pub supports_json: bool,
    /// Whether the tool exposes doctor checks.
    pub supports_doctor: bool,
    /// Whether the tool exposes update support.
    pub supports_update: bool,
}

impl ToolSpec {
    /// Create a new [`ToolSpec`].
    #[must_use]
    pub const fn new(
        bin_name: &'static str,
        display_name: &'static str,
        version: &'static str,
        license: LicenseType,
        repo: RepoInfo,
        supports_json: bool,
        supports_doctor: bool,
        supports_update: bool,
    ) -> Self {
        Self {
            bin_name,
            display_name,
            version,
            license,
            repo,
            supports_json,
            supports_doctor,
            supports_update,
        }
    }

    /// Create a new [`ToolSpec`] using [`WORKSPACE_REPO`].
    #[must_use]
    pub const fn workspace(
        bin_name: &'static str,
        display_name: &'static str,
        version: &'static str,
        license: LicenseType,
        supports_json: bool,
        supports_doctor: bool,
        supports_update: bool,
    ) -> Self {
        Self::new(
            bin_name,
            display_name,
            version,
            license,
            WORKSPACE_REPO,
            supports_json,
            supports_doctor,
            supports_update,
        )
    }
}

/// Create a [`ToolSpec`] for a binary shipped from the shared tools workspace.
#[must_use]
pub const fn workspace_tool(
    bin_name: &'static str,
    display_name: &'static str,
    version: &'static str,
    license: LicenseType,
    supports_json: bool,
    supports_doctor: bool,
    supports_update: bool,
) -> ToolSpec {
    ToolSpec::workspace(
        bin_name,
        display_name,
        version,
        license,
        supports_json,
        supports_doctor,
        supports_update,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_spec_new_preserves_fields() {
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

        assert_eq!(spec.bin_name, "tool");
        assert_eq!(spec.display_name, "Tool");
        assert_eq!(spec.version, "1.2.3");
        assert_eq!(spec.license, LicenseType::MIT);
        assert_eq!(spec.repo.owner, "owner");
        assert_eq!(spec.repo.name, "repo");
        assert!(spec.supports_json);
        assert!(!spec.supports_doctor);
        assert!(spec.supports_update);
    }

    #[test]
    fn workspace_tool_uses_workspace_repo_defaults() {
        let spec = workspace_tool(
            "tool",
            "Tool",
            "1.2.3",
            LicenseType::MIT,
            true,
            false,
            false,
        );

        assert_eq!(spec.repo.owner, WORKSPACE_REPO.owner);
        assert_eq!(spec.repo.name, WORKSPACE_REPO.name);
        assert_eq!(spec.bin_name, "tool");
        assert_eq!(spec.display_name, "Tool");
    }
}
