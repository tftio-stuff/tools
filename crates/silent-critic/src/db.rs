use anyhow::{Context, Result, bail};
use rusqlite::{Connection, params};
use std::path::Path;

use crate::models::{
    AuditEvent, Contract, ContractCriterion, ContractSandbox, Criterion, Decision,
    DiscoveryContext, Evidence, Project, Session, SessionStatus,
};

pub fn open_db(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating db directory: {}", parent.display()))?;
    }
    let conn =
        Connection::open(path).with_context(|| format!("opening database: {}", path.display()))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;",
    )?;
    Ok(conn)
}

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            repo_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS criteria (
            id TEXT PRIMARY KEY,
            namespace TEXT NOT NULL,
            name TEXT NOT NULL,
            claim TEXT NOT NULL,
            evaluator_type TEXT NOT NULL,
            check_spec TEXT NOT NULL,
            parameter_schema TEXT,
            created_at TEXT NOT NULL,
            deprecated_at TEXT
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            contract_id TEXT,
            worktree_path TEXT NOT NULL,
            status TEXT NOT NULL,
            worker_token TEXT,
            operator_token TEXT NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            FOREIGN KEY(contract_id) REFERENCES contracts(id)
        );

        CREATE TABLE IF NOT EXISTS contracts (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            goal TEXT NOT NULL,
            sandbox_json TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );

        CREATE TABLE IF NOT EXISTS contract_criteria (
            contract_id TEXT NOT NULL,
            criterion_id TEXT NOT NULL,
            visibility TEXT NOT NULL,
            base_tier TEXT NOT NULL,
            base_independence TEXT NOT NULL,
            parameters TEXT,
            residual_claim TEXT,
            PRIMARY KEY(contract_id, criterion_id),
            FOREIGN KEY(contract_id) REFERENCES contracts(id),
            FOREIGN KEY(criterion_id) REFERENCES criteria(id)
        );

        CREATE TABLE IF NOT EXISTS discovery_contexts (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_path TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            summary TEXT NOT NULL,
            gathered_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id)
        );

        CREATE TABLE IF NOT EXISTS evidence (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            criterion_id TEXT NOT NULL,
            command_run TEXT NOT NULL,
            exit_code INTEGER NOT NULL,
            stdout TEXT NOT NULL,
            stderr TEXT NOT NULL,
            effective_tier TEXT NOT NULL,
            effective_independence TEXT NOT NULL,
            recorded_at TEXT NOT NULL,
            FOREIGN KEY(session_id) REFERENCES sessions(id),
            FOREIGN KEY(criterion_id) REFERENCES criteria(id)
        );

        CREATE TABLE IF NOT EXISTS decisions (
            id TEXT PRIMARY KEY,
            contract_id TEXT NOT NULL,
            decision_type TEXT NOT NULL,
            actor TEXT NOT NULL,
            basis TEXT NOT NULL,
            evidence_refs TEXT NOT NULL DEFAULT '[]',
            resolves TEXT NOT NULL DEFAULT '[]',
            outcome TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(contract_id) REFERENCES contracts(id)
        );

        CREATE TABLE IF NOT EXISTS audit_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            contract_id TEXT,
            session_id TEXT,
            event_type TEXT NOT NULL,
            payload TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL
        );
        ",
    )?;
    Ok(())
}

// ── Project CRUD ────────────────────────────────────────────────────

pub fn insert_project(conn: &Connection, project: &Project) -> Result<()> {
    conn.execute(
        "INSERT INTO projects (id, name, repo_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            project.id,
            project.name,
            project.repo_hash,
            project.created_at
        ],
    )?;
    Ok(())
}

pub fn get_project_by_repo_hash(conn: &Connection, repo_hash: &str) -> Result<Option<Project>> {
    let mut stmt =
        conn.prepare("SELECT id, name, repo_hash, created_at FROM projects WHERE repo_hash = ?1")?;
    let mut rows = stmt.query(params![repo_hash])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            repo_hash: row.get(2)?,
            created_at: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

// ── Criterion CRUD ──────────────────────────────────────────────────

pub fn insert_criterion(conn: &Connection, c: &Criterion) -> Result<()> {
    conn.execute(
        "INSERT INTO criteria (id, namespace, name, claim, evaluator_type, check_spec, parameter_schema, created_at, deprecated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            c.id,
            c.namespace,
            c.name,
            c.claim,
            c.evaluator_type.as_str(),
            c.check_spec,
            c.parameter_schema,
            c.created_at,
            c.deprecated_at,
        ],
    )?;
    Ok(())
}

pub fn get_criterion(conn: &Connection, id: &str) -> Result<Option<Criterion>> {
    let mut stmt = conn.prepare(
        "SELECT id, namespace, name, claim, evaluator_type, check_spec, parameter_schema, created_at, deprecated_at
         FROM criteria WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_criterion(row)?))
    } else {
        Ok(None)
    }
}

pub fn list_criteria(conn: &Connection, namespace: Option<&str>) -> Result<Vec<Criterion>> {
    let mut results = Vec::new();
    if let Some(ns) = namespace {
        let mut stmt = conn.prepare(
            "SELECT id, namespace, name, claim, evaluator_type, check_spec, parameter_schema, created_at, deprecated_at
             FROM criteria WHERE namespace = ?1 AND deprecated_at IS NULL ORDER BY name",
        )?;
        let mut rows = stmt.query(params![ns])?;
        while let Some(row) = rows.next()? {
            results.push(row_to_criterion(row)?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, namespace, name, claim, evaluator_type, check_spec, parameter_schema, created_at, deprecated_at
             FROM criteria WHERE deprecated_at IS NULL ORDER BY namespace, name",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            results.push(row_to_criterion(row)?);
        }
    }
    Ok(results)
}

/// Look up a non-deprecated criterion by namespace and name.
pub fn get_criterion_by_ns_name(
    conn: &Connection,
    namespace: &str,
    name: &str,
) -> Result<Option<Criterion>> {
    let mut stmt = conn.prepare(
        "SELECT id, namespace, name, claim, evaluator_type, check_spec, parameter_schema, created_at, deprecated_at
         FROM criteria WHERE namespace = ?1 AND name = ?2 AND deprecated_at IS NULL",
    )?;
    let mut rows = stmt.query(params![namespace, name])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_criterion(row)?))
    } else {
        Ok(None)
    }
}

pub fn update_criterion(conn: &Connection, c: &Criterion) -> Result<()> {
    conn.execute(
        "UPDATE criteria SET namespace = ?2, name = ?3, claim = ?4, evaluator_type = ?5, check_spec = ?6, parameter_schema = ?7
         WHERE id = ?1",
        params![
            c.id,
            c.namespace,
            c.name,
            c.claim,
            c.evaluator_type.as_str(),
            c.check_spec,
            c.parameter_schema,
        ],
    )?;
    Ok(())
}

pub fn deprecate_criterion(conn: &Connection, id: &str, at: &str) -> Result<()> {
    conn.execute(
        "UPDATE criteria SET deprecated_at = ?2 WHERE id = ?1",
        params![id, at],
    )?;
    Ok(())
}

fn row_to_criterion(row: &rusqlite::Row<'_>) -> Result<Criterion> {
    let evaluator_type_str: String = row.get(4)?;
    let evaluator_type = evaluator_type_str
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;
    Ok(Criterion {
        id: row.get(0)?,
        namespace: row.get(1)?,
        name: row.get(2)?,
        claim: row.get(3)?,
        evaluator_type,
        check_spec: row.get(5)?,
        parameter_schema: row.get(6)?,
        created_at: row.get(7)?,
        deprecated_at: row.get(8)?,
    })
}

// ── Session CRUD ────────────────────────────────────────────────────

pub fn insert_session(conn: &Connection, s: &Session) -> Result<()> {
    conn.execute(
        "INSERT INTO sessions (id, contract_id, worktree_path, status, worker_token, operator_token, started_at, ended_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            s.id,
            s.contract_id,
            s.worktree_path,
            s.status.as_str(),
            s.worker_token,
            s.operator_token,
            s.started_at,
            s.ended_at,
        ],
    )?;
    Ok(())
}

pub fn get_session(conn: &Connection, id: &str) -> Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, contract_id, worktree_path, status, worker_token, operator_token, started_at, ended_at
         FROM sessions WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_session(row)?))
    } else {
        Ok(None)
    }
}

pub fn get_active_session(conn: &Connection) -> Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, contract_id, worktree_path, status, worker_token, operator_token, started_at, ended_at
         FROM sessions WHERE status NOT IN ('adjudicated') ORDER BY started_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_session(row)?))
    } else {
        Ok(None)
    }
}

pub fn get_session_by_token(conn: &Connection, token: &str) -> Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id, contract_id, worktree_path, status, worker_token, operator_token, started_at, ended_at
         FROM sessions WHERE worker_token = ?1 OR operator_token = ?1",
    )?;
    let mut rows = stmt.query(params![token])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_session(row)?))
    } else {
        Ok(None)
    }
}

pub fn transition_session(conn: &Connection, id: &str, new_status: &SessionStatus) -> Result<()> {
    let session =
        get_session(conn, id)?.ok_or_else(|| anyhow::anyhow!("session not found: {id}"))?;
    if !session.status.can_transition_to(new_status) {
        bail!("invalid transition: {} -> {}", session.status, new_status);
    }
    conn.execute(
        "UPDATE sessions SET status = ?2 WHERE id = ?1",
        params![id, new_status.as_str()],
    )?;
    Ok(())
}

pub fn update_session_contract(
    conn: &Connection,
    session_id: &str,
    contract_id: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET contract_id = ?2 WHERE id = ?1",
        params![session_id, contract_id],
    )?;
    Ok(())
}

pub fn update_session_worker_token(conn: &Connection, session_id: &str, token: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET worker_token = ?2 WHERE id = ?1",
        params![session_id, token],
    )?;
    Ok(())
}

pub fn end_session(conn: &Connection, id: &str, ended_at: &str) -> Result<()> {
    conn.execute(
        "UPDATE sessions SET ended_at = ?2 WHERE id = ?1",
        params![id, ended_at],
    )?;
    Ok(())
}

fn row_to_session(row: &rusqlite::Row<'_>) -> Result<Session> {
    let status_str: String = row.get(3)?;
    let status = status_str.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    Ok(Session {
        id: row.get(0)?,
        contract_id: row.get(1)?,
        worktree_path: row.get(2)?,
        status,
        worker_token: row.get(4)?,
        operator_token: row.get(5)?,
        started_at: row.get(6)?,
        ended_at: row.get(7)?,
    })
}

// ── Contract CRUD ───────────────────────────────────────────────────

pub fn insert_contract(conn: &Connection, c: &Contract) -> Result<()> {
    let sandbox_json = c
        .sandbox
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .context("serializing contract sandbox")?;
    conn.execute(
        "INSERT INTO contracts (id, session_id, goal, sandbox_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![c.id, c.session_id, c.goal, sandbox_json, c.created_at],
    )?;
    Ok(())
}

pub fn get_contract(conn: &Connection, id: &str) -> Result<Option<Contract>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, goal, sandbox_json, created_at FROM contracts WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_contract(row)?))
    } else {
        Ok(None)
    }
}

pub fn get_contract_by_session(conn: &Connection, session_id: &str) -> Result<Option<Contract>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, goal, sandbox_json, created_at FROM contracts WHERE session_id = ?1",
    )?;
    let mut rows = stmt.query(params![session_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(row_to_contract(row)?))
    } else {
        Ok(None)
    }
}

fn row_to_contract(row: &rusqlite::Row<'_>) -> Result<Contract> {
    let sandbox_json: Option<String> = row.get(3)?;
    let sandbox: Option<ContractSandbox> = sandbox_json
        .map(|j| serde_json::from_str(&j))
        .transpose()
        .context("deserializing contract sandbox")?;
    Ok(Contract {
        id: row.get(0)?,
        session_id: row.get(1)?,
        goal: row.get(2)?,
        created_at: row.get(4)?,
        sandbox,
    })
}

// ── ContractCriterion CRUD ──────────────────────────────────────────

pub fn insert_contract_criterion(conn: &Connection, cc: &ContractCriterion) -> Result<()> {
    conn.execute(
        "INSERT INTO contract_criteria (contract_id, criterion_id, visibility, base_tier, base_independence, parameters, residual_claim)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            cc.contract_id,
            cc.criterion_id,
            cc.visibility.as_str(),
            cc.base_tier.as_str(),
            cc.base_independence.as_str(),
            cc.parameters,
            cc.residual_claim,
        ],
    )?;
    Ok(())
}

pub fn list_contract_criteria(
    conn: &Connection,
    contract_id: &str,
) -> Result<Vec<ContractCriterion>> {
    let mut stmt = conn.prepare(
        "SELECT contract_id, criterion_id, visibility, base_tier, base_independence, parameters, residual_claim
         FROM contract_criteria WHERE contract_id = ?1",
    )?;
    let mut rows = stmt.query(params![contract_id])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let vis_str: String = row.get(2)?;
        let tier_str: String = row.get(3)?;
        let ind_str: String = row.get(4)?;
        results.push(ContractCriterion {
            contract_id: row.get(0)?,
            criterion_id: row.get(1)?,
            visibility: vis_str.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            base_tier: tier_str.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            base_independence: ind_str.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            parameters: row.get(5)?,
            residual_claim: row.get(6)?,
        });
    }
    Ok(results)
}

// ── DiscoveryContext CRUD ───────────────────────────────────────────

pub fn insert_discovery_context(conn: &Connection, dc: &DiscoveryContext) -> Result<()> {
    conn.execute(
        "INSERT INTO discovery_contexts (id, session_id, source_type, source_path, content_hash, summary, gathered_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            dc.id,
            dc.session_id,
            dc.source_type.as_str(),
            dc.source_path,
            dc.content_hash,
            dc.summary,
            dc.gathered_at,
        ],
    )?;
    Ok(())
}

pub fn list_discovery_contexts(
    conn: &Connection,
    session_id: &str,
) -> Result<Vec<DiscoveryContext>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, source_type, source_path, content_hash, summary, gathered_at
         FROM discovery_contexts WHERE session_id = ?1 ORDER BY gathered_at",
    )?;
    let mut rows = stmt.query(params![session_id])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let st_str: String = row.get(2)?;
        results.push(DiscoveryContext {
            id: row.get(0)?,
            session_id: row.get(1)?,
            source_type: st_str.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            source_path: row.get(3)?,
            content_hash: row.get(4)?,
            summary: row.get(5)?,
            gathered_at: row.get(6)?,
        });
    }
    Ok(results)
}

// ── Evidence CRUD ───────────────────────────────────────────────────

pub fn insert_evidence(conn: &Connection, e: &Evidence) -> Result<()> {
    conn.execute(
        "INSERT INTO evidence (id, session_id, criterion_id, command_run, exit_code, stdout, stderr, effective_tier, effective_independence, recorded_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            e.id,
            e.session_id,
            e.criterion_id,
            e.command_run,
            e.exit_code,
            e.stdout,
            e.stderr,
            e.effective_tier.as_str(),
            e.effective_independence.as_str(),
            e.recorded_at,
        ],
    )?;
    Ok(())
}

pub fn list_evidence(conn: &Connection, session_id: &str) -> Result<Vec<Evidence>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, criterion_id, command_run, exit_code, stdout, stderr, effective_tier, effective_independence, recorded_at
         FROM evidence WHERE session_id = ?1 ORDER BY recorded_at",
    )?;
    let mut rows = stmt.query(params![session_id])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let tier_str: String = row.get(7)?;
        let ind_str: String = row.get(8)?;
        results.push(Evidence {
            id: row.get(0)?,
            session_id: row.get(1)?,
            criterion_id: row.get(2)?,
            command_run: row.get(3)?,
            exit_code: row.get(4)?,
            stdout: row.get(5)?,
            stderr: row.get(6)?,
            effective_tier: tier_str.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            effective_independence: ind_str.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            recorded_at: row.get(9)?,
        });
    }
    Ok(results)
}

pub fn get_evidence_for_criterion(
    conn: &Connection,
    session_id: &str,
    criterion_id: &str,
) -> Result<Vec<Evidence>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, criterion_id, command_run, exit_code, stdout, stderr, effective_tier, effective_independence, recorded_at
         FROM evidence WHERE session_id = ?1 AND criterion_id = ?2 ORDER BY recorded_at",
    )?;
    let mut rows = stmt.query(params![session_id, criterion_id])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let tier_str: String = row.get(7)?;
        let ind_str: String = row.get(8)?;
        results.push(Evidence {
            id: row.get(0)?,
            session_id: row.get(1)?,
            criterion_id: row.get(2)?,
            command_run: row.get(3)?,
            exit_code: row.get(4)?,
            stdout: row.get(5)?,
            stderr: row.get(6)?,
            effective_tier: tier_str.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            effective_independence: ind_str.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            recorded_at: row.get(9)?,
        });
    }
    Ok(results)
}

// ── Decision CRUD ───────────────────────────────────────────────────

pub fn insert_decision(conn: &Connection, d: &Decision) -> Result<()> {
    conn.execute(
        "INSERT INTO decisions (id, contract_id, decision_type, actor, basis, evidence_refs, resolves, outcome, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            d.id,
            d.contract_id,
            d.decision_type.as_str(),
            d.actor,
            d.basis,
            d.evidence_refs,
            d.resolves,
            d.outcome,
            d.created_at,
        ],
    )?;
    Ok(())
}

pub fn list_decisions(conn: &Connection, contract_id: &str) -> Result<Vec<Decision>> {
    let mut stmt = conn.prepare(
        "SELECT id, contract_id, decision_type, actor, basis, evidence_refs, resolves, outcome, created_at
         FROM decisions WHERE contract_id = ?1 ORDER BY created_at",
    )?;
    let mut rows = stmt.query(params![contract_id])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        let dt_str: String = row.get(2)?;
        results.push(Decision {
            id: row.get(0)?,
            contract_id: row.get(1)?,
            decision_type: dt_str.parse().map_err(|e: String| anyhow::anyhow!(e))?,
            actor: row.get(3)?,
            basis: row.get(4)?,
            evidence_refs: row.get(5)?,
            resolves: row.get(6)?,
            outcome: row.get(7)?,
            created_at: row.get(8)?,
        });
    }
    Ok(results)
}

// ── AuditEvent CRUD ─────────────────────────────────────────────────

pub fn insert_audit_event(conn: &Connection, ae: &AuditEvent) -> Result<()> {
    conn.execute(
        "INSERT INTO audit_events (contract_id, session_id, event_type, payload, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            ae.contract_id,
            ae.session_id,
            ae.event_type,
            ae.payload,
            ae.created_at,
        ],
    )?;
    Ok(())
}

pub fn list_audit_events(conn: &Connection, session_id: &str) -> Result<Vec<AuditEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, contract_id, session_id, event_type, payload, created_at
         FROM audit_events WHERE session_id = ?1 ORDER BY id",
    )?;
    let mut rows = stmt.query(params![session_id])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        results.push(AuditEvent {
            id: row.get(0)?,
            contract_id: row.get(1)?,
            session_id: row.get(2)?,
            event_type: row.get(3)?,
            payload: row.get(4)?,
            created_at: row.get(5)?,
        });
    }
    Ok(results)
}

pub fn list_audit_events_by_contract(
    conn: &Connection,
    contract_id: &str,
) -> Result<Vec<AuditEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, contract_id, session_id, event_type, payload, created_at
         FROM audit_events WHERE contract_id = ?1 ORDER BY id",
    )?;
    let mut rows = stmt.query(params![contract_id])?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        results.push(AuditEvent {
            id: row.get(0)?,
            contract_id: row.get(1)?,
            session_id: row.get(2)?,
            event_type: row.get(3)?,
            payload: row.get(4)?,
            created_at: row.get(5)?,
        });
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EvaluatorType, Independence, SessionStatus, Tier};

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn schema_creation() {
        let conn = test_db();
        // Verify tables exist by querying sqlite_master
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 9);
    }

    #[test]
    fn project_roundtrip() {
        let conn = test_db();
        let p = Project {
            id: uuid::Uuid::new_v4().to_string(),
            name: "test-project".to_string(),
            repo_hash: "abc123".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        insert_project(&conn, &p).unwrap();
        let found = get_project_by_repo_hash(&conn, "abc123").unwrap().unwrap();
        assert_eq!(found.name, "test-project");
        assert_eq!(found.id, p.id);
    }

    #[test]
    fn criterion_roundtrip() {
        let conn = test_db();
        let c = Criterion {
            id: uuid::Uuid::new_v4().to_string(),
            namespace: "testing".to_string(),
            name: "unit-tests".to_string(),
            claim: "All unit tests pass".to_string(),
            evaluator_type: EvaluatorType::Automated,
            check_spec: "cargo test".to_string(),
            parameter_schema: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            deprecated_at: None,
        };
        insert_criterion(&conn, &c).unwrap();
        let found = get_criterion(&conn, &c.id).unwrap().unwrap();
        assert_eq!(found.name, "unit-tests");
        assert_eq!(found.evaluator_type, EvaluatorType::Automated);

        let listed = list_criteria(&conn, Some("testing")).unwrap();
        assert_eq!(listed.len(), 1);

        deprecate_criterion(&conn, &c.id, "2024-06-01").unwrap();
        let listed = list_criteria(&conn, Some("testing")).unwrap();
        assert_eq!(listed.len(), 0);
    }

    #[test]
    fn session_state_machine() {
        let conn = test_db();
        let s = Session {
            id: uuid::Uuid::new_v4().to_string(),
            contract_id: None,
            worktree_path: "/tmp/test".to_string(),
            status: SessionStatus::Discovering,
            worker_token: None,
            operator_token: "op-token".to_string(),
            started_at: "2024-01-01T00:00:00Z".to_string(),
            ended_at: None,
        };
        insert_session(&conn, &s).unwrap();

        // Valid transition
        transition_session(&conn, &s.id, &SessionStatus::Composing).unwrap();
        let updated = get_session(&conn, &s.id).unwrap().unwrap();
        assert_eq!(updated.status, SessionStatus::Composing);

        // Invalid transition
        let err = transition_session(&conn, &s.id, &SessionStatus::Executing);
        assert!(err.is_err());
    }

    #[test]
    fn evidence_roundtrip() {
        let conn = test_db();
        // Create session first
        let s = Session {
            id: uuid::Uuid::new_v4().to_string(),
            contract_id: None,
            worktree_path: "/tmp/test".to_string(),
            status: SessionStatus::Executing,
            worker_token: Some("wt".to_string()),
            operator_token: "op".to_string(),
            started_at: "2024-01-01T00:00:00Z".to_string(),
            ended_at: None,
        };
        insert_session(&conn, &s).unwrap();

        let c = Criterion {
            id: uuid::Uuid::new_v4().to_string(),
            namespace: "test".to_string(),
            name: "test".to_string(),
            claim: "test".to_string(),
            evaluator_type: EvaluatorType::Automated,
            check_spec: "echo ok".to_string(),
            parameter_schema: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            deprecated_at: None,
        };
        insert_criterion(&conn, &c).unwrap();

        let e = Evidence {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: s.id.clone(),
            criterion_id: c.id,
            command_run: "echo ok".to_string(),
            exit_code: 0,
            stdout: "ok\n".to_string(),
            stderr: String::new(),
            effective_tier: Tier::Must,
            effective_independence: Independence::ToolAuthored,
            recorded_at: "2024-01-01T00:00:00Z".to_string(),
        };
        insert_evidence(&conn, &e).unwrap();

        let listed = list_evidence(&conn, &s.id).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].exit_code, 0);
    }
}
