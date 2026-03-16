//! Configuration loading and workdir resolution.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Extra directory grants parsed from `.safehouse` or CLI flags.
#[derive(Debug, Default)]
pub struct ExtraDirs {
    /// Read-write directory grants.
    pub rw: Vec<PathBuf>,
    /// Read-only directory grants.
    pub ro: Vec<PathBuf>,
}

/// Sandbox policy profile loaded from a TOML file.
#[derive(Debug, Default, Deserialize)]
pub struct PolicyConfig {
    /// Directory grants.
    #[serde(default)]
    pub grants: PolicyGrants,
    /// Directory denies.
    #[serde(default)]
    pub denies: PolicyDenies,
}

/// Grant rules within a policy profile.
#[derive(Debug, Default, Deserialize)]
pub struct PolicyGrants {
    /// Read-write directory grants.
    #[serde(default)]
    pub rw: Vec<String>,
    /// Read-only directory grants.
    #[serde(default)]
    pub ro: Vec<String>,
}

/// Deny rules within a policy profile.
#[derive(Debug, Default, Deserialize)]
pub struct PolicyDenies {
    /// Paths to deny access to.
    #[serde(default)]
    pub paths: Vec<String>,
}

/// Resolve the working directory.
///
/// Resolution order:
/// 1. Explicit override (from `--workdir` flag)
/// 2. Git repository root (via `git2`)
/// 3. Physical current working directory
///
/// # Errors
/// Returns an error if the resolved path cannot be canonicalized.
pub fn resolve_workdir(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(dir) = explicit {
        return dir
            .canonicalize()
            .map_err(|e| format!("cannot resolve workdir {}: {e}", dir.display()));
    }

    // Try git root
    if let Ok(repo) = git2::Repository::discover(".") {
        if let Some(workdir) = repo.workdir() {
            if let Ok(canonical) = workdir.canonicalize() {
                return Ok(canonical);
            }
        }
    }

    std::env::current_dir()
        .and_then(|p| p.canonicalize())
        .map_err(|e| format!("cannot resolve cwd: {e}"))
}

/// Parse a `.safehouse` config file from the given directory.
///
/// Returns extra directory grants found in the file. Missing file is not
/// an error — returns empty grants.
///
/// # Format
/// ```text
/// # comment
/// add-dirs=/path/to/rw/dir
/// add-dirs-ro=/path/to/ro/dir
/// ```
#[must_use]
pub fn load_safehouse_config(workdir: &Path) -> ExtraDirs {
    let config_file = workdir.join(".safehouse");
    let mut extras = ExtraDirs::default();

    let Ok(content) = fs::read_to_string(&config_file) else {
        return extras;
    };

    for line in content.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(path) = line.strip_prefix("add-dirs=") {
            extras.rw.push(PathBuf::from(path.trim()));
        } else if let Some(path) = line.strip_prefix("add-dirs-ro=") {
            extras.ro.push(PathBuf::from(path.trim()));
        }
    }

    extras
}

/// Merge CLI-provided extra dirs with `.safehouse` config dirs.
///
/// CLI dirs are appended after config dirs (both are additive).
#[must_use]
#[allow(clippy::similar_names)]
pub fn merge_extra_dirs(
    config_extras: ExtraDirs,
    cli_rw: &[PathBuf],
    cli_ro: &[PathBuf],
) -> ExtraDirs {
    let mut merged = config_extras;
    merged.rw.extend_from_slice(cli_rw);
    merged.ro.extend_from_slice(cli_ro);
    merged
}

/// Expand a `~` prefix to the user's home directory.
///
/// If the path does not start with `~/` or the home directory cannot be
/// determined, the path is returned as-is.
#[must_use]
pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

/// Load a single named policy profile.
///
/// Resolution order:
/// 1. Project-level: `<workdir>/.gator/policies/<name>.toml`
/// 2. User-global: `~/.config/gator/policies/<name>.toml`
///
/// First match wins.
///
/// # Errors
/// Returns an error if the policy file cannot be read, is invalid TOML,
/// or is not found at either location.
pub fn load_policy(name: &str, workdir: &Path) -> Result<PolicyConfig, String> {
    let project_path = workdir.join(".gator/policies").join(format!("{name}.toml"));
    if project_path.is_file() {
        let content = fs::read_to_string(&project_path)
            .map_err(|e| format!("cannot read policy {}: {e}", project_path.display()))?;
        return toml::from_str(&content)
            .map_err(|e| format!("invalid policy {}: {e}", project_path.display()));
    }

    if let Some(home) = dirs::home_dir() {
        let global_path = home.join(".config/gator/policies").join(format!("{name}.toml"));
        if global_path.is_file() {
            let content = fs::read_to_string(&global_path)
                .map_err(|e| format!("cannot read policy {}: {e}", global_path.display()))?;
            return toml::from_str(&content)
                .map_err(|e| format!("invalid policy {}: {e}", global_path.display()));
        }
    }

    Err(format!("policy not found: {name}"))
}

/// Load multiple named policy profiles and merge their grants and denies.
///
/// Grants and denies are merged additively across all named policies.
/// Tilde expansion is applied to all paths.
///
/// # Errors
/// Returns an error if any named policy cannot be loaded.
pub fn load_policies(
    names: &[String],
    workdir: &Path,
) -> Result<(ExtraDirs, Vec<PathBuf>), String> {
    let mut extras = ExtraDirs::default();
    let mut denies = Vec::new();

    for name in names {
        let policy = load_policy(name, workdir)?;
        for p in &policy.grants.rw {
            extras.rw.push(expand_tilde(p));
        }
        for p in &policy.grants.ro {
            extras.ro.push(expand_tilde(p));
        }
        for p in &policy.denies.paths {
            denies.push(expand_tilde(p));
        }
    }

    Ok((extras, denies))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn resolve_workdir_explicit() {
        let tmp = TempDir::new().unwrap();
        let result = resolve_workdir(Some(tmp.path()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path().canonicalize().unwrap());
    }

    #[test]
    fn resolve_workdir_explicit_missing() {
        let result = resolve_workdir(Some(Path::new("/nonexistent/path/xyz")));
        assert!(result.is_err());
    }

    #[test]
    fn load_safehouse_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let extras = load_safehouse_config(tmp.path());
        assert!(extras.rw.is_empty());
        assert!(extras.ro.is_empty());
    }

    #[test]
    fn load_safehouse_with_entries() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".safehouse"),
            "# comment\nadd-dirs=/a/b\nadd-dirs-ro=/c/d\nadd-dirs=/e/f\n",
        )
        .unwrap();
        let extras = load_safehouse_config(tmp.path());
        assert_eq!(extras.rw, vec![PathBuf::from("/a/b"), PathBuf::from("/e/f")]);
        assert_eq!(extras.ro, vec![PathBuf::from("/c/d")]);
    }

    #[test]
    fn load_safehouse_ignores_comments_and_blanks() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join(".safehouse"),
            "\n# full line comment\n  \nadd-dirs=/x # inline comment\n",
        )
        .unwrap();
        let extras = load_safehouse_config(tmp.path());
        assert_eq!(extras.rw, vec![PathBuf::from("/x")]);
    }

    #[test]
    fn merge_extra_dirs_combines() {
        let config = ExtraDirs {
            rw: vec![PathBuf::from("/a")],
            ro: vec![PathBuf::from("/b")],
        };
        let merged = merge_extra_dirs(
            config,
            &[PathBuf::from("/c")],
            &[PathBuf::from("/d")],
        );
        assert_eq!(merged.rw, vec![PathBuf::from("/a"), PathBuf::from("/c")]);
        assert_eq!(merged.ro, vec![PathBuf::from("/b"), PathBuf::from("/d")]);
    }

    #[test]
    fn expand_tilde_with_home() {
        let expanded = expand_tilde("~/foo/bar");
        assert!(expanded.to_string_lossy().ends_with("/foo/bar"));
        assert!(!expanded.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn expand_tilde_absolute_unchanged() {
        let expanded = expand_tilde("/absolute/path");
        assert_eq!(expanded, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn load_policy_project_level() {
        let tmp = TempDir::new().unwrap();
        let policy_dir = tmp.path().join(".gator/policies");
        fs::create_dir_all(&policy_dir).unwrap();
        fs::write(
            policy_dir.join("test.toml"),
            "[grants]\nrw = [\"/a\"]\nro = [\"/b\"]\n\n[denies]\npaths = [\"~/.aws\"]\n",
        )
        .unwrap();

        let policy = load_policy("test", tmp.path()).unwrap();
        assert_eq!(policy.grants.rw, vec!["/a"]);
        assert_eq!(policy.grants.ro, vec!["/b"]);
        assert_eq!(policy.denies.paths, vec!["~/.aws"]);
    }

    #[test]
    fn load_policy_not_found() {
        let tmp = TempDir::new().unwrap();
        let result = load_policy("nonexistent", tmp.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn load_policies_merges_additively() {
        let tmp = TempDir::new().unwrap();
        let policy_dir = tmp.path().join(".gator/policies");
        fs::create_dir_all(&policy_dir).unwrap();
        fs::write(policy_dir.join("a.toml"), "[grants]\nrw = [\"/x\"]\n").unwrap();
        fs::write(
            policy_dir.join("b.toml"),
            "[grants]\nro = [\"/y\"]\n\n[denies]\npaths = [\"/z\"]\n",
        )
        .unwrap();

        let (extras, denies) =
            load_policies(&["a".to_owned(), "b".to_owned()], tmp.path()).unwrap();
        assert_eq!(extras.rw, vec![PathBuf::from("/x")]);
        assert_eq!(extras.ro, vec![PathBuf::from("/y")]);
        assert_eq!(denies, vec![PathBuf::from("/z")]);
    }
}
