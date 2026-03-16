//! Git worktree detection via `git2`.

use std::path::{Path, PathBuf};

/// Detected worktree topology for a working directory.
#[derive(Debug, Default)]
pub struct WorktreeInfo {
    /// Git common dir, if this is a linked worktree (needs RW grant).
    /// `None` if this is the main worktree or not a git repo.
    pub common_dir: Option<PathBuf>,
    /// Sibling worktree paths (needs RO grant). Excludes self.
    pub siblings: Vec<PathBuf>,
}

/// Detect git worktree topology for the given working directory.
///
/// Uses `git2` to discover the repository and enumerate worktrees.
/// Returns empty info if the directory is not inside a git repo.
#[must_use]
pub fn detect_worktrees(workdir: &Path) -> WorktreeInfo {
    let mut info = WorktreeInfo::default();

    let Ok(repo) = git2::Repository::discover(workdir) else {
        return info;
    };

    // Determine if this is a linked worktree by comparing git_dir to commondir
    let git_dir = repo.path().to_path_buf();
    let Ok(common_dir) = repo.commondir().canonicalize() else {
        return info;
    };

    let Ok(git_dir_canonical) = git_dir.canonicalize() else {
        return info;
    };

    // If git_dir != common_dir, this is a linked worktree
    if git_dir_canonical != common_dir {
        info.common_dir = Some(common_dir);
    }

    // Enumerate sibling worktrees
    let Ok(workdir_canonical) = workdir.canonicalize() else {
        return info;
    };

    if let Ok(worktrees) = repo.worktrees() {
        for name in worktrees.iter().flatten() {
            if let Ok(wt) = repo.find_worktree(name) {
                if let Ok(wt_path) = wt.path().canonicalize() {
                    if wt_path != workdir_canonical {
                        info.siblings.push(wt_path);
                    }
                }
            }
        }
    }

    // Also check the main worktree (not listed by repo.worktrees())
    if let Some(main_workdir) = repo.workdir() {
        if let Ok(main_canonical) = main_workdir.canonicalize() {
            if main_canonical != workdir_canonical
                && !info.siblings.contains(&main_canonical)
            {
                info.siblings.push(main_canonical);
            }
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_worktrees_non_git_dir() {
        let tmp = tempfile::TempDir::new().unwrap();
        let info = detect_worktrees(tmp.path());
        assert!(info.common_dir.is_none());
        assert!(info.siblings.is_empty());
    }

    #[test]
    fn detect_worktrees_main_worktree() {
        let tmp = tempfile::TempDir::new().unwrap();
        git2::Repository::init(tmp.path()).unwrap();
        let info = detect_worktrees(tmp.path());
        assert!(info.common_dir.is_none());
    }
}
