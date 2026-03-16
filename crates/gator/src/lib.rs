//! Gator -- agent sandbox harness.
//!
//! Wraps coding agents (Claude, Codex, Gemini) with macOS sandbox-exec
//! integration and prompter-based system prompt composition.

pub mod agent;
pub mod cli;
pub mod config;
pub mod prompt;
pub mod sandbox;
pub mod worktree;
