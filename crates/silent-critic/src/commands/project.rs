use anyhow::Result;
use serde_json::json;

use crate::config::{Config, resolve_db_path};
use crate::db;
use crate::models::Project;
use crate::project::{compute_repo_hash, find_repo_root};

pub struct InitResult {
    pub project: Project,
    pub db_path: std::path::PathBuf,
}

pub fn run_init(config: &Config, name: Option<&str>) -> Result<InitResult> {
    let cwd = std::env::current_dir()?;
    let repo_root = find_repo_root(&cwd)?;
    let repo_hash = compute_repo_hash(&repo_root)?;

    let project_name = name
        .map(String::from)
        .or_else(|| config.project_name.clone())
        .or_else(|| {
            repo_root
                .file_name()
                .and_then(|s| s.to_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "unnamed".to_string());

    let db_path = resolve_db_path(config, &repo_hash)?;
    let conn = db::open_db(&db_path)?;
    db::init_db(&conn)?;

    // Check if project already exists
    if let Some(existing) = db::get_project_by_repo_hash(&conn, &repo_hash)? {
        return Ok(InitResult {
            project: existing,
            db_path,
        });
    }

    let project = Project {
        id: uuid::Uuid::new_v4().to_string(),
        name: project_name,
        repo_hash,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    db::insert_project(&conn, &project)?;

    // Record audit event
    let ae = crate::models::AuditEvent {
        id: 0,
        contract_id: None,
        session_id: None,
        event_type: "project_init".to_string(),
        payload: json!({
            "project_id": project.id,
            "project_name": project.name,
        })
        .to_string(),
        created_at: project.created_at.clone(),
    };
    db::insert_audit_event(&conn, &ae)?;

    Ok(InitResult { project, db_path })
}
