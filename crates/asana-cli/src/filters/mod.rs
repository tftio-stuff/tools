//! Parsing helpers for CLI filter expressions.

use crate::{
    config::Config,
    error::Result,
    models::{ProjectFilter, ProjectSort},
};
use anyhow::{Context, anyhow};
use regex::Regex;
use std::fs;
use std::io::{BufRead, Write};
use std::path::PathBuf;

/// Parse a collection of string expressions into project filters.
///
/// # Errors
///
/// Returns an error if any filter expression is invalid or cannot be parsed.
pub fn parse_filters(expressions: &[String]) -> Result<Vec<ProjectFilter>> {
    expressions
        .iter()
        .map(|expression| parse_filter(expression))
        .collect()
}

/// Attempt to interpret a single filter expression.
///
/// # Errors
///
/// Returns an error if the filter expression is empty, invalid, or uses unsupported syntax.
pub fn parse_filter(expression: &str) -> Result<ProjectFilter> {
    let trimmed = expression.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("filter expression cannot be empty"));
    }

    if let Some(index) = trimmed.find("!=") {
        let (field, value) = trimmed.split_at(index);
        let value = &value[2..];
        return Ok(ProjectFilter::NotEquals(
            field.trim().to_string(),
            value.trim().to_string(),
        ));
    }

    if let Some(index) = trimmed.find('~') {
        let (field, pattern) = trimmed.split_at(index);
        let regex = Regex::new(pattern[1..].trim()).with_context(|| {
            format!(
                "failed to compile regex filter for field '{}'",
                field.trim()
            )
        })?;
        return Ok(ProjectFilter::Regex(field.trim().to_string(), regex));
    }

    if let Some(index) = trimmed.find('=') {
        let (field, value) = trimmed.split_at(index);
        let value = &value[1..];
        return Ok(ProjectFilter::Equals(
            field.trim().to_string(),
            value.trim().to_string(),
        ));
    }

    if let Some(index) = trimmed.find(':') {
        let (field, value) = trimmed.split_at(index);
        let value = &value[1..];
        return Ok(ProjectFilter::Contains(
            field.trim().to_string(),
            value.trim().to_string(),
        ));
    }

    Err(anyhow!(
        "unable to parse filter expression '{trimmed}'; \
         expected syntax field=value | field!=value | field~regex | field:substring"
    ))
}

/// Resolve a saved filter set from disk.
///
/// # Errors
///
/// Returns an error if the saved filter file does not exist, cannot be read, is empty, or contains invalid expressions.
pub fn load_saved_filters(config: &Config, name: &str) -> Result<Vec<ProjectFilter>> {
    let mut path = config.filters_dir();
    path.push(format!("{name}.filters"));
    if !path.exists() {
        return Err(anyhow!(
            "saved filter '{name}' not found in {}",
            path.display()
        ));
    }

    let file = fs::File::open(&path)
        .with_context(|| format!("failed to open saved filter {}", path.display()))?;
    let reader = std::io::BufReader::new(file);
    let mut expressions = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        expressions.push(trimmed.to_string());
    }

    if expressions.is_empty() {
        return Err(anyhow!(
            "saved filter '{name}' does not contain any expressions"
        ));
    }

    parse_filters(&expressions)
}

/// Translate sort expressions into strongly typed variants.
///
/// # Errors
///
/// Returns an error if the sort field is not supported (must be name, `created_at`, or `modified_at`).
pub fn parse_sort(value: Option<&str>) -> Result<Option<ProjectSort>> {
    Ok(match value {
        None => None,
        Some(raw) => {
            let normalized = raw.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "" => None,
                "name" => Some(ProjectSort::Name),
                "created_at" | "created" => Some(ProjectSort::CreatedAt),
                "modified_at" | "modified" => Some(ProjectSort::ModifiedAt),
                other => {
                    return Err(anyhow!(
                        "unsupported sort field '{other}'; \
                         supported values: name, created_at, modified_at"
                    ));
                }
            }
        }
    })
}

/// Ensure the filters directory exists, returning its path.
///
/// # Errors
///
/// Returns an error if the directory cannot be created due to insufficient permissions or other filesystem errors.
pub fn ensure_filters_dir(config: &Config) -> Result<PathBuf> {
    let dir = config.filters_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create filters directory {}", dir.display()))?;
    Ok(dir)
}

/// Persist filter expressions to the configured filters directory.
///
/// # Errors
///
/// Returns an error if the expressions list is empty, if the filters directory cannot be created, or if the file cannot be written.
pub fn save_filters(config: &Config, name: &str, expressions: &[String]) -> Result<PathBuf> {
    if expressions.is_empty() {
        return Err(anyhow!(
            "cannot save filter '{name}' without at least one filter expression"
        ));
    }

    let dir = ensure_filters_dir(config)?;
    let path = dir.join(format!("{name}.filters"));
    let mut file = fs::File::create(&path)
        .with_context(|| format!("failed to write saved filter {}", path.display()))?;
    for expression in expressions {
        writeln!(file, "{expression}")?;
    }
    Ok(path)
}
