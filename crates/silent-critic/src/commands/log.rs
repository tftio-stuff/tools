use anyhow::Result;
use serde_json::json;

use crate::db;
use crate::models::ExportFormat;

pub fn run_log(conn: &rusqlite::Connection, contract_id: &str, format: &ExportFormat) -> Result<String> {
    let contract = db::get_contract(conn, contract_id)?
        .ok_or_else(|| anyhow::anyhow!("contract not found: {contract_id}"))?;

    let session = db::get_session(conn, &contract.session_id)?
        .ok_or_else(|| anyhow::anyhow!("session not found"))?;

    let contract_criteria = db::list_contract_criteria(conn, contract_id)?;
    let evidence = db::list_evidence(conn, &session.id)?;
    let decisions = db::list_decisions(conn, contract_id)?;
    let audit_events = db::list_audit_events_by_contract(conn, contract_id)?;
    let discovery = db::list_discovery_contexts(conn, &session.id)?;

    // Build criterion details
    let mut criteria_details = Vec::new();
    for cc in &contract_criteria {
        let criterion = db::get_criterion(conn, &cc.criterion_id)?;
        let criterion_evidence: Vec<_> = evidence
            .iter()
            .filter(|e| e.criterion_id == cc.criterion_id)
            .collect();

        criteria_details.push(json!({
            "criterion_id": cc.criterion_id,
            "namespace": criterion.as_ref().map(|c| c.namespace.as_str()),
            "name": criterion.as_ref().map(|c| c.name.as_str()),
            "claim": criterion.as_ref().map(|c| c.claim.as_str()),
            "visibility": cc.visibility.as_str(),
            "tier": cc.base_tier.as_str(),
            "evidence_count": criterion_evidence.len(),
            "passing": criterion_evidence.iter().all(|e| e.exit_code == 0) && !criterion_evidence.is_empty(),
        }));
    }

    match format {
        ExportFormat::Json => {
            let log = json!({
                "contract": {
                    "id": contract.id,
                    "goal": contract.goal,
                    "created_at": contract.created_at,
                },
                "session": {
                    "id": session.id,
                    "status": session.status.as_str(),
                    "worktree": session.worktree_path,
                    "started_at": session.started_at,
                    "ended_at": session.ended_at,
                },
                "criteria": criteria_details,
                "evidence": evidence.iter().map(|e| json!({
                    "id": e.id,
                    "criterion_id": e.criterion_id,
                    "command": e.command_run,
                    "exit_code": e.exit_code,
                    "tier": e.effective_tier.as_str(),
                    "independence": e.effective_independence.as_str(),
                    "recorded_at": e.recorded_at,
                })).collect::<Vec<_>>(),
                "decisions": decisions.iter().map(|d| json!({
                    "id": d.id,
                    "type": d.decision_type.as_str(),
                    "actor": d.actor,
                    "basis": d.basis,
                    "outcome": d.outcome,
                    "created_at": d.created_at,
                })).collect::<Vec<_>>(),
                "audit_trail": audit_events.iter().map(|ae| json!({
                    "event_type": ae.event_type,
                    "created_at": ae.created_at,
                })).collect::<Vec<_>>(),
            });

            Ok(serde_json::to_string_pretty(&log)?)
        }
        ExportFormat::Markdown => {
            let mut md = String::new();

            md.push_str(&format!("# Decision Log: {}\n\n", contract.id));
            md.push_str(&format!("**Goal:** {}\n\n", contract.goal));
            md.push_str(&format!("**Session:** {}\n", session.id));
            md.push_str(&format!("**Status:** {}\n", session.status));
            md.push_str(&format!("**Started:** {}\n", session.started_at));
            if let Some(ref ended) = session.ended_at {
                md.push_str(&format!("**Ended:** {ended}\n"));
            }
            md.push('\n');

            // Discovery context
            if !discovery.is_empty() {
                md.push_str("## Discovery Context\n\n");
                for dc in &discovery {
                    md.push_str(&format!(
                        "- **{}** `{}` ({})\n",
                        dc.source_type, dc.source_path, dc.gathered_at
                    ));
                }
                md.push('\n');
            }

            // Criteria
            md.push_str("## Criteria\n\n");
            md.push_str("| Name | Claim | Visibility | Tier | Evidence | Passing |\n");
            md.push_str("|------|-------|------------|------|----------|---------|\n");
            for cd in &criteria_details {
                md.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} |\n",
                    cd["name"].as_str().unwrap_or("?"),
                    cd["claim"].as_str().unwrap_or("?"),
                    cd["visibility"].as_str().unwrap_or("?"),
                    cd["tier"].as_str().unwrap_or("?"),
                    cd["evidence_count"],
                    if cd["passing"].as_bool().unwrap_or(false) {
                        "yes"
                    } else {
                        "no"
                    },
                ));
            }
            md.push('\n');

            // Evidence
            if !evidence.is_empty() {
                md.push_str("## Evidence\n\n");
                for e in &evidence {
                    md.push_str(&format!(
                        "### {} (exit code: {})\n\n",
                        e.criterion_id, e.exit_code
                    ));
                    md.push_str(&format!("**Command:** `{}`\n", e.command_run));
                    md.push_str(&format!("**Recorded:** {}\n\n", e.recorded_at));
                    if !e.stdout.is_empty() {
                        md.push_str("**stdout:**\n```\n");
                        // Truncate long output
                        let stdout = if e.stdout.len() > 2000 {
                            format!("{}... (truncated)", &e.stdout[..2000])
                        } else {
                            e.stdout.clone()
                        };
                        md.push_str(&stdout);
                        md.push_str("\n```\n\n");
                    }
                    if !e.stderr.is_empty() {
                        md.push_str("**stderr:**\n```\n");
                        let stderr = if e.stderr.len() > 2000 {
                            format!("{}... (truncated)", &e.stderr[..2000])
                        } else {
                            e.stderr.clone()
                        };
                        md.push_str(&stderr);
                        md.push_str("\n```\n\n");
                    }
                }
            }

            // Decisions
            if !decisions.is_empty() {
                md.push_str("## Decisions\n\n");
                for d in &decisions {
                    md.push_str(&format!(
                        "- **{}** by {} -- {} ({})\n",
                        d.decision_type, d.actor, d.basis, d.created_at
                    ));
                }
                md.push('\n');
            }

            Ok(md)
        }
    }
}
