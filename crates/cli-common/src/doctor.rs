//! Health check and diagnostics module.
//!
//! This module provides a framework for running health checks on CLI tools
//! with tool-specific diagnostics.

use crate::types::{DoctorCheck, RepoInfo};

/// Trait for tools that support doctor health checks.
///
/// Implement this trait to provide tool-specific health checks.
pub trait DoctorChecks {
    /// Get the repository information for this tool.
    fn repo_info() -> RepoInfo;

    /// Get the current version of this tool.
    fn current_version() -> &'static str;

    /// Run tool-specific health checks.
    ///
    /// Return a vector of check results. Default implementation returns empty vector.
    fn tool_checks(&self) -> Vec<DoctorCheck> {
        Vec::new()
    }
}

/// Run doctor command to check health and configuration.
///
/// Returns exit code: 0 if healthy, 1 if issues found.
///
/// # Type Parameters
/// * `T` - A type that implements `DoctorChecks`
pub fn run_doctor<T: DoctorChecks>(tool: &T) -> i32 {
    let tool_name = T::repo_info().name;
    println!("🏥 {tool_name} health check");
    println!("{}", "=".repeat(tool_name.len() + 14));
    println!();

    let mut has_errors = false;
    let has_warnings = false;

    // Run tool-specific checks
    let tool_checks = tool.tool_checks();
    if !tool_checks.is_empty() {
        println!("Configuration:");
        for check in tool_checks {
            if check.passed {
                println!("  ✅ {}", check.name);
            } else {
                println!("  ❌ {}", check.name);
                if let Some(msg) = check.message {
                    println!("     {msg}");
                }
                has_errors = true;
            }
        }
        println!();
    }

    // Summary
    if has_errors {
        println!("❌ Issues found - see above for details");
        1
    } else if has_warnings {
        println!("⚠️  Warnings found");
        0 // Warnings don't cause failure
    } else {
        println!("✨ Everything looks healthy!");
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestTool;

    impl DoctorChecks for TestTool {
        fn repo_info() -> RepoInfo {
            RepoInfo::new("workhelix", "test-tool")
        }

        fn current_version() -> &'static str {
            "1.0.0"
        }

        fn tool_checks(&self) -> Vec<DoctorCheck> {
            vec![
                DoctorCheck::pass("Test check 1"),
                DoctorCheck::fail("Test check 2", "This is a failure"),
            ]
        }
    }

    #[test]
    fn test_run_doctor() {
        let tool = TestTool;
        let exit_code = run_doctor(&tool);
        // Should return 1 because we have a failing check
        assert_eq!(exit_code, 1);
    }
}
