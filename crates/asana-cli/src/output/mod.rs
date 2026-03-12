//! Output helpers for rendering command results.

pub mod project;
pub mod task;

use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

/// Supported output formats for project-oriented commands.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum ProjectOutputFormat {
    /// Automatically selected table (default when interactive).
    #[default]
    Table,
    /// JSON representation suitable for scripting.
    Json,
    /// Comma separated value export.
    Csv,
    /// Markdown friendly tables.
    Markdown,
}

impl fmt::Display for ProjectOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Table => "table",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Markdown => "markdown",
        })
    }
}

impl FromStr for ProjectOutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            "markdown" | "md" => Ok(Self::Markdown),
            other => Err(format!(
                "unsupported output format '{other}'; expected table, json, csv, or markdown"
            )),
        }
    }
}

/// Supported output formats for task-oriented commands.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum TaskOutputFormat {
    /// Automatically selected table (default when interactive).
    #[default]
    Table,
    /// JSON representation suitable for scripting.
    Json,
    /// Comma separated value export.
    Csv,
    /// Markdown friendly tables.
    Markdown,
}

impl fmt::Display for TaskOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Table => "table",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Markdown => "markdown",
        })
    }
}

impl FromStr for TaskOutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            "markdown" | "md" => Ok(Self::Markdown),
            other => Err(format!(
                "unsupported output format '{other}'; expected table, json, csv, or markdown"
            )),
        }
    }
}
