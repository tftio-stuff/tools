//! Common error handling helpers for the Asana CLI.

/// Result type alias leveraging `anyhow` for rich context.
pub type Result<T> = anyhow::Result<T>;
