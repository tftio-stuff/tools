//! Common functionality for Workhelix Rust CLI tools.
//!
//! This library provides shared functionality for CLI tools including:
//! - Shell completion generation
//! - Health check framework
//! - License display
//! - Terminal output utilities
//!
//! # Example Usage
//!
//! ```no_run
//! use tftio_cli_common::{
//!     RepoInfo, DoctorChecks, DoctorCheck,
//!     completions, doctor, license,
//! };
//! use clap::Parser;
//!
//! #[derive(Parser)]
//! struct Cli {
//!     // your CLI definition
//! }
//!
//! struct MyTool;
//!
//! impl DoctorChecks for MyTool {
//!     fn repo_info() -> RepoInfo {
//!         RepoInfo::new("myorg", "mytool")
//!     }
//!
//!     fn current_version() -> &'static str {
//!         env!("CARGO_PKG_VERSION")
//!     }
//!
//!     fn tool_checks(&self) -> Vec<DoctorCheck> {
//!         vec![
//!             DoctorCheck::file_exists("~/.config/mytool/config.toml"),
//!         ]
//!     }
//! }
//!
//! // Generate completions
//! completions::generate_completions::<Cli>(clap_complete::Shell::Bash);
//!
//! // Run health check
//! let tool = MyTool;
//! let exit_code = doctor::run_doctor(&tool);
//! ```

// Re-export main types and traits
pub use doctor::DoctorChecks;
pub use license::LicenseType;
pub use types::{DoctorCheck, RepoInfo};

// Public modules
pub mod agent_docs;
pub mod completions;
pub mod doctor;
pub mod license;
pub mod output;
pub mod types;
pub mod update;

// Re-export commonly used items
pub use agent_docs::{
    AgentArgument, AgentCommand, AgentConfigFile, AgentDoc, AgentDocRequest, AgentEnvironmentVar,
    AgentExample, AgentFailureMode, AgentOperatorMistake, AgentOutputShape, AgentPath,
    AgentSection, AgentTool, AgentUsage, detect_agent_doc_request, render_agent_help_yaml,
    render_agent_skill,
};
pub use completions::generate_completions;
pub use doctor::run_doctor;
pub use license::display_license;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_info_creation() {
        let repo = RepoInfo::new("workhelix", "test");
        assert_eq!(repo.owner, "workhelix");
        assert_eq!(repo.name, "test");
    }

    #[test]
    fn test_doctor_check_creation() {
        let check = DoctorCheck::pass("test");
        assert!(check.passed);

        let check = DoctorCheck::fail("test", "failed");
        assert!(!check.passed);
    }

    #[test]
    fn test_license_type() {
        assert_eq!(LicenseType::MIT.name(), "MIT");
        assert_eq!(LicenseType::Apache2.name(), "Apache-2.0");
        assert_eq!(LicenseType::CC0.name(), "CC0-1.0");
    }
}
