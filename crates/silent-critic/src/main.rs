use serde_json::json;
use tftio_cli_common::{
    AgentCapability, AgentDispatch, AgentSurfaceSpec, CommandSelector, FlagSelector, LicenseType,
    StandardCommand, ToolSpec, command::run_standard_command_no_doctor, error::print_error,
    parse_with_agent_surface, render_response, render_response_parts, workspace_tool,
};

use silent_critic::cli::{
    Cli, Command, ContractCommand, CriterionCommand, MetaCommand, ProjectCommand, SessionCommand,
};
use silent_critic::commands::{contract, criterion, decide, log, project, session};
use silent_critic::config::load_config;

const TOOL_SPEC: ToolSpec = workspace_tool(
    "silent-critic",
    "Silent Critic",
    env!("CARGO_PKG_VERSION"),
    LicenseType::MIT,
    true,
    false,
    false,
)
.with_agent_surface(&AGENT_SURFACE);

const SESSION_STATUS_COMMAND: CommandSelector = CommandSelector::new(&["session", "status"]);
const SESSION_MANIFEST_COMMAND: CommandSelector = CommandSelector::new(&["session", "manifest"]);
const SESSION_SUBMIT_COMMAND: CommandSelector = CommandSelector::new(&["session", "submit"]);
const SESSION_SUBMIT_CRITERION_FLAG: FlagSelector =
    FlagSelector::new(&["session", "submit"], "criterion");

const SESSION_STATUS_CAPABILITY: AgentCapability = AgentCapability::new(
    "session-status",
    "Inspect the currently active worker session",
    &[SESSION_STATUS_COMMAND],
    &[],
)
.with_examples(&["silent-critic session status"]);

const SESSION_MANIFEST_CAPABILITY: AgentCapability = AgentCapability::new(
    "session-manifest",
    "Read the worker-visible contract manifest for the active session",
    &[SESSION_MANIFEST_COMMAND],
    &[],
)
.with_examples(&["silent-critic session manifest"]);

const SESSION_SUBMIT_CAPABILITY: AgentCapability = AgentCapability::new(
    "session-submit",
    "Submit evidence for one visible criterion",
    &[SESSION_SUBMIT_COMMAND],
    &[SESSION_SUBMIT_CRITERION_FLAG],
)
.with_examples(&["silent-critic session submit --criterion <ID>"])
.with_constraints(
    "requires the runtime SILENT_CRITIC_TOKEN worker token and a visible criterion id",
);

const AGENT_SURFACE: AgentSurfaceSpec = AgentSurfaceSpec::new(&[
    SESSION_STATUS_CAPABILITY,
    SESSION_MANIFEST_CAPABILITY,
    SESSION_SUBMIT_CAPABILITY,
]);

fn main() {
    let cli = match parse_with_agent_surface::<Cli>(&TOOL_SPEC) {
        Ok(AgentDispatch::Cli(cli)) => cli,
        Ok(AgentDispatch::Printed(code)) => std::process::exit(code),
        Err(error) => error.exit(),
    };
    let code = run(cli);
    std::process::exit(code);
}

fn run(cli: Cli) -> i32 {
    let json = cli.json;

    if let Command::Meta { command } = &cli.command {
        let standard_command = match command {
            MetaCommand::Version => StandardCommand::Version { json },
            MetaCommand::License => StandardCommand::License,
            MetaCommand::Completions { shell } => StandardCommand::Completions { shell: *shell },
        };
        return run_standard_command_no_doctor::<Cli>(&TOOL_SPEC, &standard_command);
    }

    match dispatch(cli) {
        Ok(output) => {
            println!("{output}");
            0
        }
        Err(e) => print_error("error", json, &e.to_string()),
    }
}

fn command_output(
    command: &str,
    json: bool,
    data: serde_json::Value,
    text: impl Into<String>,
) -> String {
    render_response(command, json, data, text)
}

fn dispatch(cli: Cli) -> anyhow::Result<String> {
    let json = cli.json;
    let config = load_config()?;

    match cli.command {
        Command::Meta { .. } => unreachable!("meta commands are handled before dispatch"),
        Command::Project { command } => match command {
            ProjectCommand::Init { name } => {
                let result = project::run_init(&config, name.as_deref())?;
                Ok(command_output(
                    "project.init",
                    json,
                    json!({
                        "project_id": result.project.id,
                        "project_name": result.project.name,
                        "repo_hash": result.project.repo_hash,
                        "db_path": result.db_path.display().to_string(),
                    }),
                    format!(
                        "project: {}\ndb: {}",
                        result.project.name,
                        result.db_path.display()
                    ),
                ))
            }
        },

        Command::Criterion { command } => {
            let conn = open_project_db(&config)?;
            match command {
                CriterionCommand::Create {
                    namespace,
                    name,
                    claim,
                    evaluator_type,
                    check_spec,
                    parameter_schema,
                } => {
                    let c = criterion::run_create(
                        &conn,
                        namespace.as_deref(),
                        name.as_deref(),
                        claim.as_deref(),
                        evaluator_type.as_ref(),
                        check_spec.as_deref(),
                        parameter_schema.as_deref(),
                    )?;
                    Ok(command_output(
                        "criterion.create",
                        json,
                        json!({"criterion": c}),
                        format!("{} [{}] {}", c.id, c.namespace, c.name),
                    ))
                }
                CriterionCommand::List { namespace } => {
                    let criteria = criterion::run_list(&conn, namespace.as_deref())?;
                    Ok(render_response_parts(
                        "criterion.list",
                        json,
                        || json!({"criteria": &criteria}),
                        || {
                            let mut out = String::new();
                            for c in &criteria {
                                out.push_str(&format!(
                                    "{}\t[{}]\t{}\t{}\n",
                                    c.id, c.namespace, c.name, c.claim
                                ));
                            }
                            out
                        },
                    ))
                }
                CriterionCommand::Show { id } => {
                    let c = criterion::run_show(&conn, &id)?;
                    Ok(command_output(
                        "criterion.show",
                        json,
                        json!({"criterion": c}),
                        format!(
                            "id: {}\nnamespace: {}\nname: {}\nclaim: {}\nevaluator: {}\ncheck_spec: {}\ncreated: {}",
                            c.id,
                            c.namespace,
                            c.name,
                            c.claim,
                            c.evaluator_type,
                            c.check_spec,
                            c.created_at
                        ),
                    ))
                }
                CriterionCommand::Update {
                    id,
                    namespace,
                    name,
                    claim,
                    evaluator_type,
                    check_spec,
                } => {
                    let c = criterion::run_update(
                        &conn,
                        &id,
                        namespace.as_deref(),
                        name.as_deref(),
                        claim.as_deref(),
                        evaluator_type.as_ref(),
                        check_spec.as_deref(),
                    )?;
                    Ok(command_output(
                        "criterion.update",
                        json,
                        json!({"criterion": c}),
                        format!("updated: {} [{}] {}", c.id, c.namespace, c.name),
                    ))
                }
                CriterionCommand::Deprecate { id } => {
                    criterion::run_deprecate(&conn, &id)?;
                    Ok(command_output(
                        "criterion.deprecate",
                        json,
                        json!({"id": id}),
                        format!("deprecated: {id}"),
                    ))
                }
                CriterionCommand::Export { id } => {
                    let toml_str = criterion::run_export(&conn, &id)?;
                    Ok(command_output(
                        "criterion.export",
                        json,
                        json!({"toml": toml_str}),
                        toml_str,
                    ))
                }
                CriterionCommand::Import { file } => {
                    let c = criterion::run_import(&conn, &file)?;
                    Ok(command_output(
                        "criterion.import",
                        json,
                        json!({"criterion": c}),
                        format!("imported: {} [{}] {}", c.id, c.namespace, c.name),
                    ))
                }
            }
        }

        Command::Session { command } => {
            let conn = open_project_db(&config)?;
            match command {
                SessionCommand::New { worktree } => {
                    let s = session::run_new(&conn, &worktree)?;
                    Ok(command_output(
                        "session.new",
                        json,
                        json!({
                            "session_id": s.id,
                            "status": s.status.as_str(),
                            "operator_token": s.operator_token,
                        }),
                        format!(
                            "session: {}\nstatus: {}\noperator token: {}",
                            s.id, s.status, s.operator_token
                        ),
                    ))
                }
                SessionCommand::Discover { docs } => {
                    let contexts = session::run_discover(&conn, &docs)?;
                    Ok(command_output(
                        "session.discover",
                        json,
                        json!({
                            "context_count": contexts.len(),
                            "contexts": contexts,
                        }),
                        format!("discovered {} context items", contexts.len()),
                    ))
                }
                SessionCommand::Status => {
                    let report = session::run_status(&conn)?;
                    Ok(render_response_parts(
                        "session.status",
                        json,
                        || {
                            json!({
                                "session_id": &report.session_id,
                                "status": report.status.as_str(),
                                "worktree": &report.worktree,
                                "contract_id": &report.contract_id,
                                "goal": &report.goal,
                                "criteria_count": report.criteria_count,
                                "evidence_count": report.evidence_count,
                                "discovery_count": report.discovery_count,
                                "started_at": &report.started_at,
                            })
                        },
                        || {
                            let mut out = format!(
                                "session: {}\nstatus: {}\nworktree: {}\nstarted: {}",
                                report.session_id,
                                report.status,
                                report.worktree,
                                report.started_at
                            );
                            if let Some(ref goal) = report.goal {
                                out.push_str(&format!("\ngoal: {goal}"));
                            }
                            out.push_str(&format!(
                                "\ncriteria: {}\nevidence: {}\ndiscovery items: {}",
                                report.criteria_count,
                                report.evidence_count,
                                report.discovery_count
                            ));
                            out
                        },
                    ))
                }
                SessionCommand::End => {
                    let report = session::run_end(&conn)?;
                    Ok(render_response_parts(
                        "session.end",
                        json,
                        || {
                            json!({
                                "session_id": &report.session_id,
                                "contract_id": &report.contract_id,
                                "total_criteria": report.total_criteria,
                                "total_evidence": report.total_evidence,
                                "residual_count": report.residuals.len(),
                                "residuals": report.residuals.iter().map(|r| json!({
                                    "criterion_id": &r.criterion_id,
                                    "criterion_name": &r.criterion_name,
                                    "reason": &r.reason,
                                })).collect::<Vec<_>>(),
                            })
                        },
                        || {
                            let mut out = format!(
                                "session ended\ncontract: {}\ncriteria: {}\nevidence: {}",
                                report.contract_id, report.total_criteria, report.total_evidence
                            );
                            if report.residuals.is_empty() {
                                out.push_str("\nresiduals: none");
                            } else {
                                out.push_str(&format!("\nresiduals: {}", report.residuals.len()));
                                for r in &report.residuals {
                                    out.push_str(&format!(
                                        "\n  - {} ({}): {}",
                                        r.criterion_name, r.criterion_id, r.reason
                                    ));
                                }
                            }
                            out
                        },
                    ))
                }
                SessionCommand::Manifest => {
                    let token = std::env::var("SILENT_CRITIC_TOKEN")
                        .map_err(|_| anyhow::anyhow!("SILENT_CRITIC_TOKEN not set"))?;
                    let report = session::run_manifest(&conn, &token)?;
                    Ok(render_response_parts(
                        "session.manifest",
                        json,
                        || {
                            json!({
                                "goal": &report.goal,
                                "criteria": report.criteria.iter().map(|c| json!({
                                    "id": &c.id,
                                    "namespace": &c.namespace,
                                    "name": &c.name,
                                    "claim": &c.claim,
                                    "check_spec": &c.check_spec,
                                    "tier": c.tier.as_str(),
                                })).collect::<Vec<_>>(),
                            })
                        },
                        || {
                            let mut out = format!("Goal: {}\n\nCriteria:\n", report.goal);
                            for c in &report.criteria {
                                out.push_str(&format!(
                                    "  {} [{}] {} -- {}\n    check: {}\n",
                                    c.id, c.namespace, c.name, c.claim, c.check_spec
                                ));
                            }
                            out
                        },
                    ))
                }
                SessionCommand::Sandbox {
                    session_id,
                    format: _,
                } => {
                    let sid = if let Some(ref id) = session_id {
                        id.clone()
                    } else {
                        // Fall back to active session
                        let active =
                            silent_critic::db::get_active_session(&conn)?.ok_or_else(|| {
                                anyhow::anyhow!("no active session and no session ID provided")
                            })?;
                        active.id
                    };
                    let output = session::run_session_sandbox(&conn, &sid)?;
                    // Output is already JSON, return directly
                    Ok(output)
                }
                SessionCommand::ComposeFrom => {
                    let mut input = String::new();
                    std::io::Read::read_to_string(&mut std::io::stdin(), &mut input)?;
                    let result = session::run_compose_from(&conn, &input)?;
                    Ok(render_response_parts(
                        "session.compose_from",
                        json,
                        || {
                            json!({
                                "contract_id": &result.contract.id,
                                "goal": &result.contract.goal,
                                "criteria_created": result.criteria_created,
                                "criteria_reused": result.criteria_reused,
                            })
                        },
                        || {
                            format!(
                                "contract: {}\ngoal: {}\ncriteria created: {}\ncriteria reused: {}",
                                result.contract.id,
                                result.contract.goal,
                                result.criteria_created,
                                result.criteria_reused
                            )
                        },
                    ))
                }
                SessionCommand::Go { prompt_only } => {
                    if !prompt_only {
                        anyhow::bail!("--prompt-only is required (process spawning not supported)");
                    }
                    let result = session::run_go_prompt_only(&conn)?;
                    Ok(render_response_parts(
                        "session.go",
                        json,
                        || {
                            json!({
                                "worker_token": &result.worker_token,
                                "prompt": &result.prompt,
                            })
                        },
                        || result.prompt.clone(),
                    ))
                }
                SessionCommand::Submit { criterion } => {
                    let token = std::env::var("SILENT_CRITIC_TOKEN")
                        .map_err(|_| anyhow::anyhow!("SILENT_CRITIC_TOKEN not set"))?;
                    let e = session::run_submit(&conn, &token, &criterion)?;
                    Ok(render_response_parts(
                        "session.submit",
                        json,
                        || {
                            json!({
                                "evidence_id": &e.id,
                                "criterion_id": &e.criterion_id,
                                "exit_code": e.exit_code,
                                "pass": e.exit_code == 0,
                            })
                        },
                        || {
                            format!(
                                "evidence: {}\nexit code: {}\npass: {}",
                                e.id,
                                e.exit_code,
                                e.exit_code == 0
                            )
                        },
                    ))
                }
            }
        }

        Command::Contract { command } => {
            let conn = open_project_db(&config)?;
            match command {
                ContractCommand::Show { id, role } => {
                    let view = contract::run_show(&conn, &id, &role)?;
                    Ok(render_response_parts(
                        "contract.show",
                        json,
                        || {
                            json!({
                                "id": &view.id,
                                "goal": &view.goal,
                                "criteria": view.criteria.iter().map(|c| json!({
                                    "criterion_id": &c.criterion_id,
                                    "namespace": &c.namespace,
                                    "name": &c.name,
                                    "claim": &c.claim,
                                    "visibility": c.visibility.as_str(),
                                    "tier": c.tier,
                                })).collect::<Vec<_>>(),
                            })
                        },
                        || {
                            let mut out = format!(
                                "Contract: {}\nGoal: {}\n\nCriteria:\n",
                                view.id, view.goal
                            );
                            for c in &view.criteria {
                                out.push_str(&format!(
                                    "  [{}] {} -- {} ({})\n",
                                    c.namespace, c.name, c.claim, c.visibility
                                ));
                            }
                            out
                        },
                    ))
                }
            }
        }

        Command::Decide {
            contract,
            r#type,
            basis,
            evidence_refs,
        } => {
            let conn = open_project_db(&config)?;
            let d =
                decide::run_decide(&conn, &contract, &r#type, &basis, evidence_refs.as_deref())?;
            Ok(render_response_parts(
                "decide",
                json,
                || {
                    json!({
                        "decision_id": &d.id,
                        "type": d.decision_type.as_str(),
                        "outcome": &d.outcome,
                    })
                },
                || {
                    format!(
                        "decision: {}\ntype: {}\noutcome: {}",
                        d.id, d.decision_type, d.outcome
                    )
                },
            ))
        }

        Command::Log { contract, format } => {
            let conn = open_project_db(&config)?;
            log::run_log(&conn, &contract, &format)
        }
    }
}

/// Open the project database, resolving from current directory.
fn open_project_db(config: &silent_critic::config::Config) -> anyhow::Result<rusqlite::Connection> {
    let cwd = std::env::current_dir()?;
    let repo_root = silent_critic::project::find_repo_root(&cwd)?;
    let repo_hash = silent_critic::project::compute_repo_hash(&repo_root)?;
    let db_path = silent_critic::config::resolve_db_path(config, &repo_hash)?;
    let conn = silent_critic::db::open_db(&db_path)?;
    Ok(conn)
}
