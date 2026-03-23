use crate::models::Status;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "todoer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Meta {
        #[command(subcommand)]
        command: MetaCommand,
    },
    Init {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        json: bool,
    },
    New {
        #[arg(long)]
        project: Option<String>,
        description: String,
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        json: bool,
    },
    Task {
        #[command(subcommand)]
        command: TaskCommand,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskCommand {
    Status {
        id: String,
    },
    Show {
        id: String,
    },
    Note {
        id: String,
        note: String,
    },
    Update {
        #[command(subcommand)]
        command: TaskUpdateCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum TaskUpdateCommand {
    Status { id: String, status: Status },
}

#[derive(Subcommand, Debug)]
pub enum MetaCommand {
    Version {
        #[arg(long)]
        json: bool,
    },
    License,
    Completions {
        shell: clap_complete::Shell,
    },
}
