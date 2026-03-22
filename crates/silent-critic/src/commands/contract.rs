use anyhow::Result;

use crate::db;
use crate::models::{SessionRole, Visibility};

pub struct ContractView {
    pub id: String,
    pub goal: String,
    pub criteria: Vec<ContractCriterionView>,
}

pub struct ContractCriterionView {
    pub criterion_id: String,
    pub namespace: String,
    pub name: String,
    pub claim: String,
    pub check_spec: String,
    pub visibility: Visibility,
    pub tier: String,
}

pub fn run_show(conn: &rusqlite::Connection, id: &str, role: &str) -> Result<ContractView> {
    let contract =
        db::get_contract(conn, id)?.ok_or_else(|| anyhow::anyhow!("contract not found: {id}"))?;

    let role: SessionRole = role.parse().map_err(|e: String| anyhow::anyhow!(e))?;
    let contract_criteria = db::list_contract_criteria(conn, id)?;

    let mut criteria_views = Vec::new();
    for cc in &contract_criteria {
        // Worker role can only see visible criteria
        if role == SessionRole::Worker && cc.visibility == Visibility::Hidden {
            continue;
        }

        if let Some(criterion) = db::get_criterion(conn, &cc.criterion_id)? {
            criteria_views.push(ContractCriterionView {
                criterion_id: criterion.id,
                namespace: criterion.namespace,
                name: criterion.name,
                claim: criterion.claim,
                check_spec: criterion.check_spec,
                visibility: cc.visibility.clone(),
                tier: cc.base_tier.as_str().to_string(),
            });
        }
    }

    Ok(ContractView {
        id: contract.id,
        goal: contract.goal,
        criteria: criteria_views,
    })
}
