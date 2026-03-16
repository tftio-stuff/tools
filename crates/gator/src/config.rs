//! Configuration loading and workdir resolution.

use std::fs;
use std::path::{Path, PathBuf};

/// Extra directory grants parsed from `.safehouse` or CLI flags.
#[derive(Debug, Default)]
pub struct ExtraDirs {
    /// Read-write directory grants.
    pub rw: Vec<PathBuf>,
    /// Read-only directory grants.
    pub ro: Vec<PathBuf>,
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
}
