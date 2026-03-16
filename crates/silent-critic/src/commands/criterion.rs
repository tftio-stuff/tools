use anyhow::{Result, bail};

use crate::db;
use crate::models::{Criterion, CriterionToml, EvaluatorType};

pub fn run_create(
    conn: &rusqlite::Connection,
    namespace: Option<&str>,
    name: Option<&str>,
    claim: Option<&str>,
    evaluator_type: Option<&EvaluatorType>,
    check_spec: Option<&str>,
    parameter_schema: Option<&str>,
) -> Result<Criterion> {
    let namespace = namespace.ok_or_else(|| anyhow::anyhow!("--namespace required"))?;
    let name = name.ok_or_else(|| anyhow::anyhow!("--name required"))?;
    let claim = claim.ok_or_else(|| anyhow::anyhow!("--claim required"))?;
    let evaluator_type = evaluator_type.ok_or_else(|| anyhow::anyhow!("--evaluator-type required"))?;
    let check_spec = check_spec.unwrap_or("");

    let criterion = Criterion {
        id: uuid::Uuid::new_v4().to_string(),
        namespace: namespace.to_string(),
        name: name.to_string(),
        claim: claim.to_string(),
        evaluator_type: evaluator_type.clone(),
        check_spec: check_spec.to_string(),
        parameter_schema: parameter_schema.map(String::from),
        created_at: chrono::Utc::now().to_rfc3339(),
        deprecated_at: None,
    };

    db::insert_criterion(conn, &criterion)?;
    Ok(criterion)
}

pub fn run_list(
    conn: &rusqlite::Connection,
    namespace: Option<&str>,
) -> Result<Vec<Criterion>> {
    db::list_criteria(conn, namespace)
}

pub fn run_show(conn: &rusqlite::Connection, id: &str) -> Result<Criterion> {
    db::get_criterion(conn, id)?
        .ok_or_else(|| anyhow::anyhow!("criterion not found: {id}"))
}

pub fn run_update(
    conn: &rusqlite::Connection,
    id: &str,
    namespace: Option<&str>,
    name: Option<&str>,
    claim: Option<&str>,
    evaluator_type: Option<&EvaluatorType>,
    check_spec: Option<&str>,
) -> Result<Criterion> {
    let mut criterion = db::get_criterion(conn, id)?
        .ok_or_else(|| anyhow::anyhow!("criterion not found: {id}"))?;

    if let Some(v) = namespace {
        criterion.namespace = v.to_string();
    }
    if let Some(v) = name {
        criterion.name = v.to_string();
    }
    if let Some(v) = claim {
        criterion.claim = v.to_string();
    }
    if let Some(v) = evaluator_type {
        criterion.evaluator_type = v.clone();
    }
    if let Some(v) = check_spec {
        criterion.check_spec = v.to_string();
    }

    db::update_criterion(conn, &criterion)?;
    Ok(criterion)
}

pub fn run_deprecate(conn: &rusqlite::Connection, id: &str) -> Result<()> {
    let criterion = db::get_criterion(conn, id)?
        .ok_or_else(|| anyhow::anyhow!("criterion not found: {id}"))?;
    if criterion.deprecated_at.is_some() {
        bail!("criterion already deprecated: {id}");
    }
    let now = chrono::Utc::now().to_rfc3339();
    db::deprecate_criterion(conn, id, &now)
}

pub fn run_export(conn: &rusqlite::Connection, id: &str) -> Result<String> {
    let criterion = db::get_criterion(conn, id)?
        .ok_or_else(|| anyhow::anyhow!("criterion not found: {id}"))?;
    let toml_repr = CriterionToml::from(&criterion);
    Ok(toml::to_string_pretty(&toml_repr)?)
}

pub fn run_import(conn: &rusqlite::Connection, file: &str) -> Result<Criterion> {
    let contents = std::fs::read_to_string(file)?;
    let toml_repr: CriterionToml = toml::from_str(&contents)?;

    let criterion = Criterion {
        id: uuid::Uuid::new_v4().to_string(),
        namespace: toml_repr.namespace,
        name: toml_repr.name,
        claim: toml_repr.claim,
        evaluator_type: toml_repr.evaluator_type,
        check_spec: toml_repr.check_spec,
        parameter_schema: toml_repr.parameter_schema,
        created_at: chrono::Utc::now().to_rfc3339(),
        deprecated_at: None,
    };

    db::insert_criterion(conn, &criterion)?;
    Ok(criterion)
}
