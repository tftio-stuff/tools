//! Integration tests for the Asana CLI.

use mockito::{Matcher, Server};
use serde_json::{Value as JsonValue, json};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn bin_path() -> String {
    if let Some(value) = std::env::vars().find_map(|(key, value)| {
        if key.starts_with("CARGO_BIN_EXE_asana") {
            Some(value)
        } else {
            None
        }
    }) {
        return value;
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    let binary_name = if cfg!(windows) {
        "asana-cli.exe"
    } else {
        "asana-cli"
    };
    path.push(binary_name);

    if path.exists() {
        return path.to_string_lossy().into_owned();
    }

    panic!("asana-cli binary path environment variable not set");
}

fn run_command(args: &[&str]) -> std::process::Output {
    Command::new(bin_path())
        .args(args)
        .output()
        .expect("failed to execute binary")
}

fn run_command_with_env(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut command = Command::new(bin_path());
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("failed to execute binary")
}

fn standard_env(
    config_home: &TempDir,
    data_home: &TempDir,
    base_url: &str,
) -> Vec<(&'static str, String)> {
    vec![
        (
            "ASANA_CLI_CONFIG_HOME",
            config_home
                .path()
                .to_str()
                .expect("config path utf-8")
                .to_string(),
        ),
        (
            "ASANA_CLI_DATA_HOME",
            data_home
                .path()
                .to_str()
                .expect("data path utf-8")
                .to_string(),
        ),
        ("ASANA_BASE_URL", base_url.to_string()),
    ]
}

#[test]
fn version_command_reports_package_version() {
    let output = run_command(&["version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("asana-cli"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn license_command_mentions_project_license() {
    let output = run_command(&["license"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("asana-cli"));
    assert!(stdout.contains("MIT"));
}

#[test]
fn help_subcommand_mentions_core_commands() {
    let output = run_command(&["help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config"));
    assert!(stdout.contains("task"));
    assert!(stdout.contains("project"));
}

#[test]
fn doctor_runs_with_isolated_home() {
    let temp_home = TempDir::new().expect("temp dir");
    let output = Command::new(bin_path())
        .arg("doctor")
        .env("HOME", temp_home.path())
        .output()
        .expect("failed to execute doctor");
    assert!(
        output.status.success(),
        "doctor failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn config_set_token_and_get_reports_stored_status() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");
    let envs = vec![
        (
            "ASANA_CLI_CONFIG_HOME",
            config_home
                .path()
                .to_str()
                .expect("path should be valid UTF-8")
                .to_string(),
        ),
        (
            "ASANA_CLI_DATA_HOME",
            data_home
                .path()
                .to_str()
                .expect("path should be valid UTF-8")
                .to_string(),
        ),
    ];

    let status_output = run_command_with_env(
        &["config", "set", "token", "--token", "test-token-value"],
        &envs,
    );
    assert!(
        status_output.status.success(),
        "config set token failed: {}",
        String::from_utf8_lossy(&status_output.stderr)
    );

    let get_output = run_command_with_env(&["config", "get"], &envs);
    assert!(
        get_output.status.success(),
        "config get failed: {}",
        String::from_utf8_lossy(&get_output.stderr)
    );
    let stdout = String::from_utf8_lossy(&get_output.stdout);
    assert!(
        stdout.contains("Personal Access Token: stored in configuration file"),
        "unexpected config get output: {stdout}"
    );
}

#[test]
fn config_test_validates_token_against_mock_api() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let _m = server
            .mock("GET", "/users/me")
            .match_header("authorization", "Bearer test-token-value")
            .with_status(200)
            .with_body(r#"{ "data": { "name": "CLI User" } }"#)
            .create();

        let envs = vec![
            (
                "ASANA_CLI_CONFIG_HOME",
                config_home
                    .path()
                    .to_str()
                    .expect("path should be valid UTF-8")
                    .to_string(),
            ),
            (
                "ASANA_CLI_DATA_HOME",
                data_home
                    .path()
                    .to_str()
                    .expect("path should be valid UTF-8")
                    .to_string(),
            ),
            ("ASANA_BASE_URL", server.url()),
        ];

        let set_output = run_command_with_env(
            &["config", "set", "token", "--token", "test-token-value"],
            &envs,
        );
        assert!(
            set_output.status.success(),
            "config set token failed: {}",
            String::from_utf8_lossy(&set_output.stderr)
        );

        let test_output = run_command_with_env(&["config", "test"], &envs);
        assert!(
            test_output.status.success(),
            "config test failed: {}",
            String::from_utf8_lossy(&test_output.stderr)
        );
        let stdout = String::from_utf8_lossy(&test_output.stdout);
        assert!(stdout.contains("Personal Access Token validated"));
    }
}

#[test]
fn project_list_outputs_json() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let _list = server
            .mock("GET", "/projects")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(
                r#"{
                    "data": [
                        {
                            "gid": "P1",
                            "name": "Alpha",
                            "archived": false,
                            "workspace": { "gid": "W1", "name": "Engineering" },
                            "owner": { "gid": "U1", "name": "User" },
                            "modified_at": "2025-01-01T00:00:00Z"
                        }
                    ]
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());

        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "test-token"], &envs);
        assert!(set_output.status.success());

        let list_output = run_command_with_env(&["project", "list", "--output", "json"], &envs);
        assert!(
            list_output.status.success(),
            "project list failed: {:?}\nstdout:\n{}\nstderr:\n{}",
            list_output.status,
            String::from_utf8_lossy(&list_output.stdout),
            String::from_utf8_lossy(&list_output.stderr)
        );
        let stdout = String::from_utf8_lossy(&list_output.stdout);
        assert!(
            stdout.contains("\"name\": \"Alpha\""),
            "unexpected stdout: {stdout}"
        );
    }
}

#[test]
fn project_show_includes_members() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let _project = server
            .mock("GET", "/projects/123")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(
                r#"{
                    "data": {
                        "gid": "123",
                        "name": "Project X",
                        "archived": false,
                        "workspace": { "gid": "W1", "name": "Engineering" },
                        "owner": { "gid": "U1", "name": "Owner" }
                    }
                }"#,
            )
            .create();
        let _members = server
            .mock("GET", "/projects/123/members")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(
                r#"{
                    "data": [
                        {
                            "gid": "M1",
                            "user": { "gid": "U2", "name": "Contributor" },
                            "role": "commenter"
                        }
                    ]
                }"#,
            )
            .create();
        let _statuses = server
            .mock("GET", "/projects/123/project_statuses")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(
                r#"{
                    "data": [
                        {
                            "gid": "S1",
                            "title": "On Track",
                            "text": "Milestones are green",
                            "created_at": "2025-10-16T12:00:00Z",
                            "created_by": { "gid": "U2", "name": "Contributor" }
                        }
                    ]
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "test-token"], &envs);
        assert!(set_output.status.success());

        let show_output = run_command_with_env(
            &[
                "project",
                "show",
                "123",
                "--include-members",
                "--output",
                "json",
            ],
            &envs,
        );
        assert!(
            show_output.status.success(),
            "project show failed: {:?}\nstdout:\n{}\nstderr:\n{}",
            show_output.status,
            String::from_utf8_lossy(&show_output.stdout),
            String::from_utf8_lossy(&show_output.stderr)
        );
        let stdout = String::from_utf8_lossy(&show_output.stdout);
        assert!(
            stdout.contains("\"name\": \"Project X\""),
            "unexpected stdout: {stdout}"
        );
        assert!(stdout.contains("Project X"), "unexpected stdout: {stdout}");
        assert!(
            stdout.contains("Contributor"),
            "unexpected stdout: {stdout}"
        );
    }
}

#[test]
fn task_list_fetches_tasks() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let list_mock = server
            .mock("GET", "/tasks")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("workspace".into(), "ws-123".into()),
                Matcher::UrlEncoded("assignee".into(), "me".into()),
            ]))
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(
                r#"{
                    "data": [
                        {
                            "gid": "T1",
                            "name": "Inbox Task",
                            "completed": false
                        }
                    ]
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());

        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let list_output = run_command_with_env(
            &["task", "list", "--workspace", "ws-123", "--output", "json"],
            &envs,
        );
        assert!(
            list_output.status.success(),
            "task list failed: {}",
            String::from_utf8_lossy(&list_output.stderr)
        );
        let stdout = String::from_utf8_lossy(&list_output.stdout);
        let payload: JsonValue = serde_json::from_str(&stdout).expect("task list JSON");
        assert_eq!(payload[0]["gid"], "T1");
        assert_eq!(payload[0]["name"], "Inbox Task");
        list_mock.assert();
    }
}

#[test]
fn task_create_posts_expected_payload() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let create_mock = server
            .mock("POST", "/tasks")
            .match_header("authorization", "Bearer task-token")
            .match_body(Matcher::PartialJson(json!({
                "data": {
                    "name": "CLI Task",
                    "workspace": "ws-123"
                }
            })))
            .with_status(200)
            .with_body(
                r#"{
                    "data": {
                        "gid": "T42",
                        "name": "CLI Task",
                        "completed": false
                    }
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let create_output = run_command_with_env(
            &[
                "task",
                "create",
                "--name",
                "CLI Task",
                "--workspace",
                "ws-123",
                "--output",
                "json",
            ],
            &envs,
        );
        assert!(
            create_output.status.success(),
            "task create failed: {}",
            String::from_utf8_lossy(&create_output.stderr)
        );
        let stdout = String::from_utf8_lossy(&create_output.stdout);
        let payload: JsonValue = serde_json::from_str(&stdout).expect("task create JSON");
        assert_eq!(payload["gid"], "T42");
        assert_eq!(payload["name"], "CLI Task");
        create_mock.assert();
    }
}

#[test]
fn task_update_sends_changes() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let update_mock = server
            .mock("PUT", "/tasks/T1")
            .match_header("authorization", "Bearer task-token")
            .match_body(Matcher::PartialJson(json!({
                "data": {
                    "name": "Updated Task",
                    "completed": true
                }
            })))
            .with_status(200)
            .with_body(
                r#"{
                    "data": {
                        "gid": "T1",
                        "name": "Updated Task",
                        "completed": true
                    }
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let update_output = run_command_with_env(
            &[
                "task",
                "update",
                "T1",
                "--name",
                "Updated Task",
                "--complete",
                "--output",
                "json",
            ],
            &envs,
        );
        assert!(
            update_output.status.success(),
            "task update failed: {}",
            String::from_utf8_lossy(&update_output.stderr)
        );
        let stdout = String::from_utf8_lossy(&update_output.stdout);
        let payload: JsonValue = serde_json::from_str(&stdout).expect("task update JSON");
        assert_eq!(payload["completed"], true);
        assert_eq!(payload["name"], "Updated Task");
        update_mock.assert();
    }
}

#[test]
fn task_subtasks_list_fetches_children() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let list_mock = server
            .mock("GET", Matcher::Regex(r"^/tasks/T1/subtasks".to_string()))
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(
                r#"{
                    "data": [
                        { "gid": "ST1", "name": "Child", "completed": false }
                    ]
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let list_output = run_command_with_env(
            &["task", "subtasks", "list", "T1", "--output", "json"],
            &envs,
        );
        assert!(
            list_output.status.success(),
            "subtasks list failed: {}",
            String::from_utf8_lossy(&list_output.stderr)
        );
        let stdout = String::from_utf8_lossy(&list_output.stdout);
        let payload: JsonValue = serde_json::from_str(&stdout).expect("subtask list JSON");
        assert_eq!(payload[0]["task"]["gid"], "ST1");
        list_mock.assert();
    }
}

#[test]
fn task_subtasks_create_sets_parent() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let create_mock = server
            .mock("POST", "/tasks")
            .match_header("authorization", "Bearer task-token")
            .match_body(Matcher::PartialJson(json!({
                "data": {
                    "name": "Child",
                    "parent": "T1"
                }
            })))
            .with_status(200)
            .with_body(
                r#"{
                    "data": { "gid": "ST1", "name": "Child", "completed": false }
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let create_output = run_command_with_env(
            &[
                "task", "subtasks", "create", "T1", "--name", "Child", "--output", "json",
            ],
            &envs,
        );
        assert!(
            create_output.status.success(),
            "subtasks create failed: {}",
            String::from_utf8_lossy(&create_output.stderr)
        );
        create_mock.assert();
    }
}

#[test]
fn task_subtasks_convert_updates_parent() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let convert_mock = server
            .mock("PUT", "/tasks/T2")
            .match_body(Matcher::PartialJson(json!({
                "data": { "parent": "T1" }
            })))
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(
                r#"{
                    "data": { "gid": "T2", "name": "Converted", "completed": false }
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let convert_output = run_command_with_env(
            &["task", "subtasks", "convert", "T2", "--parent", "T1"],
            &envs,
        );
        assert!(convert_output.status.success());
        convert_mock.assert();
    }
}

#[test]
fn task_dependencies_add_hits_endpoint() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let dependency_mock = server
            .mock("POST", "/tasks/T1/addDependencies")
            .match_body(Matcher::PartialJson(json!({
                "data": { "dependencies": ["D1", "D2"] }
            })))
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let output = run_command_with_env(
            &[
                "task",
                "depends-on",
                "add",
                "T1",
                "--dependency",
                "D1",
                "--dependency",
                "D2",
            ],
            &envs,
        );
        assert!(output.status.success());
        dependency_mock.assert();
    }
}

#[test]
fn task_dependents_list_outputs() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let list_mock = server
            .mock("GET", "/tasks/T1/dependents")
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(
                r#"{
                    "data": [
                        { "gid": "B1", "name": "Blocked" }
                    ]
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let output =
            run_command_with_env(&["task", "blocks", "list", "T1", "--output", "json"], &envs);
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        let payload: JsonValue = serde_json::from_str(&stdout).expect("dependents JSON");
        assert_eq!(payload[0]["gid"], "B1");
        list_mock.assert();
    }
}

#[test]
fn task_projects_add_and_remove() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let add_mock = server
            .mock("POST", "/tasks/T1/addProject")
            .match_header("authorization", "Bearer task-token")
            .match_body(Matcher::PartialJson(json!({
                "data": { "project": "P1", "section": "S1" }
            })))
            .with_status(200)
            .create();

        let remove_mock = server
            .mock("POST", "/tasks/T1/removeProject")
            .match_header("authorization", "Bearer task-token")
            .match_body(Matcher::PartialJson(json!({
                "data": { "project": "P1" }
            })))
            .with_status(200)
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let add_output = run_command_with_env(
            &[
                "task",
                "projects",
                "add",
                "T1",
                "--project",
                "P1",
                "--section",
                "S1",
            ],
            &envs,
        );
        assert!(add_output.status.success());

        let remove_output = run_command_with_env(
            &["task", "projects", "remove", "T1", "--project", "P1"],
            &envs,
        );
        assert!(remove_output.status.success());

        add_mock.assert();
        remove_mock.assert();
    }
}

#[test]
fn task_followers_add_and_remove() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let add_mock = server
            .mock("POST", "/tasks/T1/addFollowers")
            .match_header("authorization", "Bearer task-token")
            .match_body(Matcher::PartialJson(json!({
                "data": { "followers": ["U1"] }
            })))
            .with_status(200)
            .create();

        let remove_mock = server
            .mock("POST", "/tasks/T1/removeFollowers")
            .match_header("authorization", "Bearer task-token")
            .match_body(Matcher::PartialJson(json!({
                "data": { "followers": ["U1", "U2"] }
            })))
            .with_status(200)
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let add_output = run_command_with_env(
            &["task", "followers", "add", "T1", "--follower", "U1"],
            &envs,
        );
        assert!(add_output.status.success());

        let remove_output = run_command_with_env(
            &[
                "task",
                "followers",
                "remove",
                "T1",
                "--follower",
                "U1",
                "--follower",
                "U2",
            ],
            &envs,
        );
        assert!(remove_output.status.success());

        add_mock.assert();
        remove_mock.assert();
    }
}

#[test]
fn task_create_batch_processes_json() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let create_mock = server
            .mock("POST", "/tasks")
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(
                r#"{
                    "data": { "gid": "T100", "name": "Batch A", "completed": false }
                }"#,
            )
            .expect(2)
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let batch_path = data_home.path().join("create.json");
        fs::write(
            &batch_path,
            r#"[
                { "name": "Batch A", "workspace": "ws-123" },
                { "name": "Batch B", "projects": ["P1"] }
            ]"#,
        )
        .expect("write batch file");

        let output = run_command_with_env(
            &[
                "task",
                "create-batch",
                "--file",
                batch_path.to_str().unwrap(),
                "--format",
                "json",
                "--output",
                "json",
            ],
            &envs,
        );
        assert!(
            output.status.success(),
            "create-batch failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        create_mock.assert();
    }
}

#[test]
fn task_update_batch_processes_json() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let update_mock = server
            .mock("PUT", "/tasks/T1")
            .match_header("authorization", "Bearer task-token")
            .match_body(Matcher::PartialJson(json!({
                "data": { "name": "Updated Task" }
            })))
            .with_status(200)
            .with_body(
                r#"{
                    "data": { "gid": "T1", "name": "Updated Task", "completed": false }
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let batch_path = data_home.path().join("update.json");
        fs::write(
            &batch_path,
            r#"[
                {
                    "task": "T1",
                    "name": "Updated Task",
                    "completed": true,
                    "tags": ["tag-1", "tag-2"]
                }
            ]"#,
        )
        .expect("write batch file");

        let output = run_command_with_env(
            &[
                "task",
                "update-batch",
                "--file",
                batch_path.to_str().unwrap(),
                "--format",
                "json",
                "--output",
                "json",
            ],
            &envs,
        );
        assert!(
            output.status.success(),
            "update-batch failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        update_mock.assert();
    }
}

#[test]
fn task_complete_batch_marks_tasks() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let complete_mock = server
            .mock("PUT", "/tasks/T9")
            .match_header("authorization", "Bearer task-token")
            .match_body(Matcher::PartialJson(json!({
                "data": { "completed": true }
            })))
            .with_status(200)
            .with_body(
                r#"{
                    "data": { "gid": "T9", "name": "Batch Complete", "completed": true }
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let batch_path = data_home.path().join("complete.csv");
        fs::write(&batch_path, "task,completed\nT9,true\n").expect("write batch file");

        let output = run_command_with_env(
            &[
                "task",
                "complete-batch",
                "--file",
                batch_path.to_str().unwrap(),
                "--format",
                "csv",
                "--output",
                "json",
            ],
            &envs,
        );
        assert!(
            output.status.success(),
            "complete-batch failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        complete_mock.assert();
    }
}

#[test]
fn task_search_returns_matches() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let search_mock = server
            .mock("GET", "/workspaces/ws-123/tasks/search")
            .match_query(Matcher::AllOf(vec![Matcher::UrlEncoded(
                "text".into(),
                "Alpha".into(),
            )]))
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(
                r#"{
                    "data": [
                        { "gid": "T1", "name": "Alpha Task", "completed": false },
                        { "gid": "T2", "name": "Another Alpha", "completed": false }
                    ]
                }"#,
            )
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let output = run_command_with_env(
            &[
                "task",
                "search",
                "--query",
                "Alpha",
                "--workspace",
                "ws-123",
                "--limit",
                "5",
                "--output",
                "json",
            ],
            &envs,
        );
        assert!(
            output.status.success(),
            "task search failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Alpha"));
        search_mock.assert();
    }
}

#[test]
fn task_search_recent_only_uses_cache() {
    let config_home = TempDir::new().expect("config home");
    let data_home = TempDir::new().expect("data home");

    {
        let mut server = Server::new();
        let task_mock = server
            .mock("GET", Matcher::Regex(r"^/tasks/T500(?:\?.*)?$".to_string()))
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(
                r#"{
                    "data": { "gid": "T500", "name": "Recent Task", "completed": false }
                }"#,
            )
            .create();
        let _subtasks_mock = server
            .mock(
                "GET",
                Matcher::Regex(r"^/tasks/T500/subtasks(?:\?.*)?$".to_string()),
            )
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(r#"{ "data": [] }"#)
            .create();
        let _dependencies_mock = server
            .mock(
                "GET",
                Matcher::Regex(r"^/tasks/T500/dependencies(?:\?.*)?$".to_string()),
            )
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(r#"{ "data": [] }"#)
            .create();
        let _dependents_mock = server
            .mock(
                "GET",
                Matcher::Regex(r"^/tasks/T500/dependents(?:\?.*)?$".to_string()),
            )
            .match_header("authorization", "Bearer task-token")
            .with_status(200)
            .with_body(r#"{ "data": [] }"#)
            .create();

        let envs = standard_env(&config_home, &data_home, &server.url());
        let set_output =
            run_command_with_env(&["config", "set", "token", "--token", "task-token"], &envs);
        assert!(set_output.status.success());

        let show_output =
            run_command_with_env(&["task", "show", "T500", "--output", "json"], &envs);
        if !show_output.status.success() {
            println!(
                "task show stderr: {}",
                String::from_utf8_lossy(&show_output.stderr)
            );
        }
        assert!(show_output.status.success());
        task_mock.assert();

        let recent_output = run_command_with_env(
            &["task", "search", "--recent-only", "--output", "json"],
            &envs,
        );
        assert!(recent_output.status.success());
        let stdout = String::from_utf8_lossy(&recent_output.stdout);
        assert!(stdout.contains("Recent Task"));
    }
}
