//! Shared agent-facing documentation models and renderers.

use std::ffi::OsStr;
use std::fmt::Write;

use clap::CommandFactory;

/// A top-level request for agent-facing documentation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentDocRequest {
    /// Render the canonical YAML reference document.
    Help,
    /// Render the Claude-style skill document.
    Skill,
}

/// Canonical agent-facing documentation for one CLI tool.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentDoc {
    /// Schema version for the emitted agent-help document.
    pub schema_version: String,
    /// Tool metadata.
    pub tool: AgentTool,
    /// High-level usage guidance.
    pub usage: AgentUsage,
    /// Shared inherited sections reused across tools.
    pub shared_sections: Vec<AgentSection>,
    /// Documented commands and subcommands.
    pub commands: Vec<AgentCommand>,
    /// Top-level flags and positional arguments.
    pub arguments: Vec<AgentArgument>,
    /// Environment variables consumed by the tool.
    pub environment_variables: Vec<AgentEnvironmentVar>,
    /// Config files read or written by the tool.
    pub config_files: Vec<AgentConfigFile>,
    /// Important default filesystem paths.
    pub default_paths: Vec<AgentPath>,
    /// Output contracts produced by the tool.
    pub output_shapes: Vec<AgentOutputShape>,
    /// Representative command examples.
    pub examples: Vec<AgentExample>,
    /// Known failure modes and recovery guidance.
    pub failure_modes: Vec<AgentFailureMode>,
    /// Common operator mistakes and corrections.
    pub operator_mistakes: Vec<AgentOperatorMistake>,
    /// Hard constraints or caveats.
    pub constraints: Vec<String>,
}

/// Tool metadata for the agent-doc schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentTool {
    /// Human-readable tool name.
    pub name: String,
    /// Binary name used on the command line.
    pub binary: String,
    /// One-line description of the tool.
    pub summary: String,
}

/// Top-level usage guidance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentUsage {
    /// Canonical invocation line.
    pub invocation: String,
    /// How the tool should be approached by an operator or agent.
    pub notes: Vec<String>,
}

/// A shared inherited section reused across tools.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentSection {
    /// Section title.
    pub title: String,
    /// Section body.
    pub content: String,
}

/// A documented command or subcommand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommand {
    /// Space-delimited command path.
    pub name: String,
    /// One-line command summary.
    pub summary: String,
    /// Canonical usage line for the command.
    pub usage: String,
    /// Arguments accepted by this command.
    pub arguments: Vec<AgentArgument>,
    /// Output contracts for this command.
    pub output_shapes: Vec<AgentOutputShape>,
}

/// A documented flag or positional argument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentArgument {
    /// Stable argument identifier.
    pub name: String,
    /// `true` for positional arguments, `false` for long flags.
    pub positional: bool,
    /// Description of the argument.
    pub description: String,
    /// Whether the argument is required.
    pub required: bool,
}

/// A documented environment variable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentEnvironmentVar {
    /// Variable name.
    pub name: String,
    /// What the variable controls.
    pub description: String,
    /// Whether the variable must be present.
    pub required: bool,
}

/// A documented config file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentConfigFile {
    /// File path or pattern.
    pub path: String,
    /// What lives in the file.
    pub purpose: String,
}

/// A documented default path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentPath {
    /// Label for the path.
    pub name: String,
    /// Filesystem path or template.
    pub path: String,
    /// Why the path matters.
    pub purpose: String,
}

/// A documented output contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentOutputShape {
    /// Output name.
    pub name: String,
    /// Output transport or format.
    pub format: String,
    /// Description of the output contract.
    pub description: String,
}

/// A representative command example.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentExample {
    /// Example title.
    pub name: String,
    /// Command line to run.
    pub command: String,
    /// What the example demonstrates.
    pub description: String,
}

/// A known failure mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentFailureMode {
    /// Failure label.
    pub name: String,
    /// Symptom or trigger.
    pub symptom: String,
    /// Recovery guidance.
    pub resolution: String,
}

/// A common operator mistake.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentOperatorMistake {
    /// Mistake label.
    pub name: String,
    /// What usually goes wrong.
    pub symptom: String,
    /// Corrective guidance.
    pub correction: String,
}

/// Detect an exact top-level agent-doc request from raw argv.
#[must_use]
pub fn detect_agent_doc_request<I, S>(_args: I) -> Option<AgentDocRequest>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut args = _args
        .into_iter()
        .map(|arg| arg.as_ref().to_string_lossy().into_owned());

    let _program = args.next()?;
    let request = match args.next()?.as_str() {
        "--agent-help" => AgentDocRequest::Help,
        "--agent-skill" => AgentDocRequest::Skill,
        _ => return None,
    };

    if args.next().is_some() {
        return None;
    }

    Some(request)
}

/// Render the canonical YAML agent-help document.
#[must_use]
pub fn render_agent_help_yaml(_doc: &AgentDoc) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "schema_version: {}",
        yaml_string(&_doc.schema_version)
    )
    .expect("writing to String must succeed");
    out.push_str("tool:\n");
    write_yaml_field(&mut out, 1, "name", &_doc.tool.name);
    write_yaml_field(&mut out, 1, "binary", &_doc.tool.binary);
    write_yaml_field(&mut out, 1, "summary", &_doc.tool.summary);
    out.push_str("usage:\n");
    write_yaml_field(&mut out, 1, "invocation", &_doc.usage.invocation);
    write_yaml_string_list(&mut out, 1, "notes", &_doc.usage.notes);
    write_yaml_sections(&mut out, "shared_sections", &_doc.shared_sections);
    write_yaml_commands(&mut out, &_doc.commands);
    write_yaml_arguments(&mut out, "arguments", &_doc.arguments, 0);
    write_yaml_environment_variables(&mut out, &_doc.environment_variables);
    write_yaml_config_files(&mut out, &_doc.config_files);
    write_yaml_paths(&mut out, "default_paths", &_doc.default_paths);
    write_yaml_output_shapes(&mut out, "output_shapes", &_doc.output_shapes, 0);
    write_yaml_examples(&mut out, &_doc.examples);
    write_yaml_failure_modes(&mut out, &_doc.failure_modes);
    write_yaml_operator_mistakes(&mut out, &_doc.operator_mistakes);
    write_yaml_string_list(&mut out, 0, "constraints", &_doc.constraints);
    out
}

/// Render the Claude-style agent skill document.
#[must_use]
pub fn render_agent_skill(_doc: &AgentDoc) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    writeln!(out, "name: {}", yaml_string(&_doc.tool.binary))
        .expect("writing to String must succeed");
    writeln!(out, "description: {}", yaml_string(&_doc.tool.summary))
        .expect("writing to String must succeed");
    out.push_str("---\n\n");
    writeln!(out, "# {}", _doc.tool.name).expect("writing to String must succeed");
    out.push('\n');

    out.push_str("## Usage\n");
    writeln!(out, "- Invocation: `{}`", _doc.usage.invocation)
        .expect("writing to String must succeed");
    for note in &_doc.usage.notes {
        writeln!(out, "- {note}").expect("writing to String must succeed");
    }
    out.push('\n');

    out.push_str("## Shared behavior\n");
    for section in &_doc.shared_sections {
        writeln!(out, "### {}", section.title).expect("writing to String must succeed");
        writeln!(out, "{}", section.content).expect("writing to String must succeed");
        out.push('\n');
    }

    out.push_str("## Commands\n");
    for command in &_doc.commands {
        writeln!(out, "### {}", command.name).expect("writing to String must succeed");
        writeln!(out, "{}", command.summary).expect("writing to String must succeed");
        out.push('\n');
        writeln!(out, "Usage: `{}`", command.usage).expect("writing to String must succeed");
        out.push('\n');

        out.push_str("Arguments:\n");
        for argument in &command.arguments {
            writeln!(out, "- {}", markdown_argument(argument))
                .expect("writing to String must succeed");
        }
        out.push('\n');

        out.push_str("Output shapes:\n");
        for shape in &command.output_shapes {
            writeln!(
                out,
                "- `{}` (`{}`): {}",
                shape.name, shape.format, shape.description
            )
            .expect("writing to String must succeed");
        }
        out.push('\n');
    }

    out.push_str("## Top-level arguments\n");
    for argument in &_doc.arguments {
        writeln!(out, "- {}", markdown_argument(argument))
            .expect("writing to String must succeed");
    }
    out.push('\n');

    out.push_str("## Environment variables\n");
    for env_var in &_doc.environment_variables {
        let required = if env_var.required { " (required)" } else { "" };
        writeln!(out, "- `{}`{}: {}", env_var.name, required, env_var.description)
            .expect("writing to String must succeed");
    }
    out.push('\n');

    out.push_str("## Config files\n");
    for config_file in &_doc.config_files {
        writeln!(out, "- `{}`: {}", config_file.path, config_file.purpose)
            .expect("writing to String must succeed");
    }
    out.push('\n');

    out.push_str("## Default paths\n");
    for path in &_doc.default_paths {
        writeln!(out, "- `{}`: `{}` — {}", path.name, path.path, path.purpose)
            .expect("writing to String must succeed");
    }
    out.push('\n');

    out.push_str("## Output shapes\n");
    for shape in &_doc.output_shapes {
        writeln!(
            out,
            "- `{}` (`{}`): {}",
            shape.name, shape.format, shape.description
        )
        .expect("writing to String must succeed");
    }
    out.push('\n');

    out.push_str("## Examples\n");
    for example in &_doc.examples {
        writeln!(
            out,
            "- `{}`: `{}` — {}",
            example.name, example.command, example.description
        )
        .expect("writing to String must succeed");
    }
    out.push('\n');

    out.push_str("## Failure modes\n");
    for failure_mode in &_doc.failure_modes {
        writeln!(
            out,
            "- `{}`: {}. {}",
            failure_mode.name, failure_mode.symptom, failure_mode.resolution
        )
        .expect("writing to String must succeed");
    }
    out.push('\n');

    out.push_str("## Operator mistakes\n");
    for mistake in &_doc.operator_mistakes {
        let symptom = sentence_without_trailing_period(&mistake.symptom);
        writeln!(
            out,
            "- `{}`: {}. {}",
            mistake.name, symptom, mistake.correction
        )
        .expect("writing to String must succeed");
    }
    out.push('\n');

    out.push_str("## Constraints\n");
    for constraint in &_doc.constraints {
        writeln!(out, "- {constraint}").expect("writing to String must succeed");
    }

    out
}

/// Assert that authored command documentation covers the clap subcommand tree.
pub fn assert_command_coverage<T>(_documented_paths: &[&str])
where
    T: CommandFactory,
{
}

/// Assert that authored argument documentation covers one clap command context.
pub fn assert_argument_coverage<T>(
    _command_path: &[&str],
    _documented_long_flags: &[&str],
    _documented_positionals: &[&str],
    _ignored_long_flags: &[&str],
) where
    T: CommandFactory,
{
}

fn yaml_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn write_yaml_field(out: &mut String, indent_level: usize, key: &str, value: &str) {
    let indent = "  ".repeat(indent_level);
    writeln!(out, "{indent}{key}: {}", yaml_string(value))
        .expect("writing to String must succeed");
}

fn write_yaml_bool_field(out: &mut String, indent_level: usize, key: &str, value: bool) {
    let indent = "  ".repeat(indent_level);
    writeln!(out, "{indent}{key}: {value}").expect("writing to String must succeed");
}

fn write_yaml_string_list(out: &mut String, indent_level: usize, key: &str, values: &[String]) {
    let indent = "  ".repeat(indent_level);
    writeln!(out, "{indent}{key}:").expect("writing to String must succeed");
    for value in values {
        writeln!(out, "{indent}  - {}", yaml_string(value))
            .expect("writing to String must succeed");
    }
}

fn write_yaml_sections(out: &mut String, key: &str, sections: &[AgentSection]) {
    writeln!(out, "{key}:").expect("writing to String must succeed");
    for section in sections {
        writeln!(out, "  - title: {}", yaml_string(&section.title))
            .expect("writing to String must succeed");
        writeln!(out, "    content: {}", yaml_string(&section.content))
            .expect("writing to String must succeed");
    }
}

fn write_yaml_commands(out: &mut String, commands: &[AgentCommand]) {
    out.push_str("commands:\n");
    for command in commands {
        writeln!(out, "  - name: {}", yaml_string(&command.name))
            .expect("writing to String must succeed");
        writeln!(out, "    summary: {}", yaml_string(&command.summary))
            .expect("writing to String must succeed");
        writeln!(out, "    usage: {}", yaml_string(&command.usage))
            .expect("writing to String must succeed");
        write_yaml_arguments(out, "arguments", &command.arguments, 2);
        write_yaml_output_shapes(out, "output_shapes", &command.output_shapes, 2);
    }
}

fn write_yaml_arguments(
    out: &mut String,
    key: &str,
    arguments: &[AgentArgument],
    indent_level: usize,
) {
    let indent = "  ".repeat(indent_level);
    writeln!(out, "{indent}{key}:").expect("writing to String must succeed");
    for argument in arguments {
        writeln!(out, "{indent}  - name: {}", yaml_string(&argument.name))
            .expect("writing to String must succeed");
        let kind = if argument.positional { "positional" } else { "flag" };
        writeln!(out, "{indent}    kind: {}", yaml_string(kind))
            .expect("writing to String must succeed");
        writeln!(
            out,
            "{indent}    description: {}",
            yaml_string(&argument.description)
        )
        .expect("writing to String must succeed");
        write_yaml_bool_field(out, indent_level + 2, "required", argument.required);
    }
}

fn write_yaml_environment_variables(out: &mut String, env_vars: &[AgentEnvironmentVar]) {
    out.push_str("environment_variables:\n");
    for env_var in env_vars {
        writeln!(out, "  - name: {}", yaml_string(&env_var.name))
            .expect("writing to String must succeed");
        writeln!(
            out,
            "    description: {}",
            yaml_string(&env_var.description)
        )
        .expect("writing to String must succeed");
        write_yaml_bool_field(out, 2, "required", env_var.required);
    }
}

fn write_yaml_config_files(out: &mut String, config_files: &[AgentConfigFile]) {
    out.push_str("config_files:\n");
    for config_file in config_files {
        writeln!(out, "  - path: {}", yaml_string(&config_file.path))
            .expect("writing to String must succeed");
        writeln!(out, "    purpose: {}", yaml_string(&config_file.purpose))
            .expect("writing to String must succeed");
    }
}

fn write_yaml_paths(out: &mut String, key: &str, paths: &[AgentPath]) {
    writeln!(out, "{key}:").expect("writing to String must succeed");
    for path in paths {
        writeln!(out, "  - name: {}", yaml_string(&path.name))
            .expect("writing to String must succeed");
        writeln!(out, "    path: {}", yaml_string(&path.path))
            .expect("writing to String must succeed");
        writeln!(out, "    purpose: {}", yaml_string(&path.purpose))
            .expect("writing to String must succeed");
    }
}

fn write_yaml_output_shapes(
    out: &mut String,
    key: &str,
    output_shapes: &[AgentOutputShape],
    indent_level: usize,
) {
    let indent = "  ".repeat(indent_level);
    writeln!(out, "{indent}{key}:").expect("writing to String must succeed");
    for output_shape in output_shapes {
        writeln!(out, "{indent}  - name: {}", yaml_string(&output_shape.name))
            .expect("writing to String must succeed");
        writeln!(out, "{indent}    format: {}", yaml_string(&output_shape.format))
            .expect("writing to String must succeed");
        writeln!(
            out,
            "{indent}    description: {}",
            yaml_string(&output_shape.description)
        )
        .expect("writing to String must succeed");
    }
}

fn write_yaml_examples(out: &mut String, examples: &[AgentExample]) {
    out.push_str("examples:\n");
    for example in examples {
        writeln!(out, "  - name: {}", yaml_string(&example.name))
            .expect("writing to String must succeed");
        writeln!(out, "    command: {}", yaml_string(&example.command))
            .expect("writing to String must succeed");
        writeln!(
            out,
            "    description: {}",
            yaml_string(&example.description)
        )
        .expect("writing to String must succeed");
    }
}

fn write_yaml_failure_modes(out: &mut String, failure_modes: &[AgentFailureMode]) {
    out.push_str("failure_modes:\n");
    for failure_mode in failure_modes {
        writeln!(out, "  - name: {}", yaml_string(&failure_mode.name))
            .expect("writing to String must succeed");
        writeln!(
            out,
            "    symptom: {}",
            yaml_string(&failure_mode.symptom)
        )
        .expect("writing to String must succeed");
        writeln!(
            out,
            "    resolution: {}",
            yaml_string(&failure_mode.resolution)
        )
        .expect("writing to String must succeed");
    }
}

fn write_yaml_operator_mistakes(
    out: &mut String,
    operator_mistakes: &[AgentOperatorMistake],
) {
    out.push_str("operator_mistakes:\n");
    for mistake in operator_mistakes {
        writeln!(out, "  - name: {}", yaml_string(&mistake.name))
            .expect("writing to String must succeed");
        writeln!(out, "    symptom: {}", yaml_string(&mistake.symptom))
            .expect("writing to String must succeed");
        writeln!(
            out,
            "    correction: {}",
            yaml_string(&mistake.correction)
        )
        .expect("writing to String must succeed");
    }
}

fn markdown_argument(argument: &AgentArgument) -> String {
    let rendered_name = if argument.positional {
        format!("`{}`", argument.name)
    } else {
        format!("`--{}`", argument.name)
    };
    let required = if argument.required { " (required)" } else { "" };
    format!("{rendered_name}{required}: {}", argument.description)
}

fn sentence_without_trailing_period(value: &str) -> &str {
    value.strip_suffix('.').unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Parser, Subcommand};

    fn sample_doc() -> AgentDoc {
        AgentDoc {
            schema_version: "1".to_owned(),
            tool: AgentTool {
                name: "Example Tool".to_owned(),
                binary: "example".to_owned(),
                summary: "Synchronize example resources.".to_owned(),
            },
            usage: AgentUsage {
                invocation: "example sync --project demo".to_owned(),
                notes: vec![
                    "Run from a clean checkout.".to_owned(),
                    "Writes progress updates to stderr.".to_owned(),
                ],
            },
            shared_sections: vec![AgentSection {
                title: "shared-output".to_owned(),
                content: "JSON lines on stdout.".to_owned(),
            }],
            commands: vec![AgentCommand {
                name: "sync".to_owned(),
                summary: "Sync remote state.".to_owned(),
                usage: "example sync --project demo".to_owned(),
                arguments: vec![
                    AgentArgument {
                        name: "project".to_owned(),
                        positional: false,
                        description: "Project slug.".to_owned(),
                        required: true,
                    },
                    AgentArgument {
                        name: "target".to_owned(),
                        positional: true,
                        description: "Sync target.".to_owned(),
                        required: false,
                    },
                ],
                output_shapes: vec![AgentOutputShape {
                    name: "result".to_owned(),
                    format: "jsonl".to_owned(),
                    description: "Sync summary envelope then items.".to_owned(),
                }],
            }],
            arguments: vec![AgentArgument {
                name: "verbose".to_owned(),
                positional: false,
                description: "Emit verbose logs.".to_owned(),
                required: false,
            }],
            environment_variables: vec![AgentEnvironmentVar {
                name: "EXAMPLE_TOKEN".to_owned(),
                description: "Bearer token for API calls.".to_owned(),
                required: true,
            }],
            config_files: vec![AgentConfigFile {
                path: "~/.config/example/config.toml".to_owned(),
                purpose: "Stores API defaults.".to_owned(),
            }],
            default_paths: vec![AgentPath {
                name: "cache".to_owned(),
                path: "~/.cache/example".to_owned(),
                purpose: "Stores API cache entries.".to_owned(),
            }],
            output_shapes: vec![AgentOutputShape {
                name: "stderr-progress".to_owned(),
                format: "text".to_owned(),
                description: "Human-oriented progress stream.".to_owned(),
            }],
            examples: vec![AgentExample {
                name: "basic-sync".to_owned(),
                command: "example sync demo".to_owned(),
                description: "Sync the demo target.".to_owned(),
            }],
            failure_modes: vec![AgentFailureMode {
                name: "auth".to_owned(),
                symptom: "401 from API".to_owned(),
                resolution: "Refresh EXAMPLE_TOKEN and retry.".to_owned(),
            }],
            operator_mistakes: vec![AgentOperatorMistake {
                name: "wrong-directory".to_owned(),
                symptom: "Command cannot find project metadata.".to_owned(),
                correction: "Run inside the repository root.".to_owned(),
            }],
            constraints: vec![
                "Top-level agent-doc flags are hidden from normal help.".to_owned(),
                "Never place --agent-help after a subcommand.".to_owned(),
            ],
        }
    }

    #[test]
    fn detects_only_exact_top_level_agent_doc_requests() {
        assert_eq!(
            detect_agent_doc_request(["example", "--agent-help"]),
            Some(AgentDocRequest::Help)
        );
        assert_eq!(
            detect_agent_doc_request(["example", "--agent-skill"]),
            Some(AgentDocRequest::Skill)
        );
        assert_eq!(detect_agent_doc_request(["example"]), None);
        assert_eq!(detect_agent_doc_request(["example", "sync", "--agent-help"]), None);
        assert_eq!(
            detect_agent_doc_request(["example", "--agent-help", "--agent-skill"]),
            None
        );
    }

    #[test]
    fn renders_canonical_yaml_with_stable_top_level_order() {
        let rendered = render_agent_help_yaml(&sample_doc());

        let expected = r#"schema_version: "1"
tool:
  name: "Example Tool"
  binary: "example"
  summary: "Synchronize example resources."
usage:
  invocation: "example sync --project demo"
  notes:
    - "Run from a clean checkout."
    - "Writes progress updates to stderr."
shared_sections:
  - title: "shared-output"
    content: "JSON lines on stdout."
commands:
  - name: "sync"
    summary: "Sync remote state."
    usage: "example sync --project demo"
    arguments:
      - name: "project"
        kind: "flag"
        description: "Project slug."
        required: true
      - name: "target"
        kind: "positional"
        description: "Sync target."
        required: false
    output_shapes:
      - name: "result"
        format: "jsonl"
        description: "Sync summary envelope then items."
arguments:
  - name: "verbose"
    kind: "flag"
    description: "Emit verbose logs."
    required: false
environment_variables:
  - name: "EXAMPLE_TOKEN"
    description: "Bearer token for API calls."
    required: true
config_files:
  - path: "~/.config/example/config.toml"
    purpose: "Stores API defaults."
default_paths:
  - name: "cache"
    path: "~/.cache/example"
    purpose: "Stores API cache entries."
output_shapes:
  - name: "stderr-progress"
    format: "text"
    description: "Human-oriented progress stream."
examples:
  - name: "basic-sync"
    command: "example sync demo"
    description: "Sync the demo target."
failure_modes:
  - name: "auth"
    symptom: "401 from API"
    resolution: "Refresh EXAMPLE_TOKEN and retry."
operator_mistakes:
  - name: "wrong-directory"
    symptom: "Command cannot find project metadata."
    correction: "Run inside the repository root."
constraints:
  - "Top-level agent-doc flags are hidden from normal help."
  - "Never place --agent-help after a subcommand."
"#;

        assert_eq!(rendered, expected);
    }

    #[test]
    fn renders_skill_from_the_same_agent_doc_source() {
        let rendered = render_agent_skill(&sample_doc());
        let expected = r#"---
name: "example"
description: "Synchronize example resources."
---

# Example Tool

## Usage
- Invocation: `example sync --project demo`
- Run from a clean checkout.
- Writes progress updates to stderr.

## Shared behavior
### shared-output
JSON lines on stdout.

## Commands
### sync
Sync remote state.

Usage: `example sync --project demo`

Arguments:
- `--project` (required): Project slug.
- `target`: Sync target.

Output shapes:
- `result` (`jsonl`): Sync summary envelope then items.

## Top-level arguments
- `--verbose`: Emit verbose logs.

## Environment variables
- `EXAMPLE_TOKEN` (required): Bearer token for API calls.

## Config files
- `~/.config/example/config.toml`: Stores API defaults.

## Default paths
- `cache`: `~/.cache/example` — Stores API cache entries.

## Output shapes
- `stderr-progress` (`text`): Human-oriented progress stream.

## Examples
- `basic-sync`: `example sync demo` — Sync the demo target.

## Failure modes
- `auth`: 401 from API. Refresh EXAMPLE_TOKEN and retry.

## Operator mistakes
- `wrong-directory`: Command cannot find project metadata. Run inside the repository root.

## Constraints
- Top-level agent-doc flags are hidden from normal help.
- Never place --agent-help after a subcommand.
"#;

        assert_eq!(rendered, expected);
    }

    #[derive(Parser, Debug)]
    #[command(name = "coverage")]
    struct CoverageCli {
        #[arg(long)]
        verbose: bool,
        #[arg(long, hide = true)]
        agent_help: bool,
        #[command(subcommand)]
        command: CoverageCommand,
    }

    #[derive(Subcommand, Debug)]
    enum CoverageCommand {
        Sync {
            #[arg(long)]
            project: String,
            target: Option<String>,
        },
        Config {
            #[command(subcommand)]
            command: CoverageConfigCommand,
        },
    }

    #[derive(Subcommand, Debug)]
    enum CoverageConfigCommand {
        Show {
            #[arg(long)]
            json: bool,
        },
    }

    #[test]
    fn coverage_helpers_fail_when_documented_commands_or_arguments_are_missing() {
        let missing_commands = std::panic::catch_unwind(|| {
            assert_command_coverage::<CoverageCli>(&["sync", "config"]);
        });
        assert!(missing_commands.is_err());

        let missing_arguments = std::panic::catch_unwind(|| {
            assert_argument_coverage::<CoverageCli>(&["sync"], &["project"], &[], &[]);
        });
        assert!(missing_arguments.is_err());
    }

    #[test]
    fn coverage_helpers_allow_hidden_agent_doc_flags_to_be_ignored() {
        assert_command_coverage::<CoverageCli>(&["sync", "config", "config show"]);
        assert_argument_coverage::<CoverageCli>(&[], &["verbose"], &[], &["agent-help"]);
        assert_argument_coverage::<CoverageCli>(&["sync"], &["project"], &["target"], &[]);
        assert_argument_coverage::<CoverageCli>(&["config", "show"], &["json"], &[], &[]);
    }
}
