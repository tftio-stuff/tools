//! Chronicle CLI entrypoint.

use serde_json::json;
use tftio_cli_common::{
    AgentCapability, AgentDispatch, AgentSurfaceSpec, CommandSelector, DoctorCheck, DoctorChecks,
    FlagSelector, LicenseType, RepoInfo, StandardCommand, ToolSpec, command::run_standard_command,
    error::print_error, parse_with_agent_surface, render_response, render_response_with,
    workspace_tool,
};

use chronicle::cli::{Cli, Command, MetaCommand, SourcesCommand};
use chronicle::commands::{
    export::run_export, ingest::run_ingest, reindex::run_reindex, search::run_search, sources,
    stats::run_stats,
};
use chronicle::config::{ensure_db_dir, resolve_db_path};
use chronicle::db::{init_db, open_db};
use chronicle::models::SourceFormat;
use chronicle::repo;

// --- Agent surface ---

const INGEST_COMMAND: CommandSelector = CommandSelector::new(&["ingest"]);
const REINDEX_COMMAND: CommandSelector = CommandSelector::new(&["reindex"]);
const SEARCH_COMMAND: CommandSelector = CommandSelector::new(&["search"]);
const STATS_COMMAND: CommandSelector = CommandSelector::new(&["stats"]);
const EXPORT_COMMAND: CommandSelector = CommandSelector::new(&["export"]);
const SOURCES_LIST_COMMAND: CommandSelector = CommandSelector::new(&["sources", "list"]);
const SOURCES_ADD_COMMAND: CommandSelector = CommandSelector::new(&["sources", "add"]);

const INGEST_SOURCE_FLAG: FlagSelector = FlagSelector::new(&["ingest"], "source");
const INGEST_PATH_FLAG: FlagSelector = FlagSelector::new(&["ingest"], "path");
const INGEST_PARSER_FLAG: FlagSelector = FlagSelector::new(&["ingest"], "parser");
const INGEST_FULL_FLAG: FlagSelector = FlagSelector::new(&["ingest"], "full");
const INGEST_JSON_FLAG: FlagSelector = FlagSelector::new(&["ingest"], "json");
const REINDEX_SOURCE_FLAG: FlagSelector = FlagSelector::new(&["reindex"], "source");
const REINDEX_FULL_FLAG: FlagSelector = FlagSelector::new(&["reindex"], "full");
const REINDEX_JSON_FLAG: FlagSelector = FlagSelector::new(&["reindex"], "json");
const SEARCH_SOURCE_FLAG: FlagSelector = FlagSelector::new(&["search"], "source");
const SEARCH_ROLE_FLAG: FlagSelector = FlagSelector::new(&["search"], "role");
const SEARCH_PROJECT_FLAG: FlagSelector = FlagSelector::new(&["search"], "project");
const SEARCH_LIMIT_FLAG: FlagSelector = FlagSelector::new(&["search"], "limit");
const SEARCH_JSON_FLAG: FlagSelector = FlagSelector::new(&["search"], "json");
const STATS_SOURCE_FLAG: FlagSelector = FlagSelector::new(&["stats"], "source");
const STATS_JSON_FLAG: FlagSelector = FlagSelector::new(&["stats"], "json");
const EXPORT_SESSION_FLAG: FlagSelector = FlagSelector::new(&["export"], "session");
const EXPORT_FORMAT_FLAG: FlagSelector = FlagSelector::new(&["export"], "format");
const SOURCES_LIST_JSON_FLAG: FlagSelector = FlagSelector::new(&["sources", "list"], "json");
const SOURCES_ADD_NAME_FLAG: FlagSelector = FlagSelector::new(&["sources", "add"], "name");
const SOURCES_ADD_PARSER_FLAG: FlagSelector = FlagSelector::new(&["sources", "add"], "parser");
const SOURCES_ADD_PATH_FLAG: FlagSelector = FlagSelector::new(&["sources", "add"], "path");
const SOURCES_ADD_DESC_FLAG: FlagSelector = FlagSelector::new(&["sources", "add"], "description");
const SOURCES_ADD_JSON_FLAG: FlagSelector = FlagSelector::new(&["sources", "add"], "json");

const INGEST_CAPABILITY: AgentCapability = AgentCapability::minimal(
    "ingest",
    &[INGEST_COMMAND],
    &[
        INGEST_SOURCE_FLAG,
        INGEST_PATH_FLAG,
        INGEST_PARSER_FLAG,
        INGEST_FULL_FLAG,
        INGEST_JSON_FLAG,
    ],
);
const REINDEX_CAPABILITY: AgentCapability = AgentCapability::minimal(
    "reindex",
    &[REINDEX_COMMAND],
    &[REINDEX_SOURCE_FLAG, REINDEX_FULL_FLAG, REINDEX_JSON_FLAG],
);
const SEARCH_CAPABILITY: AgentCapability = AgentCapability::minimal(
    "search",
    &[SEARCH_COMMAND],
    &[
        SEARCH_SOURCE_FLAG,
        SEARCH_ROLE_FLAG,
        SEARCH_PROJECT_FLAG,
        SEARCH_LIMIT_FLAG,
        SEARCH_JSON_FLAG,
    ],
);
const STATS_CAPABILITY: AgentCapability = AgentCapability::minimal(
    "stats",
    &[STATS_COMMAND],
    &[STATS_SOURCE_FLAG, STATS_JSON_FLAG],
);
const EXPORT_CAPABILITY: AgentCapability = AgentCapability::minimal(
    "export",
    &[EXPORT_COMMAND],
    &[EXPORT_SESSION_FLAG, EXPORT_FORMAT_FLAG],
);
const SOURCES_LIST_CAPABILITY: AgentCapability = AgentCapability::minimal(
    "sources-list",
    &[SOURCES_LIST_COMMAND],
    &[SOURCES_LIST_JSON_FLAG],
);
const SOURCES_ADD_CAPABILITY: AgentCapability = AgentCapability::minimal(
    "sources-add",
    &[SOURCES_ADD_COMMAND],
    &[
        SOURCES_ADD_NAME_FLAG,
        SOURCES_ADD_PARSER_FLAG,
        SOURCES_ADD_PATH_FLAG,
        SOURCES_ADD_DESC_FLAG,
        SOURCES_ADD_JSON_FLAG,
    ],
);

const AGENT_SURFACE: AgentSurfaceSpec = AgentSurfaceSpec::new(&[
    INGEST_CAPABILITY,
    REINDEX_CAPABILITY,
    SEARCH_CAPABILITY,
    STATS_CAPABILITY,
    EXPORT_CAPABILITY,
    SOURCES_LIST_CAPABILITY,
    SOURCES_ADD_CAPABILITY,
]);

const TOOL_SPEC: ToolSpec = workspace_tool(
    "chronicle",
    "Chronicle",
    env!("CARGO_PKG_VERSION"),
    LicenseType::MIT,
    true,
    false,
    false,
)
.with_agent_surface(&AGENT_SURFACE);

// --- Doctor ---

struct ChronicleDoctorChecks;

impl DoctorChecks for ChronicleDoctorChecks {
    fn repo_info() -> RepoInfo {
        RepoInfo::new("tftio-stuff", "tools")
    }

    fn current_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn tool_checks(&self) -> Vec<DoctorCheck> {
        let mut checks = Vec::new();

        // Check DB exists and is readable
        match resolve_db_path() {
            Ok(db_path) => {
                if db_path.exists() {
                    checks.push(DoctorCheck::pass("database exists"));
                    // Try opening
                    match open_db(&db_path) {
                        Ok(conn) => {
                            checks.push(DoctorCheck::pass("database readable"));
                            // Check source counts
                            if let Ok(stats) = repo::compute_stats(&conn, None) {
                                let total_sources = stats.len();
                                let total_sessions: i64 =
                                    stats.iter().map(|s| s.session_count).sum();
                                let total_messages: i64 =
                                    stats.iter().map(|s| s.message_count).sum();
                                checks.push(DoctorCheck::pass(format!(
                                    "{total_sources} sources, {total_sessions} sessions, {total_messages} messages"
                                )));
                            }
                            // Check source base paths
                            if let Ok(srcs) = repo::list_sources(&conn) {
                                for src in &srcs {
                                    if let Some(ref bp) = src.base_path {
                                        if !std::path::Path::new(bp).exists() {
                                            checks.push(DoctorCheck::fail(
                                                format!("source '{}' base_path", src.name),
                                                format!("path does not exist: {bp}"),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            checks.push(DoctorCheck::fail(
                                "database readable",
                                format!("cannot open: {e}"),
                            ));
                        }
                    }
                } else {
                    checks.push(DoctorCheck::fail(
                        "database exists",
                        format!("not found at {}", db_path.display()),
                    ));
                }
            }
            Err(e) => {
                checks.push(DoctorCheck::fail(
                    "database path",
                    format!("cannot resolve: {e}"),
                ));
            }
        }

        checks
    }
}

// --- Main ---

fn main() {
    match parse_with_agent_surface::<Cli>(&TOOL_SPEC) {
        Ok(AgentDispatch::Cli(cli)) => {
            let code = run(cli);
            std::process::exit(code);
        }
        Ok(AgentDispatch::Printed(code)) => std::process::exit(code),
        Err(error) => error.exit(),
    }
}

fn run(cli: Cli) -> i32 {
    match cli.command {
        Command::Meta { command } => {
            let standard_command = match command {
                MetaCommand::Version { json } => StandardCommand::Version { json },
                MetaCommand::License => StandardCommand::License,
                MetaCommand::Completions { shell } => StandardCommand::Completions { shell },
                MetaCommand::Doctor { .. } => StandardCommand::Doctor,
            };
            run_standard_command::<Cli, _>(
                &TOOL_SPEC,
                &standard_command,
                Some(&ChronicleDoctorChecks),
            )
        }
        Command::Ingest {
            source,
            path,
            parser,
            full,
            json,
        } => {
            let conn = match open_conn(json) {
                Ok(c) => c,
                Err(code) => return code,
            };

            // Determine parser: explicit flag, or look up from existing source
            let format = match resolve_parser(&conn, &source, parser, json) {
                Ok(f) => f,
                Err(code) => return code,
            };

            // Determine path: explicit flag, or look up from existing source
            let base_path = match resolve_ingest_path(&conn, &source, path.as_deref(), json) {
                Ok(p) => p,
                Err(code) => return code,
            };

            match run_ingest(
                &conn,
                &source,
                format,
                std::path::Path::new(&base_path),
                full,
                None,
            ) {
                Ok(result) => {
                    println!(
                        "{}",
                        render_response(
                            "ingest",
                            json,
                            json!({
                                "source": source,
                                "scanned": result.scanned,
                                "new": result.new,
                                "updated": result.updated,
                                "skipped": result.skipped,
                                "sessions": result.sessions,
                                "messages": result.messages,
                            }),
                            format!(
                                "source: {source}\nscanned: {} | new: {} | updated: {} | skipped: {}\nsessions: {} | messages: {}",
                                result.scanned,
                                result.new,
                                result.updated,
                                result.skipped,
                                result.sessions,
                                result.messages
                            ),
                        )
                    );
                    0
                }
                Err(e) => print_error("ingest", json, &e.to_string()),
            }
        }
        Command::Reindex { source, full, json } => {
            let conn = match open_conn(json) {
                Ok(c) => c,
                Err(code) => return code,
            };
            match run_reindex(&conn, source.as_deref(), full) {
                Ok(results) => {
                    let data: Vec<_> = results
                        .iter()
                        .map(|(name, r)| {
                            json!({
                                "source": name,
                                "scanned": r.scanned,
                                "new": r.new,
                                "updated": r.updated,
                                "skipped": r.skipped,
                                "sessions": r.sessions,
                                "messages": r.messages,
                            })
                        })
                        .collect();
                    let text = results
                        .iter()
                        .map(|(name, r)| {
                            format!(
                                "{name}: scanned={} new={} updated={} skipped={} sessions={} messages={}",
                                r.scanned, r.new, r.updated, r.skipped, r.sessions, r.messages
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    println!(
                        "{}",
                        render_response("reindex", json, json!({"results": data}), text)
                    );
                    0
                }
                Err(e) => print_error("reindex", json, &e.to_string()),
            }
        }
        Command::Search {
            query,
            source,
            role,
            project,
            limit,
            json,
        } => {
            let conn = match open_conn(json) {
                Ok(c) => c,
                Err(code) => return code,
            };
            let role_str = role.map(|r| r.as_str().to_string());
            match run_search(
                &conn,
                &query,
                source.as_deref(),
                role_str.as_deref(),
                project.as_deref(),
                limit,
            ) {
                Ok(results) => {
                    let data: Vec<_> = results
                        .iter()
                        .map(|m| {
                            json!({
                                "session_id": m.session_id,
                                "ordinal": m.ordinal,
                                "role": m.role,
                                "content": m.content,
                                "timestamp": m.timestamp,
                            })
                        })
                        .collect();
                    let text = results
                        .iter()
                        .map(|m| {
                            let preview = if m.content.len() > 120 {
                                format!("{}...", &m.content[..120])
                            } else {
                                m.content.clone()
                            };
                            format!("[{}] {} ({}): {}", m.session_id, m.role, m.ordinal, preview)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    println!(
                        "{}",
                        render_response(
                            "search",
                            json,
                            json!({"count": results.len(), "results": data}),
                            text,
                        )
                    );
                    0
                }
                Err(e) => print_error("search", json, &e.to_string()),
            }
        }
        Command::Stats { source, json } => {
            let conn = match open_conn(json) {
                Ok(c) => c,
                Err(code) => return code,
            };
            match run_stats(&conn, source.as_deref()) {
                Ok(stats) => {
                    let data: Vec<_> = stats
                        .iter()
                        .map(|s| {
                            json!({
                                "name": s.name,
                                "files": s.file_count,
                                "sessions": s.session_count,
                                "messages": s.message_count,
                                "earliest": s.earliest,
                                "latest": s.latest,
                            })
                        })
                        .collect();
                    let text = stats
                        .iter()
                        .map(|s| {
                            format!(
                                "{}: {} files, {} sessions, {} messages ({}..{})",
                                s.name,
                                s.file_count,
                                s.session_count,
                                s.message_count,
                                s.earliest.as_deref().unwrap_or("?"),
                                s.latest.as_deref().unwrap_or("?"),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    println!(
                        "{}",
                        render_response("stats", json, json!({"sources": data}), text)
                    );
                    0
                }
                Err(e) => print_error("stats", json, &e.to_string()),
            }
        }
        Command::Export { session, format } => {
            let conn = match open_conn(false) {
                Ok(c) => c,
                Err(code) => return code,
            };
            match run_export(&conn, &session, format) {
                Ok(output) => {
                    println!("{output}");
                    0
                }
                Err(e) => print_error("export", false, &e.to_string()),
            }
        }
        Command::Sources { command } => match command {
            SourcesCommand::List { json } => {
                let conn = match open_conn(json) {
                    Ok(c) => c,
                    Err(code) => return code,
                };
                match sources::run_list(&conn) {
                    Ok(srcs) => {
                        let data: Vec<_> = srcs
                            .iter()
                            .map(|s| {
                                json!({
                                    "name": s.name,
                                    "parser": s.parser,
                                    "base_path": s.base_path,
                                    "description": s.description,
                                    "created_at": s.created_at,
                                })
                            })
                            .collect();
                        let text = srcs
                            .iter()
                            .map(|s| {
                                format!(
                                    "{} [{}] {}",
                                    s.name,
                                    s.parser,
                                    s.base_path.as_deref().unwrap_or("")
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        println!(
                            "{}",
                            render_response("sources.list", json, json!({"sources": data}), text,)
                        );
                        0
                    }
                    Err(e) => print_error("sources.list", json, &e.to_string()),
                }
            }
            SourcesCommand::Add {
                name,
                parser,
                path,
                description,
                json,
            } => {
                let conn = match open_conn(json) {
                    Ok(c) => c,
                    Err(code) => return code,
                };
                match sources::run_add(&conn, &name, parser, &path, description.as_deref()) {
                    Ok(id) => {
                        println!(
                            "{}",
                            render_response_with(
                                "sources.add",
                                json,
                                json!({"id": id, "name": name, "parser": parser.as_str(), "path": path}),
                                || format!("added source: {name} (id={id})")
                            )
                        );
                        0
                    }
                    Err(e) => print_error("sources.add", json, &e.to_string()),
                }
            }
        },
    }
}

/// Open (or create) the database connection.
fn open_conn(json: bool) -> Result<rusqlite::Connection, i32> {
    let db_path = match resolve_db_path() {
        Ok(p) => p,
        Err(e) => return Err(print_error("db", json, &e.to_string())),
    };
    if let Err(e) = ensure_db_dir(&db_path) {
        return Err(print_error("db", json, &e.to_string()));
    }
    let conn = match open_db(&db_path) {
        Ok(c) => c,
        Err(e) => return Err(print_error("db", json, &e.to_string())),
    };
    if let Err(e) = init_db(&conn) {
        return Err(print_error("db", json, &e.to_string()));
    }
    Ok(conn)
}

/// Resolve parser format from flag or existing source.
fn resolve_parser(
    conn: &rusqlite::Connection,
    source_name: &str,
    explicit: Option<SourceFormat>,
    json: bool,
) -> Result<SourceFormat, i32> {
    if let Some(format) = explicit {
        return Ok(format);
    }
    // Try to look up from existing source
    match repo::get_source_by_name(conn, source_name) {
        Ok(Some(source)) => source
            .parser
            .parse::<SourceFormat>()
            .map_err(|e| print_error("ingest", json, &e)),
        Ok(None) => Err(print_error(
            "ingest",
            json,
            "new source requires --parser flag",
        )),
        Err(e) => Err(print_error("ingest", json, &e.to_string())),
    }
}

/// Resolve ingest path from flag or existing source.
fn resolve_ingest_path(
    conn: &rusqlite::Connection,
    source_name: &str,
    explicit: Option<&str>,
    json: bool,
) -> Result<String, i32> {
    if let Some(path) = explicit {
        return Ok(path.to_string());
    }
    match repo::get_source_by_name(conn, source_name) {
        Ok(Some(source)) => source.base_path.map_or_else(
            || {
                Err(print_error(
                    "ingest",
                    json,
                    "source has no stored base_path; provide --path",
                ))
            },
            Ok,
        ),
        Ok(None) => Err(print_error(
            "ingest",
            json,
            "new source requires --path flag",
        )),
        Err(e) => Err(print_error("ingest", json, &e.to_string())),
    }
}
