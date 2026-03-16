use clap::Parser;
use serde_json::json;

use silent_critic::cli::{
    Cli, Command, ContractCommand, CriterionCommand, ProjectCommand, SessionCommand,
};
use silent_critic::commands::{contract, criterion, decide, log, project, session};
use silent_critic::config::load_config;
use silent_critic::output::{err_response, ok_response};

fn main() {
    let cli = Cli::parse();
    let code = run(cli);
    std::process::exit(code);
}

fn run(cli: Cli) -> i32 {
    let json = cli.json;

    match dispatch(cli) {
        Ok(output) => {
            println!("{output}");
            0
        }
        Err(e) => {
            if json {
                let out = err_response("error", "ERROR", &e.to_string(), json!({}));
                println!("{out}");
            } else {
                eprintln!("error: {e}");
            }
            1
        }
    }
}

fn dispatch(cli: Cli) -> anyhow::Result<String> {
    let json = cli.json;
    let config = load_config()?;

    match cli.command {
        Command::Project { command } => match command {
            ProjectCommand::Init { name } => {
                let result = project::run_init(&config, name.as_deref())?;
                if json {
                    Ok(ok_response(
                        "project.init",
                        json!({
                            "project_id": result.project.id,
                            "project_name": result.project.name,
                            "repo_hash": result.project.repo_hash,
                            "db_path": result.db_path.display().to_string(),
                        }),
                    )
                    .to_string())
                } else {
                    Ok(format!(
                        "project: {}\ndb: {}",
                        result.project.name,
                        result.db_path.display()
                    ))
                }
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
                    if json {
                        Ok(ok_response("criterion.create", json!({"criterion": c})).to_string())
                    } else {
                        Ok(format!("{} [{}] {}", c.id, c.namespace, c.name))
                    }
                }
                CriterionCommand::List { namespace } => {
                    let criteria = criterion::run_list(&conn, namespace.as_deref())?;
                    if json {
                        Ok(ok_response("criterion.list", json!({"criteria": criteria}))
                            .to_string())
                    } else {
                        let mut out = String::new();
                        for c in &criteria {
                            out.push_str(&format!(
                                "{}\t[{}]\t{}\t{}\n",
                                c.id, c.namespace, c.name, c.claim
                            ));
                        }
                        Ok(out)
                    }
                }
                CriterionCommand::Show { id } => {
                    let c = criterion::run_show(&conn, &id)?;
                    if json {
                        Ok(ok_response("criterion.show", json!({"criterion": c})).to_string())
                    } else {
                        Ok(format!(
                            "id: {}\nnamespace: {}\nname: {}\nclaim: {}\nevaluator: {}\ncheck_spec: {}\ncreated: {}",
                            c.id, c.namespace, c.name, c.claim, c.evaluator_type, c.check_spec, c.created_at
                        ))
                    }
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
                    if json {
                        Ok(ok_response("criterion.update", json!({"criterion": c})).to_string())
                    } else {
                        Ok(format!("updated: {} [{}] {}", c.id, c.namespace, c.name))
                    }
                }
                CriterionCommand::Deprecate { id } => {
                    criterion::run_deprecate(&conn, &id)?;
                    if json {
                        Ok(ok_response("criterion.deprecate", json!({"id": id})).to_string())
                    } else {
                        Ok(format!("deprecated: {id}"))
                    }
                }
                CriterionCommand::Export { id } => {
                    let toml_str = criterion::run_export(&conn, &id)?;
                    if json {
                        Ok(ok_response("criterion.export", json!({"toml": toml_str})).to_string())
                    } else {
                        Ok(toml_str)
                    }
                }
                CriterionCommand::Import { file } => {
                    let c = criterion::run_import(&conn, &file)?;
                    if json {
                        Ok(ok_response("criterion.import", json!({"criterion": c})).to_string())
                    } else {
                        Ok(format!("imported: {} [{}] {}", c.id, c.namespace, c.name))
                    }
                }
            }
        }

        Command::Session { command } => {
            let conn = open_project_db(&config)?;
            match command {
                SessionCommand::New { worktree } => {
                    let s = session::run_new(&conn, &worktree)?;
                    if json {
                        Ok(ok_response(
                            "session.new",
                            json!({
                                "session_id": s.id,
                                "status": s.status.as_str(),
                                "operator_token": s.operator_token,
                            }),
                        )
                        .to_string())
                    } else {
                        Ok(format!(
                            "session: {}\nstatus: {}\noperator token: {}",
                            s.id, s.status, s.operator_token
                        ))
                    }
                }
                SessionCommand::Discover { docs } => {
                    let contexts = session::run_discover(&conn, &docs)?;
                    if json {
                        Ok(ok_response(
                            "session.discover",
                            json!({
                                "context_count": contexts.len(),
                                "contexts": contexts,
                            }),
                        )
                        .to_string())
                    } else {
                        Ok(format!("discovered {} context items", contexts.len()))
                    }
                }
                SessionCommand::Status => {
                    let report = session::run_status(&conn)?;
                    if json {
                        Ok(ok_response(
                            "session.status",
                            json!({
                                "session_id": report.session_id,
                                "status": report.status.as_str(),
                                "worktree": report.worktree,
                                "contract_id": report.contract_id,
                                "goal": report.goal,
                                "criteria_count": report.criteria_count,
                                "evidence_count": report.evidence_count,
                                "discovery_count": report.discovery_count,
                                "started_at": report.started_at,
                            }),
                        )
                        .to_string())
                    } else {
                        let mut out = format!(
                            "session: {}\nstatus: {}\nworktree: {}\nstarted: {}",
                            report.session_id, report.status, report.worktree, report.started_at
                        );
                        if let Some(ref goal) = report.goal {
                            out.push_str(&format!("\ngoal: {goal}"));
                        }
                        out.push_str(&format!(
                            "\ncriteria: {}\nevidence: {}\ndiscovery items: {}",
                            report.criteria_count, report.evidence_count, report.discovery_count
                        ));
                        Ok(out)
                    }
                }
                SessionCommand::End => {
                    let report = session::run_end(&conn)?;
                    if json {
                        Ok(ok_response(
                            "session.end",
                            json!({
                                "session_id": report.session_id,
                                "contract_id": report.contract_id,
                                "total_criteria": report.total_criteria,
                                "total_evidence": report.total_evidence,
                                "residual_count": report.residuals.len(),
                                "residuals": report.residuals.iter().map(|r| json!({
                                    "criterion_id": r.criterion_id,
                                    "criterion_name": r.criterion_name,
                                    "reason": r.reason,
                                })).collect::<Vec<_>>(),
                            }),
                        )
                        .to_string())
                    } else {
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
                        Ok(out)
                    }
                }
                SessionCommand::Manifest => {
                    let token = std::env::var("SILENT_CRITIC_TOKEN")
                        .map_err(|_| anyhow::anyhow!("SILENT_CRITIC_TOKEN not set"))?;
                    let report = session::run_manifest(&conn, &token)?;
                    if json {
                        Ok(ok_response(
                            "session.manifest",
                            json!({
                                "goal": report.goal,
                                "criteria": report.criteria.iter().map(|c| json!({
                                    "id": c.id,
                                    "namespace": c.namespace,
                                    "name": c.name,
                                    "claim": c.claim,
                                    "check_spec": c.check_spec,
                                    "tier": c.tier.as_str(),
                                })).collect::<Vec<_>>(),
                            }),
                        )
                        .to_string())
                    } else {
                        let mut out = format!("Goal: {}\n\nCriteria:\n", report.goal);
                        for c in &report.criteria {
                            out.push_str(&format!(
                                "  {} [{}] {} -- {}\n    check: {}\n",
                                c.id, c.namespace, c.name, c.claim, c.check_spec
                            ));
                        }
                        Ok(out)
                    }
                }
                SessionCommand::Sandbox {
                    session_id,
                    format: _,
                } => {
                    let sid = if let Some(ref id) = session_id {
                        id.clone()
                    } else {
                        // Fall back to active session
                        let active = silent_critic::db::get_active_session(&conn)?
                            .ok_or_else(|| anyhow::anyhow!("no active session and no session ID provided"))?;
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
                    if json {
                        Ok(ok_response(
                            "session.compose_from",
                            json!({
                                "contract_id": result.contract.id,
                                "goal": result.contract.goal,
                                "criteria_created": result.criteria_created,
                                "criteria_reused": result.criteria_reused,
                            }),
                        )
                        .to_string())
                    } else {
                        Ok(format!(
                            "contract: {}\ngoal: {}\ncriteria created: {}\ncriteria reused: {}",
                            result.contract.id,
                            result.contract.goal,
                            result.criteria_created,
                            result.criteria_reused
                        ))
                    }
                }
                SessionCommand::Go { prompt_only } => {
                    if !prompt_only {
                        anyhow::bail!(
                            "--prompt-only is required (process spawning not supported)"
                        );
                    }
                    let result = session::run_go_prompt_only(&conn)?;
                    if json {
                        Ok(ok_response(
                            "session.go",
                            json!({
                                "worker_token": result.worker_token,
                                "prompt": result.prompt,
                            }),
                        )
                        .to_string())
                    } else {
                        Ok(result.prompt)
                    }
                }
                SessionCommand::Submit { criterion } => {
                    let token = std::env::var("SILENT_CRITIC_TOKEN")
                        .map_err(|_| anyhow::anyhow!("SILENT_CRITIC_TOKEN not set"))?;
                    let e = session::run_submit(&conn, &token, &criterion)?;
                    if json {
                        Ok(ok_response(
                            "session.submit",
                            json!({
                                "evidence_id": e.id,
                                "criterion_id": e.criterion_id,
                                "exit_code": e.exit_code,
                                "pass": e.exit_code == 0,
                            }),
                        )
                        .to_string())
                    } else {
                        Ok(format!(
                            "evidence: {}\nexit code: {}\npass: {}",
                            e.id,
                            e.exit_code,
                            e.exit_code == 0
                        ))
                    }
                }
            }
        }

        Command::Contract { command } => {
            let conn = open_project_db(&config)?;
            match command {
                ContractCommand::Show { id, role } => {
                    let view = contract::run_show(&conn, &id, &role)?;
                    if json {
                        Ok(ok_response(
                            "contract.show",
                            json!({
                                "id": view.id,
                                "goal": view.goal,
                                "criteria": view.criteria.iter().map(|c| json!({
                                    "criterion_id": c.criterion_id,
                                    "namespace": c.namespace,
                                    "name": c.name,
                                    "claim": c.claim,
                                    "visibility": c.visibility.as_str(),
                                    "tier": c.tier,
                                })).collect::<Vec<_>>(),
                            }),
                        )
                        .to_string())
                    } else {
                        let mut out = format!("Contract: {}\nGoal: {}\n\nCriteria:\n", view.id, view.goal);
                        for c in &view.criteria {
                            out.push_str(&format!(
                                "  [{}] {} -- {} ({})\n",
                                c.namespace, c.name, c.claim, c.visibility
                            ));
                        }
                        Ok(out)
                    }
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
            let d = decide::run_decide(
                &conn,
                &contract,
                &r#type,
                &basis,
                evidence_refs.as_deref(),
            )?;
            if json {
                Ok(ok_response(
                    "decide",
                    json!({
                        "decision_id": d.id,
                        "type": d.decision_type.as_str(),
                        "outcome": d.outcome,
                    }),
                )
                .to_string())
            } else {
                Ok(format!(
                    "decision: {}\ntype: {}\noutcome: {}",
                    d.id, d.decision_type, d.outcome
                ))
            }
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
