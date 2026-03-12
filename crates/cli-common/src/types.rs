//! Shared types for Workhelix CLI tools.

/// Repository information for CLI tools.
///
/// This structure holds basic repository metadata for identification purposes.
#[derive(Debug, Clone)]
pub struct RepoInfo {
    /// Repository owner (e.g., "workhelix")
    pub owner: &'static str,
    /// Repository name (e.g., "prompter")
    pub name: &'static str,
}

impl RepoInfo {
    /// Create a new `RepoInfo` instance.
    #[must_use]
    pub const fn new(owner: &'static str, name: &'static str) -> Self {
        Self { owner, name }
    }
}

/// Health check result for doctor command.
#[derive(Debug, Clone)]
pub struct DoctorCheck {
    /// Name of the check
    pub name: String,
    /// Whether the check passed
    pub passed: bool,
    /// Optional message
    pub message: Option<String>,
}

impl DoctorCheck {
    /// Create a new passing check.
    #[must_use]
    pub fn pass(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: None,
        }
    }

    /// Create a new failing check with a message.
    #[must_use]
    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: Some(message.into()),
        }
    }

    /// Create a file existence check.
    ///
    /// # Errors
    /// Returns a failing check if the file doesn't exist.
    pub fn file_exists(path: impl AsRef<std::path::Path>) -> Self {
        let path_ref = path.as_ref();
        if path_ref.exists() && path_ref.is_file() {
            Self::pass(format!("File exists: {}", path_ref.display()))
        } else {
            Self::fail(
                format!("File check: {}", path_ref.display()),
                format!("File not found: {}", path_ref.display()),
            )
        }
    }

    /// Create a directory existence check.
    ///
    /// # Errors
    /// Returns a failing check if the directory doesn't exist.
    pub fn dir_exists(path: impl AsRef<std::path::Path>) -> Self {
        let path_ref = path.as_ref();
        if path_ref.exists() && path_ref.is_dir() {
            Self::pass(format!("Directory exists: {}", path_ref.display()))
        } else {
            Self::fail(
                format!("Directory check: {}", path_ref.display()),
                format!("Directory not found: {}", path_ref.display()),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_info_creation() {
        let repo = RepoInfo::new("workhelix", "prompter");
        assert_eq!(repo.owner, "workhelix");
        assert_eq!(repo.name, "prompter");
    }

    #[test]
    fn test_doctor_check_pass() {
        let check = DoctorCheck::pass("test check");
        assert!(check.passed);
        assert_eq!(check.name, "test check");
        assert!(check.message.is_none());
    }

    #[test]
    fn test_doctor_check_fail() {
        let check = DoctorCheck::fail("test check", "error message");
        assert!(!check.passed);
        assert_eq!(check.name, "test check");
        assert_eq!(check.message, Some("error message".to_string()));
    }
}
