//! Gator CLI entrypoint.

use clap::Parser;
use gator::cli::Cli;
use tftio_cli_common::{
    AgentArgument, AgentCommand, AgentConfigFile, AgentDoc, AgentDocRequest,
    AgentEnvironmentVar, AgentExample, AgentFailureMode, AgentOperatorMistake, AgentOutputShape,
    AgentPath, AgentSection, AgentTool, AgentUsage, detect_agent_doc_request,
    render_agent_help_yaml, render_agent_skill,
};

fn main() {
    let raw_args = std::env::args_os().collect::<Vec<_>>();
    if let Some(request) = detect_agent_doc_request(&raw_args) {
        print_agent_doc(request);
        std::process::exit(0);
    }

    let cli = Cli::parse_from(raw_args);
    let json = cli.json;

    if let Err(e) = cli.validate() {
        if json {
            eprintln!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\""));
        } else {
            eprintln!("gator: {e}");
        }
        std::process::exit(1);
    }

    if let Err(e) = gator::run(&cli) {
        if json {
            eprintln!(r#"{{"error":"{}"}}"#, e.replace('"', "\\\""));
        } else {
            eprintln!("gator: {e}");
        }
        std::process::exit(1);
    }
}

fn print_agent_doc(request: AgentDocRequest) {
    let doc = build_agent_doc();
    let rendered = match request {
        AgentDocRequest::Help => render_agent_help_yaml(&doc),
        AgentDocRequest::Skill => render_agent_skill(&doc),
    };
    print!("{rendered}");
}

fn build_agent_doc() -> AgentDoc {
    AgentDoc {
        schema_version: text("1.0"),
        tool: AgentTool {
            name: text("gator"),
            binary: text("gator"),
            summary: text(
                "Launch Claude, Codex, or Gemini inside a macOS sandbox with optional prompter and session integration.",
            ),
        },
        usage: AgentUsage {
            invocation: text("gator <AGENT> [PROFILES...] [OPTIONS] [-- <AGENT_ARGS>...]"),
            notes: vec![
                text("Supported agent targets are `claude`, `codex`, and `gemini`."),
                text("Agent-doc requests are top-level only: use `gator --agent-help` or `gator --agent-skill`, never `gator claude --agent-help`."),
                text("Session mode makes the silent-critic contract the sole authority for sandbox grants and disables manual grant flags plus YOLO injection."),
            ],
        },
        shared_sections: vec![AgentSection {
            title: text("Top-level agent-doc contract"),
            content: text(
                "The hidden agent-doc flags print to stdout, exit 0, and do not appear in normal `gator --help` output.",
            ),
        }],
        commands: vec![AgentCommand {
            name: text("run"),
            summary: text(
                "Assemble a sandbox policy, optionally compose a prompter prompt, inject agent-specific YOLO flags when allowed, and exec the selected agent.",
            ),
            usage: text("gator <AGENT> [PROFILES...] [OPTIONS] [-- <AGENT_ARGS>...]"),
            arguments: vec![
                positional_arg("agent", "Agent target: `claude`, `codex`, or `gemini`.", true),
                positional_arg(
                    "profiles",
                    "Prompter profile names appended after the built-in baseline profiles.",
                    false,
                ),
                flag_arg("--workdir", "Override the inferred worktree root.", false),
                flag_arg("--add-dirs", "Grant extra read-write directories.", false),
                flag_arg("--add-dirs-ro", "Grant extra read-only directories.", false),
                flag_arg("--policy", "Load a named policy from `.gator/policies` or the user config directory.", false),
                flag_arg("--session", "Load sandbox grants from `silent-critic session sandbox <id> --format json`.", false),
                flag_arg("--share-worktrees", "Grant sibling worktrees read-only access in non-session mode.", false),
                flag_arg("--no-yolo", "Disable automatic autonomous-mode flag injection.", false),
                flag_arg("--no-prompt", "Skip prompter prompt composition entirely.", false),
                flag_arg("--dry-run", "Print the assembled sandbox policy to stderr and exit without exec.", false),
                flag_arg("--json", "Render validation and runtime errors as JSON on stderr.", false),
            ],
            output_shapes: vec![
                AgentOutputShape {
                    name: text("dry_run_policy"),
                    format: text("stderr text"),
                    description: text(
                        "`gator claude rust.full --dry-run` prints the assembled SBPL sandbox policy and exits 0 without launching the agent.",
                    ),
                },
                AgentOutputShape {
                    name: text("json_error"),
                    format: text("stderr json"),
                    description: text(
                        "With `--json`, validation and runtime failures print `{\"error\":\"...\"}` to stderr and exit 1.",
                    ),
                },
                AgentOutputShape {
                    name: text("agent_exec"),
                    format: text("process replacement"),
                    description: text(
                        "Successful non-dry invocations replace the `gator` process with `sandbox-exec -- <agent ...>`.",
                    ),
                },
            ],
        }],
        arguments: vec![
            flag_arg("--agent-help", "Print this canonical YAML reference document.", false),
            flag_arg("--agent-skill", "Print the same content as a Claude skill document.", false),
        ],
        environment_variables: vec![
            AgentEnvironmentVar {
                name: text("PATH"),
                description: text(
                    "Prepended with `~/.local/clankers/bin` when that directory exists so wrapped tools stay available inside the sandbox.",
                ),
                required: false,
            },
            AgentEnvironmentVar {
                name: text("HOME"),
                description: text(
                    "Used to resolve default policy and prompt configuration locations.",
                ),
                required: false,
            },
        ],
        config_files: vec![
            AgentConfigFile {
                path: text("<workdir>/.gator/policies/<name>.toml"),
                purpose: text("Project-local named policy files loaded by `--policy`."),
            },
            AgentConfigFile {
                path: text("~/.config/gator/policies/<name>.toml"),
                purpose: text("Fallback location for named policy files loaded by `--policy`."),
            },
            AgentConfigFile {
                path: text("~/.config/sandbox-exec/agent.sb"),
                purpose: text("Base macOS sandbox profile used by policy assembly."),
            },
        ],
        default_paths: vec![
            AgentPath {
                name: text("default workdir"),
                path: text("git root or current working directory"),
                purpose: text("Used when `--workdir` is omitted outside session mode."),
            },
            AgentPath {
                name: text("sandbox policy tempfile"),
                path: text("$TMPDIR/gator-policy-XXXXXX.sb"),
                purpose: text("Temporary file written before `sandbox-exec` is launched."),
            },
        ],
        output_shapes: vec![
            AgentOutputShape {
                name: text("stderr json error"),
                format: text("stderr json"),
                description: text("`--json` converts validation and runtime errors into a single JSON object."),
            },
            AgentOutputShape {
                name: text("stderr text error"),
                format: text("stderr text"),
                description: text("Without `--json`, errors print as `gator: <message>`."),
            },
            AgentOutputShape {
                name: text("dry run policy"),
                format: text("stderr text"),
                description: text("`--dry-run` prints the full assembled sandbox policy instead of executing an agent."),
            },
        ],
        examples: vec![
            AgentExample {
                name: text("dry-run policy review"),
                command: text("gator claude rust.full --dry-run"),
                description: text("Inspect the sandbox policy and prompt setup without launching Claude."),
            },
            AgentExample {
                name: text("session mode"),
                command: text("gator codex --session contract-123 -- --model gpt-5-codex"),
                description: text("Run Codex using a silent-critic session contract; session mode disables manual grants and YOLO injection."),
            },
            AgentExample {
                name: text("shared worktrees"),
                command: text("gator gemini --share-worktrees python.full -- --prompt foo"),
                description: text("Grant sibling worktrees read-only access while passing extra args through to Gemini."),
            },
        ],
        failure_modes: vec![
            AgentFailureMode {
                name: text("session validation conflict"),
                symptom: text("`--session` is combined with `--workdir`, `--add-dirs`, `--add-dirs-ro`, `--policy`, `--share-worktrees`, or `--no-yolo`."),
                resolution: text("Remove the conflicting manual grant flags or drop `--session`."),
            },
            AgentFailureMode {
                name: text("policy not found"),
                symptom: text("A named policy cannot be loaded from the project or user config directories."),
                resolution: text("Create the missing TOML file or correct the `--policy` name."),
            },
            AgentFailureMode {
                name: text("session sandbox lookup failed"),
                symptom: text("`silent-critic session sandbox <id> --format json` fails or returns invalid JSON."),
                resolution: text("Verify `silent-critic` is installed, the session ID exists, and the session sandbox command succeeds independently."),
            },
            AgentFailureMode {
                name: text("sandbox or agent exec failed"),
                symptom: text("`gator` cannot build or exec the final `sandbox-exec` command."),
                resolution: text("Confirm macOS `sandbox-exec` is available and the target agent CLI is installed on PATH."),
            },
        ],
        operator_mistakes: vec![
            AgentOperatorMistake {
                name: text("Placing agent-doc flags after the agent positional"),
                symptom: text("`gator claude --agent-help` fails normal clap validation instead of printing docs."),
                correction: text("Move the request to the top level: `gator --agent-help` or `gator --agent-skill`."),
            },
            AgentOperatorMistake {
                name: text("Expecting session mode to honor manual grant flags"),
                symptom: text("`--session` rejects `--policy`, `--workdir`, extra directory grants, `--share-worktrees`, and `--no-yolo`."),
                correction: text("Encode grants in the silent-critic contract or run outside session mode."),
            },
            AgentOperatorMistake {
                name: text("Expecting YOLO injection in all modes"),
                symptom: text("No automatic autonomous-mode flag is injected when `--no-yolo` is set or when session mode is active."),
                correction: text("Leave `--no-yolo` unset in non-session mode when you want the default agent-specific YOLO behavior."),
            },
        ],
        constraints: vec![
            text("`gator` is macOS-specific because it shells out to `sandbox-exec`."),
            text("Gemini has no known YOLO flag; gator only injects sandbox plumbing for that target."),
            text("Agent-doc flags intentionally bypass clap only for the exact top-level invocations."),
        ],
    }
}

fn positional_arg(name: &str, description: &str, required: bool) -> AgentArgument {
    AgentArgument {
        name: text(name),
        positional: true,
        description: text(description),
        required,
    }
}

fn flag_arg(name: &str, description: &str, required: bool) -> AgentArgument {
    AgentArgument {
        name: text(name),
        positional: false,
        description: text(description),
        required,
    }
}

fn text(value: &str) -> String {
    value.to_owned()
}
