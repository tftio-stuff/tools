//! Shared agent-facing documentation models and renderers.

use std::ffi::OsStr;

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
    None
}

/// Render the canonical YAML agent-help document.
#[must_use]
pub fn render_agent_help_yaml(_doc: &AgentDoc) -> String {
    String::new()
}

/// Render the Claude-style agent skill document.
#[must_use]
pub fn render_agent_skill(_doc: &AgentDoc) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
