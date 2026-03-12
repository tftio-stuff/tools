//! Integration tests for unvenv - Python venv detector

use std::{fs, process::Command};
use tempfile::TempDir;

/// Helper to get the path to the compiled binary
fn get_binary_path() -> std::path::PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    // In a workspace, target/ is at the workspace root (two levels up from crates/unvenv/)
    std::path::Path::new(manifest_dir)
        .join("../../target/debug/unvenv")
}

/// Test that the binary exists and compiles
#[test]
fn test_binary_exists() {
    let output = Command::new("cargo")
        .args(["build", "--bin", "unvenv"])
        .output()
        .expect("Failed to execute cargo build");

    assert!(output.status.success(), "Failed to build binary");
}

/// Test version subcommand
#[test]
fn test_version_command() {
    let binary_path = get_binary_path();
    let output = Command::new(binary_path)
        .arg("version")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unvenv"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

/// Test built-in help flag
#[test]
fn test_help_flag() {
    let binary_path = get_binary_path();
    let output = Command::new(binary_path)
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python virtual environment detector CLI"));
}

/// Test behavior when not in a Git repository (default scan)
#[test]
fn test_no_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = get_binary_path();

    let output = Command::new(binary_path)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute binary");

    // Should exit with code 0 (success) when not in a Git repo
    assert!(output.status.success());
}

/// Test explicit scan subcommand when not in a Git repository
#[test]
fn test_scan_no_git_repo() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let binary_path = get_binary_path();

    let output = Command::new(binary_path)
        .arg("scan")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute binary");

    // Should exit with code 0 (success) when not in a Git repo
    assert!(output.status.success());
}

/// Test detection of unignored pyvenv.cfg files
#[test]
fn test_detect_unignored_venv() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    let init_output = Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repo");
    assert!(init_output.status.success());

    // Create a venv directory with pyvenv.cfg
    let venv_dir = temp_dir.path().join("venv");
    fs::create_dir(&venv_dir).expect("Failed to create venv directory");

    let pyvenv_cfg = venv_dir.join("pyvenv.cfg");
    fs::write(
        &pyvenv_cfg,
        "home = /usr/bin\nversion = 3.9.7\ninclude-system-site-packages = false\n",
    )
    .expect("Failed to write pyvenv.cfg");

    // Run unvenv - should detect the unignored file
    let binary_path = get_binary_path();
    let output = Command::new(binary_path)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute binary");

    // Should exit with code 2 (policy violation)
    assert_eq!(output.status.code(), Some(2));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WARNING"));
    assert!(stdout.contains("Python virtual environment"));
    assert!(stdout.contains("venv/pyvenv.cfg"));
}

/// Test detection with explicit scan subcommand
#[test]
fn test_scan_detect_unignored_venv() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    let init_output = Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repo");
    assert!(init_output.status.success());

    // Create a venv directory with pyvenv.cfg
    let venv_dir = temp_dir.path().join("venv");
    fs::create_dir(&venv_dir).expect("Failed to create venv directory");

    let pyvenv_cfg = venv_dir.join("pyvenv.cfg");
    fs::write(&pyvenv_cfg, "home = /usr/bin\nversion = 3.9.7\n")
        .expect("Failed to write pyvenv.cfg");

    // Run unvenv scan - should detect the unignored file
    let binary_path = get_binary_path();
    let output = Command::new(binary_path)
        .arg("scan")
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute binary");

    // Should exit with code 2 (policy violation)
    assert_eq!(output.status.code(), Some(2));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("WARNING"));
    assert!(stdout.contains("venv/pyvenv.cfg"));
}

/// Test that ignored venv files are not reported
#[test]
fn test_ignored_venv() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    let init_output = Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repo");
    assert!(init_output.status.success());

    // Create .gitignore that ignores venv/
    let gitignore = temp_dir.path().join(".gitignore");
    fs::write(&gitignore, "venv/\n").expect("Failed to write .gitignore");

    // Create a venv directory with pyvenv.cfg
    let venv_dir = temp_dir.path().join("venv");
    fs::create_dir(&venv_dir).expect("Failed to create venv directory");

    let pyvenv_cfg = venv_dir.join("pyvenv.cfg");
    fs::write(&pyvenv_cfg, "home = /usr/bin\nversion = 3.9.7\n")
        .expect("Failed to write pyvenv.cfg");

    // Run unvenv - should NOT detect the ignored file
    let binary_path = get_binary_path();
    let output = Command::new(binary_path)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute binary");

    // Should exit with code 0 (no violations found)
    assert!(output.status.success());
}

/// Test multiple venv directories
#[test]
fn test_multiple_venvs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    let init_output = Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repo");
    assert!(init_output.status.success());

    // Create multiple venv directories
    for venv_name in ["venv", ".env", "myenv"] {
        let venv_dir = temp_dir.path().join(venv_name);
        fs::create_dir(&venv_dir).expect("Failed to create venv directory");

        let pyvenv_cfg = venv_dir.join("pyvenv.cfg");
        fs::write(&pyvenv_cfg, "home = /usr/bin\nversion = 3.9.7\n")
            .expect("Failed to write pyvenv.cfg");
    }

    // Run unvenv - should detect all unignored files
    let binary_path = get_binary_path();
    let output = Command::new(binary_path)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute binary");

    // Should exit with code 2 (policy violations)
    assert_eq!(output.status.code(), Some(2));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("venv/pyvenv.cfg"));
    assert!(stdout.contains(".env/pyvenv.cfg"));
    assert!(stdout.contains("myenv/pyvenv.cfg"));
}

/// Test no issues when no venv files exist
#[test]
fn test_no_venv_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Initialize Git repository
    let init_output = Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repo");
    assert!(init_output.status.success());

    // Create some regular files but no pyvenv.cfg
    fs::write(temp_dir.path().join("README.md"), "# Test Project\n")
        .expect("Failed to write README.md");
    fs::write(temp_dir.path().join("requirements.txt"), "requests\n")
        .expect("Failed to write requirements.txt");

    // Run unvenv - should find no issues
    let binary_path = get_binary_path();
    let output = Command::new(binary_path)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute binary");

    // Should exit with code 0 (no issues)
    assert!(output.status.success());
}
