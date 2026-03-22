use clap::Parser;
use todoer::cli::{Cli, Command, MetaCommand, TaskCommand, TaskUpdateCommand};

#[test]
fn parse_list_all_flag() {
    let cli = Cli::parse_from(["todoer", "list", "--all"]);
    match cli.command {
        Command::List { all, project, json } => {
            assert!(all);
            assert!(project.is_none());
            assert!(!json);
        }
        _ => panic!("expected list command"),
    }
}

#[test]
fn parse_task_update_status() {
    let cli = Cli::parse_from(["todoer", "task", "update", "status", "123", "COMPLETED"]);
    match cli.command {
        Command::Task { command, json } => {
            assert!(!json);
            match command {
                TaskCommand::Update { command } => match command {
                    TaskUpdateCommand::Status { id, status } => {
                        assert_eq!(id, "123");
                        assert_eq!(status.as_str(), "COMPLETED");
                    }
                },
                _ => panic!("expected update status"),
            }
        }
        _ => panic!("expected task command"),
    }
}

#[test]
fn parse_meta_version_json() {
    let cli = Cli::parse_from(["todoer", "meta", "version", "--json"]);
    match cli.command {
        Command::Meta { command } => match command {
            MetaCommand::Version { json } => assert!(json),
            _ => panic!("expected meta version command"),
        },
        _ => panic!("expected meta command"),
    }
}
