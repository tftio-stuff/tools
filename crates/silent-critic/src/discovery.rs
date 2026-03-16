use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::db;
use crate::models::{DiscoveryContext, DiscoverySourceType};

/// Discover repository context for a session.
/// Gathers build system, test infrastructure, CI config, docs, and git history.
pub fn discover_repo_context(
    conn: &rusqlite::Connection,
    session_id: &str,
    worktree: &Path,
    extra_docs: &[String],
) -> Result<Vec<DiscoveryContext>> {
    let mut contexts = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    // Build system detection
    let build_files = [
        ("Cargo.toml", "rust/cargo"),
        ("package.json", "javascript/npm"),
        ("pyproject.toml", "python/pyproject"),
        ("go.mod", "go/module"),
        ("Makefile", "make"),
        ("justfile", "just"),
    ];

    for (filename, build_type) in &build_files {
        let path = worktree.join(filename);
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let dc = DiscoveryContext {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                source_type: DiscoverySourceType::File,
                source_path: filename.to_string(),
                content_hash: hash_content(&content),
                summary: serde_json::json!({
                    "type": "build_system",
                    "build_type": build_type,
                    "file": filename,
                })
                .to_string(),
                gathered_at: now.clone(),
            };
            db::insert_discovery_context(conn, &dc)?;
            contexts.push(dc);
        }
    }

    // Documentation files
    let doc_files = [
        "AGENTS.md",
        "CLAUDE.md",
        "README.md",
        ".github/CODEOWNERS",
    ];

    for filename in &doc_files {
        let path = worktree.join(filename);
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?;
            let dc = DiscoveryContext {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                source_type: DiscoverySourceType::Doc,
                source_path: filename.to_string(),
                content_hash: hash_content(&content),
                summary: serde_json::json!({
                    "type": "documentation",
                    "file": filename,
                    "size_bytes": content.len(),
                })
                .to_string(),
                gathered_at: now.clone(),
            };
            db::insert_discovery_context(conn, &dc)?;
            contexts.push(dc);
        }
    }

    // CI config detection
    let ci_configs = [
        (".github/workflows", "github_actions"),
        (".gitlab-ci.yml", "gitlab_ci"),
        (".circleci", "circleci"),
        ("Jenkinsfile", "jenkins"),
    ];

    for (path_str, ci_type) in &ci_configs {
        let path = worktree.join(path_str);
        if path.exists() {
            let dc = DiscoveryContext {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                source_type: DiscoverySourceType::CiConfig,
                source_path: path_str.to_string(),
                content_hash: hash_content(path_str),
                summary: serde_json::json!({
                    "type": "ci_config",
                    "ci_type": ci_type,
                    "path": path_str,
                })
                .to_string(),
                gathered_at: now.clone(),
            };
            db::insert_discovery_context(conn, &dc)?;
            contexts.push(dc);
        }
    }

    // Git log (recent commits)
    if let Ok(repo) = git2::Repository::discover(worktree) {
        if let Ok(mut revwalk) = repo.revwalk() {
            revwalk.push_head().ok();
            let mut commits = Vec::new();
            for oid in revwalk.take(20).flatten() {
                if let Ok(commit) = repo.find_commit(oid) {
                    commits.push(serde_json::json!({
                        "sha": oid.to_string(),
                        "message": commit.message().unwrap_or("").lines().next().unwrap_or(""),
                        "author": commit.author().name().unwrap_or("unknown"),
                    }));
                }
            }
            if !commits.is_empty() {
                let dc = DiscoveryContext {
                    id: uuid::Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    source_type: DiscoverySourceType::GitLog,
                    source_path: ".git".to_string(),
                    content_hash: hash_content(&serde_json::to_string(&commits).unwrap_or_default()),
                    summary: serde_json::json!({
                        "type": "git_log",
                        "commit_count": commits.len(),
                        "commits": commits,
                    })
                    .to_string(),
                    gathered_at: now.clone(),
                };
                db::insert_discovery_context(conn, &dc)?;
                contexts.push(dc);
            }
        }
    }

    // Operator-specified extra docs
    for doc_path in extra_docs {
        let path = if Path::new(doc_path).is_absolute() {
            std::path::PathBuf::from(doc_path)
        } else {
            worktree.join(doc_path)
        };
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("reading extra doc: {}", path.display()))?;
            let dc = DiscoveryContext {
                id: uuid::Uuid::new_v4().to_string(),
                session_id: session_id.to_string(),
                source_type: DiscoverySourceType::Doc,
                source_path: doc_path.clone(),
                content_hash: hash_content(&content),
                summary: serde_json::json!({
                    "type": "operator_doc",
                    "file": doc_path,
                    "size_bytes": content.len(),
                })
                .to_string(),
                gathered_at: now.clone(),
            };
            db::insert_discovery_context(conn, &dc)?;
            contexts.push(dc);
        }
    }

    Ok(contexts)
}

fn hash_content(content: &str) -> String {
    use std::fmt::Write;
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    result
        .iter()
        .take(16)
        .fold(String::with_capacity(32), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
}

/// Generate a human-readable summary of discovered context.
pub fn format_discovery_summary(contexts: &[DiscoveryContext]) -> String {
    let mut out = String::new();
    out.push_str("Discovery Summary\n");
    out.push_str("=================\n\n");

    let build_systems: Vec<_> = contexts
        .iter()
        .filter(|c| c.summary.contains("\"type\":\"build_system\""))
        .collect();
    if !build_systems.is_empty() {
        out.push_str("Build Systems:\n");
        for bs in &build_systems {
            if let Ok(summary) = serde_json::from_str::<serde_json::Value>(&bs.summary) {
                out.push_str(&format!(
                    "  - {} ({})\n",
                    summary["file"].as_str().unwrap_or("?"),
                    summary["build_type"].as_str().unwrap_or("?"),
                ));
            }
        }
        out.push('\n');
    }

    let docs: Vec<_> = contexts
        .iter()
        .filter(|c| {
            c.summary.contains("\"type\":\"documentation\"")
                || c.summary.contains("\"type\":\"operator_doc\"")
        })
        .collect();
    if !docs.is_empty() {
        out.push_str("Documentation:\n");
        for doc in &docs {
            if let Ok(summary) = serde_json::from_str::<serde_json::Value>(&doc.summary) {
                out.push_str(&format!(
                    "  - {}\n",
                    summary["file"].as_str().unwrap_or("?"),
                ));
            }
        }
        out.push('\n');
    }

    let ci: Vec<_> = contexts
        .iter()
        .filter(|c| c.summary.contains("\"type\":\"ci_config\""))
        .collect();
    if !ci.is_empty() {
        out.push_str("CI/CD:\n");
        for c in &ci {
            if let Ok(summary) = serde_json::from_str::<serde_json::Value>(&c.summary) {
                out.push_str(&format!(
                    "  - {} ({})\n",
                    summary["path"].as_str().unwrap_or("?"),
                    summary["ci_type"].as_str().unwrap_or("?"),
                ));
            }
        }
        out.push('\n');
    }

    let git: Vec<_> = contexts
        .iter()
        .filter(|c| c.summary.contains("\"type\":\"git_log\""))
        .collect();
    if !git.is_empty() {
        out.push_str("Recent Git History:\n");
        for g in &git {
            if let Ok(summary) = serde_json::from_str::<serde_json::Value>(&g.summary) {
                if let Some(commits) = summary["commits"].as_array() {
                    for commit in commits.iter().take(5) {
                        out.push_str(&format!(
                            "  - {} {}\n",
                            &commit["sha"].as_str().unwrap_or("?")[..8.min(
                                commit["sha"].as_str().unwrap_or("?").len()
                            )],
                            commit["message"].as_str().unwrap_or("?"),
                        ));
                    }
                    if commits.len() > 5 {
                        out.push_str(&format!("  ... and {} more\n", commits.len() - 5));
                    }
                }
            }
        }
        out.push('\n');
    }

    out
}
