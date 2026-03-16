use anyhow::{Result, bail};
use serde_json::json;

use crate::db;
use crate::discovery;
use crate::models::{AuditEvent, Evidence, Independence, Session, SessionStatus, Tier, Visibility};

pub fn run_new(conn: &rusqlite::Connection, worktree: &str) -> Result<Session> {
    // Check that worktree path exists
    let path = std::path::Path::new(worktree);
    if !path.exists() {
        bail!("worktree path does not exist: {worktree}");
    }

    let now = chrono::Utc::now().to_rfc3339();
    let session = Session {
        id: uuid::Uuid::new_v4().to_string(),
        contract_id: None,
        worktree_path: worktree.to_string(),
        status: SessionStatus::Discovering,
        worker_token: None,
        operator_token: generate_token(),
        started_at: now.clone(),
        ended_at: None,
    };

    db::insert_session(conn, &session)?;

    let ae = AuditEvent {
        id: 0,
        contract_id: None,
        session_id: Some(session.id.clone()),
        event_type: "session_new".to_string(),
        payload: json!({
            "worktree_path": worktree,
        })
        .to_string(),
        created_at: now,
    };
    db::insert_audit_event(conn, &ae)?;

    Ok(session)
}

pub fn run_discover(
    conn: &rusqlite::Connection,
    extra_docs: &[String],
) -> Result<Vec<crate::models::DiscoveryContext>> {
    let session = get_active_session_in_status(conn, &SessionStatus::Discovering)?;
    let worktree = std::path::Path::new(&session.worktree_path);

    let contexts = discovery::discover_repo_context(conn, &session.id, worktree, extra_docs)?;

    db::transition_session(conn, &session.id, &SessionStatus::Composing)?;

    let ae = AuditEvent {
        id: 0,
        contract_id: None,
        session_id: Some(session.id.clone()),
        event_type: "session_discover".to_string(),
        payload: json!({
            "context_count": contexts.len(),
        })
        .to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    db::insert_audit_event(conn, &ae)?;

    Ok(contexts)
}

pub fn run_status(conn: &rusqlite::Connection) -> Result<SessionStatusReport> {
    let session = db::get_active_session(conn)?
        .ok_or_else(|| anyhow::anyhow!("no active session"))?;

    let evidence = db::list_evidence(conn, &session.id)?;
    let contexts = db::list_discovery_contexts(conn, &session.id)?;

    let contract = if let Some(ref cid) = session.contract_id {
        db::get_contract(conn, cid)?
    } else {
        None
    };

    let criteria_count = if let Some(ref c) = contract {
        db::list_contract_criteria(conn, &c.id)?.len()
    } else {
        0
    };

    Ok(SessionStatusReport {
        session_id: session.id,
        status: session.status,
        worktree: session.worktree_path,
        contract_id: session.contract_id,
        goal: contract.map(|c| c.goal),
        criteria_count,
        evidence_count: evidence.len(),
        discovery_count: contexts.len(),
        started_at: session.started_at,
    })
}

pub struct SessionStatusReport {
    pub session_id: String,
    pub status: SessionStatus,
    pub worktree: String,
    pub contract_id: Option<String>,
    pub goal: Option<String>,
    pub criteria_count: usize,
    pub evidence_count: usize,
    pub discovery_count: usize,
    pub started_at: String,
}

pub fn run_end(conn: &rusqlite::Connection) -> Result<ResidualReport> {
    let session = get_active_session_in_status(conn, &SessionStatus::Executing)?;
    let now = chrono::Utc::now().to_rfc3339();
    db::end_session(conn, &session.id, &now)?;

    // Compute residuals
    let contract_id = session
        .contract_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("session has no contract"))?;
    let contract_criteria = db::list_contract_criteria(conn, contract_id)?;
    let evidence = db::list_evidence(conn, &session.id)?;

    let mut residuals = Vec::new();
    for cc in &contract_criteria {
        let criterion = db::get_criterion(conn, &cc.criterion_id)?;
        let criterion_evidence: Vec<_> = evidence
            .iter()
            .filter(|e| e.criterion_id == cc.criterion_id)
            .collect();

        if criterion_evidence.is_empty() {
            residuals.push(Residual {
                criterion_id: cc.criterion_id.clone(),
                criterion_name: criterion
                    .as_ref()
                    .map(|c| c.name.clone())
                    .unwrap_or_default(),
                reason: "no evidence collected".to_string(),
            });
        } else {
            let all_passing = criterion_evidence.iter().all(|e| e.exit_code == 0);
            if !all_passing {
                residuals.push(Residual {
                    criterion_id: cc.criterion_id.clone(),
                    criterion_name: criterion
                        .as_ref()
                        .map(|c| c.name.clone())
                        .unwrap_or_default(),
                    reason: "evidence shows failure".to_string(),
                });
            }
        }
    }

    db::transition_session(conn, &session.id, &SessionStatus::AwaitingAdjudication)?;

    let ae = AuditEvent {
        id: 0,
        contract_id: session.contract_id.clone(),
        session_id: Some(session.id.clone()),
        event_type: "session_end".to_string(),
        payload: json!({
            "residual_count": residuals.len(),
        })
        .to_string(),
        created_at: now,
    };
    db::insert_audit_event(conn, &ae)?;

    Ok(ResidualReport {
        session_id: session.id,
        contract_id: contract_id.to_string(),
        total_criteria: contract_criteria.len(),
        total_evidence: evidence.len(),
        residuals,
    })
}

pub struct Residual {
    pub criterion_id: String,
    pub criterion_name: String,
    pub reason: String,
}

pub struct ResidualReport {
    pub session_id: String,
    pub contract_id: String,
    pub total_criteria: usize,
    pub total_evidence: usize,
    pub residuals: Vec<Residual>,
}

pub fn run_manifest(conn: &rusqlite::Connection, token: &str) -> Result<ManifestReport> {
    let session = db::get_session_by_token(conn, token)?
        .ok_or_else(|| anyhow::anyhow!("invalid token"))?;

    if session.status != SessionStatus::Executing {
        bail!("session is not in executing state");
    }

    // Verify this is a worker token
    if session.worker_token.as_deref() != Some(token) {
        bail!("token is not a worker token");
    }

    let contract_id = session
        .contract_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("session has no contract"))?;

    let contract = db::get_contract(conn, contract_id)?
        .ok_or_else(|| anyhow::anyhow!("contract not found"))?;

    let contract_criteria = db::list_contract_criteria(conn, contract_id)?;

    // Only return visible criteria
    let mut visible_criteria = Vec::new();
    for cc in &contract_criteria {
        if cc.visibility == Visibility::Visible {
            if let Some(criterion) = db::get_criterion(conn, &cc.criterion_id)? {
                visible_criteria.push(ManifestCriterion {
                    id: criterion.id,
                    namespace: criterion.namespace,
                    name: criterion.name,
                    claim: criterion.claim,
                    check_spec: criterion.check_spec,
                    tier: cc.base_tier.clone(),
                });
            }
        }
    }

    Ok(ManifestReport {
        goal: contract.goal,
        criteria: visible_criteria,
    })
}

pub struct ManifestCriterion {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub claim: String,
    pub check_spec: String,
    pub tier: Tier,
}

pub struct ManifestReport {
    pub goal: String,
    pub criteria: Vec<ManifestCriterion>,
}

pub fn run_submit(
    conn: &rusqlite::Connection,
    token: &str,
    criterion_id: &str,
) -> Result<Evidence> {
    let session = db::get_session_by_token(conn, token)?
        .ok_or_else(|| anyhow::anyhow!("invalid token"))?;

    if session.status != SessionStatus::Executing {
        bail!("session is not in executing state");
    }

    if session.worker_token.as_deref() != Some(token) {
        bail!("token is not a worker token");
    }

    let contract_id = session
        .contract_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("session has no contract"))?;

    // Verify criterion is in contract
    let contract_criteria = db::list_contract_criteria(conn, contract_id)?;
    let cc = contract_criteria
        .iter()
        .find(|cc| cc.criterion_id == criterion_id)
        .ok_or_else(|| anyhow::anyhow!("criterion not in contract: {criterion_id}"))?;

    let criterion = db::get_criterion(conn, criterion_id)?
        .ok_or_else(|| anyhow::anyhow!("criterion not found: {criterion_id}"))?;

    if criterion.check_spec.is_empty() {
        bail!("criterion has no check_spec (human judgment required)");
    }

    // Execute the check_spec
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(&criterion.check_spec)
        .current_dir(&session.worktree_path)
        .output()?;

    let evidence = Evidence {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session.id.clone(),
        criterion_id: criterion_id.to_string(),
        command_run: criterion.check_spec.clone(),
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        effective_tier: cc.base_tier.clone(),
        effective_independence: Independence::ToolAuthored,
        recorded_at: chrono::Utc::now().to_rfc3339(),
    };

    db::insert_evidence(conn, &evidence)?;

    let ae = AuditEvent {
        id: 0,
        contract_id: Some(contract_id.to_string()),
        session_id: Some(session.id),
        event_type: "evidence_recorded".to_string(),
        payload: json!({
            "evidence_id": evidence.id,
            "criterion_id": criterion_id,
            "exit_code": evidence.exit_code,
        })
        .to_string(),
        created_at: evidence.recorded_at.clone(),
    };
    db::insert_audit_event(conn, &ae)?;

    Ok(evidence)
}

fn get_active_session_in_status(
    conn: &rusqlite::Connection,
    expected: &SessionStatus,
) -> Result<Session> {
    let session = db::get_active_session(conn)?
        .ok_or_else(|| anyhow::anyhow!("no active session"))?;
    if &session.status != expected {
        bail!(
            "session is in '{}' state, expected '{}'",
            session.status,
            expected
        );
    }
    Ok(session)
}

/// Result of a compose-from operation.
pub struct ComposeFromResult {
    /// The created contract.
    pub contract: crate::models::Contract,
    /// Number of criteria newly created.
    pub criteria_created: usize,
    /// Number of criteria reused from the library.
    pub criteria_reused: usize,
}

/// Create a contract from JSON input, binding criteria and transitioning to ready.
pub fn run_compose_from(
    conn: &rusqlite::Connection,
    json_input: &str,
) -> Result<ComposeFromResult> {
    let input: crate::models::ComposeFromInput = serde_json::from_str(json_input)?;
    let session = get_active_session_in_status(conn, &SessionStatus::Composing)?;

    let now = chrono::Utc::now().to_rfc3339();
    let contract_id = uuid::Uuid::new_v4().to_string();

    let contract = crate::models::Contract {
        id: contract_id.clone(),
        session_id: session.id.clone(),
        goal: input.goal,
        created_at: now.clone(),
        sandbox: input.sandbox.clone(),
    };
    db::insert_contract(conn, &contract)?;
    db::update_session_contract(conn, &session.id, &contract_id)?;

    let mut created = 0;
    let mut reused = 0;

    for c in &input.criteria {
        let criterion_id =
            if let Some(existing) = db::get_criterion_by_ns_name(conn, &c.namespace, &c.name)? {
                // Update claim/check_spec if they changed
                if existing.claim != c.claim || existing.check_spec != c.check_spec {
                    let mut updated = existing.clone();
                    updated.claim.clone_from(&c.claim);
                    updated.check_spec.clone_from(&c.check_spec);
                    updated.evaluator_type = c.evaluator_type.clone();
                    db::update_criterion(conn, &updated)?;
                }
                reused += 1;
                existing.id
            } else {
                let criterion = crate::models::Criterion {
                    id: uuid::Uuid::new_v4().to_string(),
                    namespace: c.namespace.clone(),
                    name: c.name.clone(),
                    claim: c.claim.clone(),
                    evaluator_type: c.evaluator_type.clone(),
                    check_spec: c.check_spec.clone(),
                    parameter_schema: c.parameter_schema.clone(),
                    created_at: now.clone(),
                    deprecated_at: None,
                };
                let id = criterion.id.clone();
                db::insert_criterion(conn, &criterion)?;
                created += 1;
                id
            };

        let cc = crate::models::ContractCriterion {
            contract_id: contract_id.clone(),
            criterion_id,
            visibility: c.visibility.clone(),
            base_tier: c.tier.clone(),
            base_independence: Independence::ToolAuthored,
            parameters: None,
            residual_claim: c.residual_claim.clone(),
        };
        db::insert_contract_criterion(conn, &cc)?;
    }

    db::transition_session(conn, &session.id, &SessionStatus::Ready)?;

    let ae = AuditEvent {
        id: 0,
        contract_id: Some(contract_id),
        session_id: Some(session.id),
        event_type: "session_compose_from".to_string(),
        payload: json!({
            "criteria_created": created,
            "criteria_reused": reused,
        })
        .to_string(),
        created_at: now,
    };
    db::insert_audit_event(conn, &ae)?;

    Ok(ComposeFromResult {
        contract,
        criteria_created: created,
        criteria_reused: reused,
    })
}

/// Result of the `go --prompt-only` command.
pub struct GoPromptResult {
    /// The generated worker prompt text.
    pub prompt: String,
    /// The worker token for this session.
    pub worker_token: String,
}

/// Generate a worker prompt from a ready session, transitioning to executing.
///
/// The prompt contains only visible criteria. Hidden criteria are excluded.
pub fn run_go_prompt_only(conn: &rusqlite::Connection) -> Result<GoPromptResult> {
    let session = get_active_session_in_status(conn, &SessionStatus::Ready)?;

    let contract_id = session
        .contract_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("session has no contract"))?;

    let contract = db::get_contract(conn, contract_id)?
        .ok_or_else(|| anyhow::anyhow!("contract not found"))?;

    let contract_criteria = db::list_contract_criteria(conn, contract_id)?;

    let token = generate_token();
    db::update_session_worker_token(conn, &session.id, &token)?;
    db::transition_session(conn, &session.id, &SessionStatus::Executing)?;

    // Build prompt with only visible criteria
    let mut prompt = String::new();
    prompt.push_str(&format!(
        "# Task\n\nYou are working on: {}\n\n",
        contract.goal
    ));
    prompt.push_str("# Acceptance Criteria\n\nYour work will be evaluated against these criteria:\n\n");

    let mut submit_commands = Vec::new();
    for cc in &contract_criteria {
        if cc.visibility != Visibility::Visible {
            continue;
        }
        let criterion = db::get_criterion(conn, &cc.criterion_id)?
            .ok_or_else(|| anyhow::anyhow!("criterion not found: {}", cc.criterion_id))?;

        prompt.push_str(&format!(
            "- **[{}] {}**: {}\n",
            cc.base_tier, criterion.name, criterion.claim
        ));

        if !criterion.check_spec.is_empty() {
            submit_commands.push((criterion.id.clone(), criterion.name.clone()));
        }
    }

    prompt.push_str("\n# Submitting Evidence\n\n");
    prompt.push_str(
        "When you believe you have satisfied a criterion, submit evidence by running:\n\n",
    );

    for (id, name) in &submit_commands {
        prompt.push_str(&format!(
            "```bash\nsilent-critic session submit --criterion {id}\n```\n{name}\n\n",
        ));
    }

    prompt.push_str("Each command executes a predefined check and records the result.\n");
    prompt.push_str("The `SILENT_CRITIC_TOKEN` environment variable is already set.\n\n");

    prompt.push_str("# Boundaries\n\n");
    prompt.push_str("- Do the work described in the task above\n");
    prompt.push_str("- Submit evidence when you believe criteria are satisfied\n");
    prompt.push_str(
        "- You cannot see the full acceptance surface -- there may be criteria you are not aware of\n",
    );
    prompt.push_str(
        "- Do not inspect or modify silent-critic state beyond the submit command\n",
    );

    let ae = AuditEvent {
        id: 0,
        contract_id: Some(contract_id.to_string()),
        session_id: Some(session.id),
        event_type: "session_go_prompt".to_string(),
        payload: serde_json::json!({
            "worker_token_prefix": &token[..6],
        })
        .to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    db::insert_audit_event(conn, &ae)?;

    Ok(GoPromptResult {
        prompt,
        worker_token: token,
    })
}

/// Generate an opaque session token.
fn generate_token() -> String {
    format!("sc-{}", uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ComposeFromInput, Criterion, EvaluatorType, SessionStatus};

    fn test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        db::init_db(&conn).unwrap();
        conn
    }

    /// Create a session in composing state for tests.
    fn create_composing_session(conn: &rusqlite::Connection) -> Session {
        let now = chrono::Utc::now().to_rfc3339();
        let session = Session {
            id: uuid::Uuid::new_v4().to_string(),
            contract_id: None,
            worktree_path: "/tmp/test-worktree".to_string(),
            status: SessionStatus::Discovering,
            worker_token: None,
            operator_token: generate_token(),
            started_at: now,
            ended_at: None,
        };
        db::insert_session(conn, &session).unwrap();
        db::transition_session(conn, &session.id, &SessionStatus::Composing).unwrap();
        // Re-fetch to get updated status
        db::get_session(conn, &session.id).unwrap().unwrap()
    }

    #[test]
    fn compose_from_creates_contract_and_criteria() {
        let conn = test_db();
        let session = create_composing_session(&conn);

        let input = r#"{
            "goal": "Implement feature X",
            "criteria": [
                {
                    "namespace": "testing",
                    "name": "unit-tests",
                    "claim": "All unit tests pass",
                    "evaluator_type": "automated",
                    "check_spec": "cargo test",
                    "visibility": "visible",
                    "tier": "must"
                },
                {
                    "namespace": "security",
                    "name": "no-secrets",
                    "claim": "No secrets in source",
                    "evaluator_type": "automated",
                    "check_spec": "grep -r SECRET .",
                    "visibility": "hidden",
                    "tier": "should",
                    "residual_claim": "Manual review may be needed"
                }
            ]
        }"#;

        let result = run_compose_from(&conn, input).unwrap();

        // Contract created with correct goal
        assert_eq!(result.contract.goal, "Implement feature X");
        assert_eq!(result.contract.session_id, session.id);

        // 2 criteria created, 0 reused
        assert_eq!(result.criteria_created, 2);
        assert_eq!(result.criteria_reused, 0);

        // Session is now in ready state
        let updated_session = db::get_session(&conn, &session.id).unwrap().unwrap();
        assert_eq!(updated_session.status, SessionStatus::Ready);

        // Contract linked to session
        assert_eq!(
            updated_session.contract_id.as_deref(),
            Some(result.contract.id.as_str())
        );

        // Check contract criteria
        let cc = db::list_contract_criteria(&conn, &result.contract.id).unwrap();
        assert_eq!(cc.len(), 2);

        // Verify visibility counts
        let visible_count = cc
            .iter()
            .filter(|c| c.visibility == Visibility::Visible)
            .count();
        let hidden_count = cc
            .iter()
            .filter(|c| c.visibility == Visibility::Hidden)
            .count();
        assert_eq!(visible_count, 1);
        assert_eq!(hidden_count, 1);
    }

    #[test]
    fn compose_from_reuses_existing_criteria() {
        let conn = test_db();

        // Pre-create a criterion
        let existing = Criterion {
            id: uuid::Uuid::new_v4().to_string(),
            namespace: "testing".to_string(),
            name: "unit-tests".to_string(),
            claim: "Old claim".to_string(),
            evaluator_type: EvaluatorType::Automated,
            check_spec: "old-check".to_string(),
            parameter_schema: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            deprecated_at: None,
        };
        db::insert_criterion(&conn, &existing).unwrap();

        let _session = create_composing_session(&conn);

        let input = r#"{
            "goal": "Update feature Y",
            "criteria": [
                {
                    "namespace": "testing",
                    "name": "unit-tests",
                    "claim": "Updated claim",
                    "evaluator_type": "automated",
                    "check_spec": "cargo test --all",
                    "visibility": "visible",
                    "tier": "must"
                }
            ]
        }"#;

        let result = run_compose_from(&conn, input).unwrap();

        // Reused, not duplicated
        assert_eq!(result.criteria_reused, 1);
        assert_eq!(result.criteria_created, 0);

        // Only 1 criterion in library (the original, now updated)
        let all_criteria = db::list_criteria(&conn, Some("testing")).unwrap();
        assert_eq!(all_criteria.len(), 1);

        // Claim and check_spec were updated
        let updated = db::get_criterion(&conn, &existing.id).unwrap().unwrap();
        assert_eq!(updated.claim, "Updated claim");
        assert_eq!(updated.check_spec, "cargo test --all");
    }

    #[test]
    fn compose_from_input_with_sandbox() {
        let json = r#"{
            "goal": "test goal",
            "criteria": [],
            "sandbox": {
                "workdir": "/tmp/test",
                "rw": ["/tmp/test"],
                "denies": ["~/.aws"]
            }
        }"#;
        let input: ComposeFromInput = serde_json::from_str(json).unwrap();
        assert!(input.sandbox.is_some());
        let sb = input.sandbox.unwrap();
        assert_eq!(sb.workdir, Some("/tmp/test".to_owned()));
        assert_eq!(sb.rw, vec!["/tmp/test"]);
        assert_eq!(sb.denies, vec!["~/.aws"]);
    }

    #[test]
    fn compose_from_input_without_sandbox() {
        let json = r#"{"goal": "test goal", "criteria": []}"#;
        let input: ComposeFromInput = serde_json::from_str(json).unwrap();
        assert!(input.sandbox.is_none());
    }

    #[test]
    fn compose_from_passes_sandbox_to_contract() {
        let conn = test_db();
        let _session = create_composing_session(&conn);

        let input = r#"{
            "goal": "Sandbox test",
            "criteria": [],
            "sandbox": {
                "workdir": "/tmp/work",
                "rw": ["/tmp/work"],
                "denies": ["~/.ssh"]
            }
        }"#;

        let result = run_compose_from(&conn, input).unwrap();
        assert!(result.contract.sandbox.is_some());
        let sb = result.contract.sandbox.unwrap();
        assert_eq!(sb.workdir, Some("/tmp/work".to_owned()));
        assert_eq!(sb.rw, vec!["/tmp/work"]);
        assert_eq!(sb.denies, vec!["~/.ssh"]);
    }

    #[test]
    fn go_prompt_only_generates_worker_prompt() {
        let conn = test_db();

        // Set up session through to ready state
        let session = create_composing_session(&conn);

        let input = serde_json::json!({
            "goal": "Fix the bug",
            "criteria": [{
                "namespace": "testing",
                "name": "tests-pass",
                "claim": "All tests pass",
                "evaluator_type": "automated",
                "check_spec": "cargo test",
                "visibility": "visible",
                "tier": "must"
            },
            {
                "namespace": "review",
                "name": "secret-check",
                "claim": "No secrets in code",
                "evaluator_type": "automated",
                "check_spec": "grep -r SECRET .",
                "visibility": "hidden",
                "tier": "should"
            }]
        });
        run_compose_from(&conn, &input.to_string()).unwrap();

        let result = run_go_prompt_only(&conn).unwrap();

        // Prompt should contain the goal
        assert!(result.prompt.contains("Fix the bug"));
        // Prompt should contain visible criterion
        assert!(result.prompt.contains("tests-pass"));
        assert!(result.prompt.contains("All tests pass"));
        // Prompt must NOT contain hidden criterion
        assert!(!result.prompt.contains("secret-check"));
        assert!(!result.prompt.contains("No secrets in code"));
        // Should have a worker token
        assert!(result.worker_token.starts_with("sc-"));
        // Session should be in executing state
        let s = db::get_session(&conn, &session.id).unwrap().unwrap();
        assert_eq!(s.status, SessionStatus::Executing);
    }
}
