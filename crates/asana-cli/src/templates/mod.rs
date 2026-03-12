//! Template loading utilities for project creation flows.

use crate::{
    config::Config,
    error::Result,
    models::{ProjectCreateData, ProjectTemplate},
};
use anyhow::{Context, anyhow};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const DEFAULT_TEMPLATES: &[(&str, &str)] = &[(
    "standard_project.toml",
    include_str!("templates/standard_project.toml"),
)];

/// Load all project templates from the configured templates directory.
///
/// # Errors
///
/// Returns an error if the templates directory cannot be read, if any template file cannot be loaded, or if deserialization fails.
pub fn load_project_templates(config: &Config) -> Result<Vec<ProjectTemplate>> {
    install_default_templates(config)?;
    let dir = config.templates_dir();
    let mut templates = Vec::new();
    for entry in fs::read_dir(&dir)
        .with_context(|| format!("failed to read templates directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !is_template_file(&path) {
            continue;
        }
        let template = load_template_file(&path)?;
        templates.push(template);
    }
    Ok(templates)
}

/// Attempt to find a template by logical name or file path.
///
/// # Errors
///
/// Returns an error if the template cannot be found, if the file cannot be read, or if deserialization fails.
pub fn resolve_project_template(config: &Config, identifier: &str) -> Result<ProjectTemplate> {
    let candidate = Path::new(identifier);
    if candidate.exists() {
        return load_template_file(candidate);
    }
    install_default_templates(config)?;
    let templates = load_project_templates(config)?;
    let needle = normalize(identifier);
    templates
        .into_iter()
        .find(|template| {
            normalize(&template.name) == needle || matches_file_stem(template, &needle)
        })
        .ok_or_else(|| {
            anyhow!(
                "template '{identifier}' not found; place templates in {}",
                config.templates_dir().display()
            )
        })
}

fn install_default_templates(config: &Config) -> Result<()> {
    let dir = ensure_templates_dir(config)?;
    install_defaults_into(&dir)
}

fn install_defaults_into(dir: &Path) -> Result<()> {
    for (filename, contents) in DEFAULT_TEMPLATES {
        let path = dir.join(filename);
        if !path.exists() {
            fs::write(&path, contents)
                .with_context(|| format!("failed to write default template {}", path.display()))?;
        }
    }
    Ok(())
}

fn load_template_file(path: &Path) -> Result<ProjectTemplate> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read template {}", path.display()))?;
    let mut template: ProjectTemplate = toml::from_str(&contents)
        .with_context(|| format!("failed to parse template {}", path.display()))?;
    if template.project.name.is_empty() {
        template.project.name = template.name.clone();
    }
    template.source = Some(path.to_path_buf());
    Ok(template)
}

fn matches_file_stem(template: &ProjectTemplate, needle: &str) -> bool {
    if normalize(&template.name) == needle
        || template.tags.iter().any(|tag| normalize(tag) == needle)
    {
        return true;
    }

    template
        .source
        .as_ref()
        .and_then(|path| path.file_stem())
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| normalize(stem) == needle)
}

fn is_template_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "toml" | "template" | "tpl"))
}

fn ensure_templates_dir(config: &Config) -> Result<PathBuf> {
    let dir = config.templates_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create templates directory {}", dir.display()))?;
    Ok(dir)
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_'))
        .collect()
}

/// Apply variable substitutions to a project creation payload.
#[must_use]
pub fn apply_template_variables(
    mut data: ProjectCreateData,
    vars: &BTreeMap<String, String>,
) -> ProjectCreateData {
    if vars.is_empty() {
        return data;
    }

    let substitute_option = |value: &mut Option<String>| {
        if let Some(inner) = value {
            *inner = substitute(inner, vars);
        }
    };

    data.name = substitute(&data.name, vars);
    substitute_option(&mut data.workspace);
    substitute_option(&mut data.team);
    substitute_option(&mut data.notes);
    substitute_option(&mut data.color);
    substitute_option(&mut data.start_on);
    substitute_option(&mut data.due_on);
    substitute_option(&mut data.owner);

    if !data.members.is_empty() {
        data.members = data
            .members
            .into_iter()
            .map(|member| substitute(&member, vars))
            .collect();
    }

    if !data.custom_fields.is_empty() {
        data.custom_fields = data
            .custom_fields
            .into_iter()
            .map(|(key, value)| {
                let replaced_value = match value {
                    serde_json::Value::String(string) => {
                        serde_json::Value::String(substitute(&string, vars))
                    }
                    other => other,
                };
                (substitute(&key, vars), replaced_value)
            })
            .collect();
    }

    data
}

fn substitute(input: &str, vars: &BTreeMap<String, String>) -> String {
    let mut result = input.to_string();
    for (key, value) in vars {
        let token = format!("{{{{{key}}}}}");
        result = result.replace(&token, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn substitutes_placeholders() {
        let mut data = ProjectCreateData {
            name: "{{project_name}}".into(),
            notes: Some("Owned by {{owner}}".into()),
            members: vec!["{{owner}}".into()],
            ..ProjectCreateData::default()
        };
        let mut vars = BTreeMap::new();
        vars.insert("project_name".into(), "Alpha".into());
        vars.insert("owner".into(), "owner@example.com".into());
        data = apply_template_variables(data, &vars);
        assert_eq!(data.name, "Alpha");
        assert_eq!(data.notes.as_deref(), Some("Owned by owner@example.com"));
        assert_eq!(data.members, vec!["owner@example.com"]);
    }

    #[test]
    fn writes_default_template_files() {
        let temp = TempDir::new().unwrap();
        install_defaults_into(temp.path()).unwrap();
        assert!(temp.path().join("standard_project.toml").exists());
    }
}
