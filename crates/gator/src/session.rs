//! Silent-critic session integration.
//!
//! Shells out to `silent-critic session sandbox <id> --format json`
//! to get the complete sandbox specification from the contract.

use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;

use crate::config::ExtraDirs;

/// Sandbox specification returned by silent-critic.
#[derive(Debug, Deserialize)]
pub struct SessionSandbox {
    /// Working directory.
    pub workdir: PathBuf,
    /// Directory grants.
    #[serde(default)]
    pub grants: SessionGrants,
    /// Paths to deny.
    #[serde(default)]
    pub denies: Vec<String>,
}

/// Grant section from the session sandbox response.
#[derive(Debug, Default, Deserialize)]
pub struct SessionGrants {
    /// Read-write directories.
    #[serde(default)]
    pub rw: Vec<String>,
    /// Read-only directories.
    #[serde(default)]
    pub ro: Vec<String>,
}

/// Query silent-critic for the sandbox specification of a session.
///
/// Runs `silent-critic session sandbox <id> --format json` and parses
/// the JSON response.
///
/// # Errors
/// Returns an error if silent-critic is not installed, the session ID
/// is invalid, or the output cannot be parsed.
pub fn fetch_session_sandbox(session_id: &str) -> Result<SessionSandbox, String> {
    let output = Command::new("silent-critic")
        .args(["session", "sandbox", session_id, "--format", "json"])
        .output()
        .map_err(|e| format!("cannot run silent-critic: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "silent-critic session sandbox failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("invalid UTF-8 from silent-critic: {e}"))?;

    serde_json::from_str(&stdout).map_err(|e| format!("cannot parse silent-critic output: {e}"))
}

/// Convert a `SessionSandbox` into gator's internal types.
///
/// Expands tilde in all paths.
#[must_use]
pub fn into_sandbox_parts(sandbox: SessionSandbox) -> (PathBuf, ExtraDirs, Vec<PathBuf>) {
    let workdir = sandbox.workdir;

    let mut extras = ExtraDirs::default();
    for p in &sandbox.grants.rw {
        extras.rw.push(crate::config::expand_tilde(p));
    }
    for p in &sandbox.grants.ro {
        extras.ro.push(crate::config::expand_tilde(p));
    }

    let denies: Vec<PathBuf> = sandbox
        .denies
        .iter()
        .map(|p| crate::config::expand_tilde(p))
        .collect();

    (workdir, extras, denies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_session_sandbox_json() {
        let json = r#"{
            "workdir": "/Users/jfb/Projects/test",
            "grants": {
                "rw": ["/Users/jfb/Projects/test"],
                "ro": ["/Users/jfb/Projects/other"]
            },
            "denies": ["~/.aws"]
        }"#;

        let sandbox: SessionSandbox = serde_json::from_str(json).unwrap();
        assert_eq!(sandbox.workdir, PathBuf::from("/Users/jfb/Projects/test"));
        assert_eq!(sandbox.grants.rw, vec!["/Users/jfb/Projects/test"]);
        assert_eq!(sandbox.grants.ro, vec!["/Users/jfb/Projects/other"]);
        assert_eq!(sandbox.denies, vec!["~/.aws"]);
    }

    #[test]
    fn into_sandbox_parts_expands_tilde() {
        let sandbox = SessionSandbox {
            workdir: PathBuf::from("/work"),
            grants: SessionGrants {
                rw: vec!["/a".to_owned()],
                ro: vec!["~/b".to_owned()],
            },
            denies: vec!["~/.aws".to_owned()],
        };

        let (workdir, extras, denies) = into_sandbox_parts(sandbox);
        assert_eq!(workdir, PathBuf::from("/work"));
        assert_eq!(extras.rw, vec![PathBuf::from("/a")]);
        assert!(!extras.ro[0].to_string_lossy().starts_with('~'));
        assert!(!denies[0].to_string_lossy().starts_with('~'));
    }

    #[test]
    fn parse_session_sandbox_minimal() {
        let json = r#"{"workdir": "/tmp"}"#;
        let sandbox: SessionSandbox = serde_json::from_str(json).unwrap();
        assert_eq!(sandbox.workdir, PathBuf::from("/tmp"));
        assert!(sandbox.grants.rw.is_empty());
        assert!(sandbox.denies.is_empty());
    }
}
