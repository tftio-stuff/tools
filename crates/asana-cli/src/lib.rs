//! Core library for the Asana CLI application.

/// Asana API client abstractions.
pub mod api;
/// Command-line interface components.
pub mod cli;
/// Configuration management utilities.
pub mod config;
/// Health check integrations.
pub mod doctor;
/// Error handling helpers.
pub mod error;
/// Filter parsing and persistence.
pub mod filters;
/// Shared data models.
pub mod models;
/// Output rendering helpers.
pub mod output;
/// User configurable templates.
pub mod templates;

use crate::error::Result;
use anyhow::anyhow;
use tracing_subscriber::{EnvFilter, fmt};

/// Initialize global tracing with sensible defaults.
///
/// # Errors
/// Returns an error if the tracing subscriber cannot be initialised.
pub fn init_tracing() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init()
        .map_err(|err| anyhow!(err))?;

    Ok(())
}
