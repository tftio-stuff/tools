use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Compute a deterministic hash for a repository path.
/// Uses the absolute, canonicalized path to ensure consistency.
pub fn compute_repo_hash(repo_root: &Path) -> Result<String> {
    let canonical = repo_root
        .canonicalize()
        .with_context(|| format!("canonicalizing path: {}", repo_root.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(canonical.to_string_lossy().as_bytes());
    let result = hasher.finalize();
    Ok(hex::encode(&result[..16]))
}

/// Find the repository root using git2.
pub fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let repo = git2::Repository::discover(start)
        .with_context(|| format!("finding git repository from: {}", start.display()))?;
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("bare repository has no workdir"))?;
    Ok(workdir.to_path_buf())
}

// hex encoding helper (avoids adding the `hex` crate)
mod hex {
    use std::fmt::Write;

    pub fn encode(bytes: &[u8]) -> String {
        bytes
            .iter()
            .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
                let _ = write!(acc, "{b:02x}");
                acc
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        let h1 = compute_repo_hash(tmp.path()).unwrap();
        let h2 = compute_repo_hash(tmp.path()).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 32); // 16 bytes -> 32 hex chars
    }

    #[test]
    fn different_paths_different_hashes() {
        let t1 = tempfile::tempdir().unwrap();
        let t2 = tempfile::tempdir().unwrap();
        let h1 = compute_repo_hash(t1.path()).unwrap();
        let h2 = compute_repo_hash(t2.path()).unwrap();
        assert_ne!(h1, h2);
    }
}
