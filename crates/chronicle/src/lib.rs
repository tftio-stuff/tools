//! Chronicle: interaction history corpus indexer.
//!
//! Indexes interaction histories from coding agents (Codex, Claude Code) into
//! a queryable `SQLite` database. Provides a discovery surface for agentic
//! analysis of past interactions.

pub mod cli;
pub mod commands;
pub mod config;
pub mod db;
pub mod models;
pub mod parsers;
pub mod repo;
pub mod strip;
