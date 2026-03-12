use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct ProjectFile {
    pub project: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedProject {
    pub name: String,
    pub key: String,
}

pub fn find_project_file(start: &Path, home: &Path) -> anyhow::Result<Option<PathBuf>> {
    let mut cur = start;
    loop {
        let candidate = cur.join(".todoer.toml");
        if candidate.exists() {
            return Ok(Some(candidate));
        }
        if cur == home {
            break;
        }
        match cur.parent() {
            Some(parent) => cur = parent,
            None => break,
        }
    }
    Ok(None)
}

pub fn load_project_name(path: &Path) -> anyhow::Result<String> {
    let contents = std::fs::read_to_string(path)?;
    let pf: ProjectFile = toml::from_str(&contents)?;
    if pf.project.trim().is_empty() {
        anyhow::bail!("project name empty");
    }
    Ok(pf.project)
}

pub fn resolve_project(
    project_override: Option<&str>,
    discovered: Option<String>,
    cwd: &Path,
    home: &Path,
    git_name: Option<&str>,
) -> anyhow::Result<ResolvedProject> {
    if let Some(p) = project_override {
        let key = crate::models::normalize_project_key(p);
        return Ok(ResolvedProject {
            name: p.to_string(),
            key,
        });
    }
    if let Some(p) = discovered {
        let key = crate::models::normalize_project_key(&p);
        return Ok(ResolvedProject { name: p, key });
    }
    if let Some(path) = find_project_file(cwd, home)? {
        let name = load_project_name(&path)?;
        let key = crate::models::normalize_project_key(&name);
        return Ok(ResolvedProject { name, key });
    }
    if let Some(g) = git_name {
        let key = crate::models::normalize_project_key(g);
        return Ok(ResolvedProject {
            name: g.to_string(),
            key,
        });
    }
    anyhow::bail!("no project")
}

pub fn resolve_init_project(
    project_override: Option<&str>,
    cwd: &Path,
    home: &Path,
    git_name: Option<&str>,
) -> anyhow::Result<ResolvedProject> {
    if let Some(p) = project_override {
        let key = crate::models::normalize_project_key(p);
        return Ok(ResolvedProject {
            name: p.to_string(),
            key,
        });
    }
    if let Some(path) = find_project_file(cwd, home)? {
        let name = load_project_name(&path)?;
        let key = crate::models::normalize_project_key(&name);
        return Ok(ResolvedProject { name, key });
    }
    if let Some(g) = git_name {
        let key = crate::models::normalize_project_key(g);
        return Ok(ResolvedProject {
            name: g.to_string(),
            key,
        });
    }
    anyhow::bail!("no project")
}
