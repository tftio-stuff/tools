//! Health check and diagnostics module.

use serde::Serialize;
use std::path::Path;

/// JSON output structure for doctor command
#[derive(Debug, Serialize)]
struct DoctorOutput {
    config_file_exists: bool,
    config_valid_toml: bool,
    library_directory_exists: bool,
    version: String,
    errors: Vec<String>,
    warnings: Vec<String>,
}

/// Run doctor command to check health and configuration with JSON support.
///
/// Returns exit code: 0 if healthy, 1 if issues found.
pub fn run_doctor_with_json(json: bool) -> i32 {
    if json {
        run_doctor_json()
    } else {
        run_doctor()
    }
}

/// Run doctor command with JSON output.
fn run_doctor_json() -> i32 {
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

    let output = DoctorOutput {
        config_file_exists,
        config_valid_toml,
        library_directory_exists,
        version: env!("CARGO_PKG_VERSION").to_string(),
        errors,
        warnings,
    };

    match serde_json::to_string_pretty(&output) {
        Ok(json_output) => {
            println!("{json_output}");
            if output.errors.is_empty() { 0 } else { 1 }
        }
        Err(e) => {
            eprintln!(r#"{{"error":"JSON serialization error: {}"}}"#, e);
            1
        }
    }
}

/// Run doctor command to check health and configuration.
///
/// Returns exit code: 0 if healthy, 1 if issues found.
fn run_doctor() -> i32 {
    println!("🏥 prompter health check");
    println!("========================");
    println!();

    let mut has_errors = false;

    // Check configuration
    println!("Configuration:");
    let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    let config_path = Path::new(&home).join(".config/prompter/config.toml");

    if config_path.exists() {
        println!("  ✅ Config file: {}", config_path.display());

        // Try to parse it
        match std::fs::read_to_string(&config_path) {
            Ok(content) => {
                if toml::from_str::<toml::Value>(&content).is_ok() {
                    println!("  ✅ Config is valid TOML");
                } else {
                    println!("  ❌ Config is invalid TOML");
                    has_errors = true;
                }
            }
            Err(e) => {
                println!("  ❌ Failed to read config: {e}");
                has_errors = true;
            }
        }
    } else {
        println!("  ❌ Config file not found: {}", config_path.display());
        println!("  ℹ️  Run 'prompter init' to create default configuration");
        has_errors = true;
    }

    // Check library directory
    let library_path = Path::new(&home).join(".local/prompter/library");

    if library_path.exists() {
        println!("  ✅ Library directory: {}", library_path.display());
    } else {
        println!(
            "  ❌ Library directory not found: {}",
            library_path.display()
        );
        println!("  ℹ️  Run 'prompter init' to create default library");
        has_errors = true;
    }

    println!();

    // Version info
    println!("Version:");
    println!("  ℹ️  Current version: v{}", env!("CARGO_PKG_VERSION"));
    println!("  💡 Check https://github.com/tftio/prompter/releases for updates");

    println!();

    // Summary
    if has_errors {
        println!("❌ Errors found");
        1
    } else {
        println!("✨ Everything looks healthy!");
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_doctor_returns_valid_exit_code() {
        let exit_code = run_doctor();
        // Should return 0 or 1
        assert!(exit_code == 0 || exit_code == 1);
    }

    #[test]
    fn test_run_doctor_json_returns_valid_exit_code() {
        let exit_code = run_doctor_json();
        // Should return 0 or 1
        assert!(exit_code == 0 || exit_code == 1);
    }
}
