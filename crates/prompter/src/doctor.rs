//! Health check and diagnostics module.

use serde_json::json;
use std::path::Path;
use tftio_cli_common::{DoctorCheck, DoctorChecks, DoctorReport, RepoInfo};

pub struct PrompterDoctor;

impl DoctorChecks for PrompterDoctor {
    fn repo_info() -> RepoInfo {
        RepoInfo::new("tftio-stuff", "tools")
    }

    fn current_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn tool_checks(&self) -> Vec<DoctorCheck> {
        let state = collect_doctor_state();
        let mut checks = Vec::new();

        if state.config_file_exists {
            checks.push(DoctorCheck::pass(format!(
                "Config file: {}",
                state.config_path.display()
            )));
            if state.config_valid_toml {
                checks.push(DoctorCheck::pass("Config is valid TOML"));
            } else {
                checks.push(DoctorCheck::fail(
                    "Config is valid TOML",
                    format!("Config is invalid TOML: {}", state.config_path.display()),
                ));
            }
        } else {
            checks.push(DoctorCheck::fail(
                "Config file",
                format!("Config file not found: {}", state.config_path.display()),
            ));
        }

        if state.library_directory_exists {
            checks.push(DoctorCheck::pass(format!(
                "Library directory: {}",
                state.library_path.display()
            )));
        } else {
            checks.push(DoctorCheck::fail(
                "Library directory",
                format!(
                    "Library directory not found: {}",
                    state.library_path.display()
                ),
            ));
        }

        checks
    }
}

struct DoctorState {
    config_path: std::path::PathBuf,
    library_path: std::path::PathBuf,
    config_file_exists: bool,
    config_valid_toml: bool,
    library_directory_exists: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
}

fn collect_doctor_state() -> DoctorState {
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let config_path = Path::new(&home).join(".config/prompter/config.toml");
    let library_path = Path::new(&home).join(".local/prompter/library");

    let config_file_exists = config_path.exists();
    let mut config_valid_toml = false;
    let mut errors = Vec::new();
    let warnings = Vec::new();

    if config_file_exists {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                if toml::from_str::<toml::Value>(&content).is_ok() {
                    config_valid_toml = true;
                } else {
                    errors.push(format!("Config is invalid TOML: {}", config_path.display()));
                }
            }
            Err(e) => {
                errors.push(format!("Failed to read config: {e}"));
            }
        }
    } else {
        errors.push(format!("Config file not found: {}", config_path.display()));
    }

    let library_directory_exists = library_path.exists();
    if !library_directory_exists {
        errors.push(format!(
            "Library directory not found: {}",
            library_path.display()
        ));
    }

    DoctorState {
        config_path,
        library_path,
        config_file_exists,
        config_valid_toml,
        library_directory_exists,
        errors,
        warnings,
    }
}

fn build_doctor_report() -> DoctorReport {
    let state = collect_doctor_state();
    let mut report = DoctorReport::for_tool(&PrompterDoctor)
        .with_detail("config_file_exists", json!(state.config_file_exists))
        .with_detail("config_valid_toml", json!(state.config_valid_toml))
        .with_detail(
            "library_directory_exists",
            json!(state.library_directory_exists),
        );

    for error in state.errors {
        report = report.with_error(error);
    }
    for warning in state.warnings {
        report = report.with_warning(warning);
    }

    report
        .with_info(format!("Current version: v{}", env!("CARGO_PKG_VERSION")))
        .with_info("Check https://github.com/tftio/prompter/releases for updates")
}

/// Run doctor command to check health and configuration.
///
/// Returns exit code: 0 if healthy, 1 if issues found.
pub fn run_doctor(json: bool) -> i32 {
    let report = build_doctor_report();
    report.emit(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_doctor_returns_valid_exit_code() {
        let exit_code = run_doctor(false);
        // Should return 0 or 1
        assert!(exit_code == 0 || exit_code == 1);
    }

    #[test]
    fn test_run_doctor_json_returns_valid_exit_code() {
        let exit_code = run_doctor(true);
        // Should return 0 or 1
        assert!(exit_code == 0 || exit_code == 1);
    }
}
