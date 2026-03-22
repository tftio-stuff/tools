//! Health check and diagnostics module.
//!
//! This module provides a framework for running health checks on CLI tools
//! with tool-specific diagnostics.

use crate::types::{DoctorCheck, RepoInfo};
use serde_json::{Map, Value, json};
use std::fmt::Write as _;

/// Structured doctor report reusable for text and JSON output.
#[derive(Debug, Clone)]
pub struct DoctorReport {
    header: String,
    checks: Vec<DoctorCheck>,
    errors: Vec<String>,
    warnings: Vec<String>,
    info: Vec<String>,
    version: Option<String>,
    details: Map<String, Value>,
}

impl DoctorReport {
    /// Create an empty doctor report.
    #[must_use]
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            checks: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
            version: None,
            details: Map::new(),
        }
    }

    /// Create a doctor report scaffold for a tool using the standard header, version, and checks.
    #[must_use]
    pub fn for_tool<T: DoctorChecks>(tool: &T) -> Self {
        Self::with_tool_header(tool, format!("🏥 {} health check", T::repo_info().name))
    }

    /// Create a doctor report scaffold for a tool with a caller-provided header.
    #[must_use]
    pub fn with_tool_header<T: DoctorChecks>(tool: &T, header: impl Into<String>) -> Self {
        Self::new(header)
            .with_checks(tool.tool_checks())
            .with_version(T::current_version())
    }

    /// Set the report checks.
    #[must_use]
    pub fn with_checks(mut self, checks: Vec<DoctorCheck>) -> Self {
        self.checks = checks;
        self
    }

    /// Set the reported version string.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Add an error line.
    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self
    }

    /// Add a warning line.
    #[must_use]
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Add an informational line.
    #[must_use]
    pub fn with_info(mut self, info: impl Into<String>) -> Self {
        self.info.push(info.into());
        self
    }

    /// Add a custom JSON detail field.
    #[must_use]
    pub fn with_detail(mut self, key: impl Into<String>, value: Value) -> Self {
        self.details.insert(key.into(), value);
        self
    }

    /// Access the underlying checks.
    #[must_use]
    pub fn checks(&self) -> &[DoctorCheck] {
        &self.checks
    }

    fn failed_checks(&self) -> usize {
        self.checks.iter().filter(|check| !check.passed).count()
    }

    /// Return the process exit code implied by this report.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        if self.failed_checks() > 0 || !self.errors.is_empty() {
            1
        } else {
            0
        }
    }

    /// Render the report as JSON.
    #[must_use]
    pub fn to_json_value(&self) -> Value {
        let mut value = json!({
            "ok": self.exit_code() == 0,
            "header": self.header,
            "checks": self
                .checks
                .iter()
                .map(|check| json!({
                    "name": check.name,
                    "passed": check.passed,
                    "message": check.message,
                }))
                .collect::<Vec<_>>(),
            "errors": self.errors,
            "warnings": self.warnings,
            "info": self.info,
            "version": self.version,
        });

        let object = value
            .as_object_mut()
            .expect("doctor report JSON must be an object");
        for (key, detail) in &self.details {
            object.insert(key.clone(), detail.clone());
        }
        value
    }

    /// Render the report as plain text.
    #[must_use]
    pub fn render_text(&self) -> String {
        let mut output = String::new();
        writeln!(&mut output, "{}", self.header).expect("write to string");
        writeln!(&mut output, "{}", "=".repeat(self.header.chars().count()))
            .expect("write to string");
        writeln!(&mut output).expect("write to string");

        if !self.checks.is_empty() {
            writeln!(&mut output, "Configuration:").expect("write to string");
            for check in &self.checks {
                if check.passed {
                    writeln!(&mut output, "  ✅ {}", check.name).expect("write to string");
                } else {
                    writeln!(&mut output, "  ❌ {}", check.name).expect("write to string");
                    if let Some(message) = &check.message {
                        writeln!(&mut output, "     {message}").expect("write to string");
                    }
                }
            }
            writeln!(&mut output).expect("write to string");
        }

        if !self.info.is_empty() {
            writeln!(&mut output, "Info:").expect("write to string");
            for info in &self.info {
                writeln!(&mut output, "  ℹ️  {info}").expect("write to string");
            }
            writeln!(&mut output).expect("write to string");
        }

        if !self.warnings.is_empty() {
            writeln!(&mut output, "Warnings:").expect("write to string");
            for warning in &self.warnings {
                writeln!(&mut output, "  ⚠️  {warning}").expect("write to string");
            }
            writeln!(&mut output).expect("write to string");
        }

        if self.exit_code() == 0 {
            writeln!(&mut output, "✨ Everything looks healthy!").expect("write to string");
        } else {
            writeln!(&mut output, "❌ Issues found - see above for details")
                .expect("write to string");
        }

        output
    }

    /// Emit the report in the selected format and return its exit code.
    #[must_use]
    pub fn emit(&self, json: bool) -> i32 {
        if json {
            print_doctor_report_json(self)
        } else {
            print_doctor_report_text(self)
        }
    }
}

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
    let header = format!("🏥 {} health check", T::repo_info().name);
    run_doctor_with_header(tool, &header)
}

fn build_doctor_report<T: DoctorChecks>(tool: &T, header: &str) -> DoctorReport {
    DoctorReport::with_tool_header(tool, header)
}

fn render_doctor_with_header<T: DoctorChecks>(tool: &T, header: &str) -> (String, i32) {
    let report = build_doctor_report(tool, header);
    (report.render_text(), report.exit_code())
}

/// Run doctor output with a custom header.
pub fn run_doctor_with_header<T: DoctorChecks>(tool: &T, header: &str) -> i32 {
    let (output, exit_code) = render_doctor_with_header(tool, header);
    print!("{output}");
    exit_code
}

/// Print a structured doctor report as JSON and return its exit code.
#[must_use]
pub fn print_doctor_report_json(report: &DoctorReport) -> i32 {
    println!(
        "{}",
        serde_json::to_string_pretty(&report.to_json_value())
            .expect("doctor report JSON must serialize")
    );
    report.exit_code()
}

/// Print a structured doctor report as plain text and return its exit code.
#[must_use]
pub fn print_doctor_report_text(report: &DoctorReport) -> i32 {
    print!("{}", report.render_text());
    report.exit_code()
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

    #[test]
    fn test_run_doctor_with_custom_header() {
        let tool = TestTool;
        let (output, exit_code) = render_doctor_with_header(&tool, "Custom Header");
        assert!(output.contains("Custom Header"));
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn doctor_report_json_includes_details() {
        let report = DoctorReport::new("Header")
            .with_checks(vec![DoctorCheck::pass("check")])
            .with_detail("config_file_exists", json!(true));

        let value = report.to_json_value();
        assert_eq!(value["config_file_exists"], json!(true));
        assert_eq!(value["ok"], json!(true));
    }

    #[test]
    fn doctor_report_for_tool_uses_repo_name_version_and_checks() {
        let report = DoctorReport::for_tool(&TestTool);
        let value = report.to_json_value();

        assert_eq!(value["header"], json!("🏥 test-tool health check"));
        assert_eq!(value["version"], json!("1.0.0"));
        assert_eq!(value["checks"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn doctor_report_emit_returns_exit_code_for_selected_format() {
        let report = DoctorReport::for_tool(&TestTool);
        assert_eq!(report.emit(true), 1);
    }
}
