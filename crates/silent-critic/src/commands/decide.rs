use anyhow::Result;
use serde_json::json;

use crate::db;
use crate::models::{AuditEvent, Decision, DecisionType, SessionStatus};

pub fn run_decide(
    conn: &rusqlite::Connection,
    contract_id: &str,
    decision_type: &DecisionType,
    basis: &str,
    evidence_refs: Option<&str>,
) -> Result<Decision> {
    let contract = db::get_contract(conn, contract_id)?
        .ok_or_else(|| anyhow::anyhow!("contract not found: {contract_id}"))?;

    // Verify session is in awaiting_adjudication
    let session = db::get_session(conn, &contract.session_id)?
        .ok_or_else(|| anyhow::anyhow!("session not found for contract"))?;

    if session.status != SessionStatus::AwaitingAdjudication {
        anyhow::bail!(
            "session is in '{}' state, expected 'awaiting_adjudication'",
            session.status
        );
    }

    let evidence_refs_json = evidence_refs.map_or_else(
        || "[]".to_string(),
        |refs| {
            let ids: Vec<&str> = refs.split(',').map(str::trim).collect();
            serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string())
        },
    );

    let decision = Decision {
        id: uuid::Uuid::new_v4().to_string(),
        contract_id: contract_id.to_string(),
        decision_type: decision_type.clone(),
        actor: "operator".to_string(),
        basis: basis.to_string(),
        evidence_refs: evidence_refs_json,
        resolves: "[]".to_string(),
        outcome: decision_type.as_str().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    db::insert_decision(conn, &decision)?;

    // Transition to adjudicated
    db::transition_session(conn, &session.id, &SessionStatus::Adjudicated)?;

    let ae = AuditEvent {
        id: 0,
        contract_id: Some(contract_id.to_string()),
        session_id: Some(session.id),
        event_type: "decision_recorded".to_string(),
        payload: json!({
            "decision_id": decision.id,
            "decision_type": decision.decision_type.as_str(),
            "outcome": decision.outcome,
        })
        .to_string(),
        created_at: decision.created_at.clone(),
    };
    db::insert_audit_event(conn, &ae)?;

    Ok(decision)
}
