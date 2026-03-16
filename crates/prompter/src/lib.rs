//! Prompter: A CLI tool for composing reusable prompt snippets.
//!
//! This library provides functionality for managing and rendering prompt snippets
//! from a structured library using TOML configuration files. It supports recursive
//! profile dependencies, file deduplication, and customizable output formatting.

pub mod completions;

use chrono::Local;
use clap::{Parser, Subcommand};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use is_terminal::IsTerminal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Configuration structure holding profile definitions and their dependencies.
///
/// Profiles map names to lists of dependencies, where dependencies can be either
/// markdown files (ending in .md) or references to other profiles.
#[derive(Debug)]
pub struct Config {
    /// Map of profile names to their dependency lists
    pub(crate) profiles: HashMap<String, Vec<String>>,
    /// Optional post-prompt text to append at the end of output
    pub(crate) post_prompt: Option<String>,
}

/// Command-line interface structure for the prompter tool.
///
/// This structure defines the main CLI interface using clap's derive API.
#[derive(Parser, Debug)]
#[command(name = "prompter")]
#[command(about = "A CLI tool for composing reusable prompt snippets")]
#[command(version)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,

    /// Override configuration file path
    #[arg(short = 'c', long, value_name = "FILE", global = true)]
    pub config: Option<PathBuf>,

    /// Output in JSON format
    #[arg(short = 'j', long, global = true)]
    pub json: bool,
}

/// Available subcommands for the prompter CLI.
///
/// Each variant represents a different operation mode of the tool.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show version information
    Version,
    /// Show license information
    License,
    /// Initialize default config and library
    Init,
    /// List available profiles
    List,
    /// Show dependency tree for profiles
    Tree,
    /// Validate configuration and library references
    Validate,
    /// Render one or more profiles (concatenated file contents with deduplication)
    Run {
        /// Profile name(s) to render
        #[arg(required = true)]
        profiles: Vec<String>,
        /// Separator between files
        #[arg(short, long)]
        separator: Option<String>,
        /// Pre-prompt text to inject at the beginning
        #[arg(short = 'p', long)]
        pre_prompt: Option<String>,
        /// Post-prompt text to inject at the end
        #[arg(short = 'P', long)]
        post_prompt: Option<String>,
    },
    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Check health and configuration status
    Doctor,
}

/// Application execution modes after parsing command-line arguments.
///
/// This enum represents the resolved execution mode after processing
/// both subcommands and direct profile arguments.
#[derive(Debug)]
pub enum AppMode {
    /// Render one or more profiles with optional separator and pre-prompt
    Run {
        /// Profile name(s) to render
        profiles: Vec<String>,
        /// Optional separator between concatenated files
        separator: Option<String>,
        /// Optional custom pre-prompt text
        pre_prompt: Option<String>,
        /// Optional custom post-prompt text
        post_prompt: Option<String>,
        /// Optional configuration file override
        config: Option<PathBuf>,
        /// Output in JSON format
        json: bool,
    },
    /// List all available profiles using an optional config override
    List {
        /// Optional configuration file override
        config: Option<PathBuf>,
        /// Output in JSON format
        json: bool,
    },
    /// Show dependency tree for profiles
    Tree {
        /// Optional configuration file override
        config: Option<PathBuf>,
        /// Output in JSON format
        json: bool,
    },
    /// Validate configuration and library references with an optional config override
    Validate {
        /// Optional configuration file override
        config: Option<PathBuf>,
        /// Output in JSON format
        json: bool,
    },
    /// Initialize default configuration and library
    Init,
    /// Show version information
    Version {
        /// Output in JSON format
        json: bool,
    },
    /// Show license information
    License,
    /// Show help information
    Help,
    /// Generate shell completion scripts
    Completions {
        /// Shell to generate completions for
        shell: clap_complete::Shell,
    },
    /// Check health and configuration status
    Doctor {
        /// Output in JSON format
        json: bool,
    },
}

/// Parse command-line arguments and return the resolved application mode.
///
/// This function takes raw command-line arguments and uses clap to parse them
/// into a structured `AppMode` enum.
///
/// # Arguments
/// * `args` - Vector of command-line arguments including program name
///
/// # Returns
/// * `Ok(AppMode)` - Successfully parsed application mode
/// * `Err(String)` - Error message if parsing fails
///
/// # Errors
/// Returns an error if:
/// - Invalid command-line syntax is provided
/// - Required arguments are missing
/// - Conflicting options are specified
pub fn parse_args_from(args: Vec<String>) -> Result<AppMode, String> {
    let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;

    match cli.command {
        Commands::Version => Ok(AppMode::Version { json: cli.json }),
        Commands::License => Ok(AppMode::License),
        Commands::Init => Ok(AppMode::Init),
        Commands::List => Ok(AppMode::List {
            config: cli.config,
            json: cli.json,
        }),
        Commands::Tree => Ok(AppMode::Tree {
            config: cli.config,
            json: cli.json,
        }),
        Commands::Validate => Ok(AppMode::Validate {
            config: cli.config,
            json: cli.json,
        }),
        Commands::Completions { shell } => Ok(AppMode::Completions { shell }),
        Commands::Doctor => Ok(AppMode::Doctor { json: cli.json }),
        Commands::Run {
            profiles,
            separator,
            pre_prompt,
            post_prompt,
        } => {
            let sep = separator.as_ref().map(|s| unescape(s));
            let pre = pre_prompt.as_ref().map(|s| unescape(s));
            let post = post_prompt.as_ref().map(|s| unescape(s));
            Ok(AppMode::Run {
                profiles,
                separator: sep,
                pre_prompt: pre,
                post_prompt: post,
                config: cli.config,
                json: cli.json,
            })
        }
    }
}

/// Unescape special characters in strings.
///
/// Processes escape sequences like `\n`, `\t`, `\"`, and `\\` in input strings,
/// converting them to their literal character equivalents.
///
/// # Arguments
/// * `s` - Input string that may contain escape sequences
///
/// # Returns
/// String with escape sequences converted to literal characters
///
/// # Examples
/// ```
/// use prompter::unescape;
/// assert_eq!(unescape("line1\\nline2"), "line1\nline2");
/// ```
#[must_use]
#[allow(clippy::while_let_on_iterator)]
pub fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn home_dir() -> Result<PathBuf, String> {
    env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| "$HOME not set".into())
}

fn config_path() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".config/prompter/config.toml"))
}

fn library_dir() -> Result<PathBuf, String> {
    Ok(home_dir()?.join(".local/prompter/library"))
}

fn config_path_override(path: &Path) -> Result<PathBuf, String> {
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|e| format!("Failed to resolve working directory: {e}"))?
            .join(path)
    };
    Ok(resolved)
}

fn library_dir_for_config(config: &Path) -> Result<PathBuf, String> {
    let parent = config
        .parent()
        .ok_or_else(|| format!("Config path {} has no parent directory", config.display()))?;
    Ok(parent.join("library"))
}

fn is_terminal() -> bool {
    std::io::stdout().is_terminal()
}

fn default_pre_prompt() -> String {
    "You are an LLM coding agent. Here are invariants that you must adhere to. Please respond with 'Got it' when you have studied these and understand them. At that point, the operator will give you further instructions. You are *not* to do anything to the contents of this directory until you have been explicitly asked to, by the operator.\n\n".to_string()
}

fn default_post_prompt() -> String {
    "Now, read the @AGENTS.md and @CLAUDE.md files in this directory, if they exist.".to_string()
}

fn format_system_prefix() -> String {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    if is_terminal() {
        format!(
            "🗓️  Today is {}, and you are running on a {}/{} system.\n\n",
            date.bright_cyan(),
            arch.bright_green(),
            os.bright_green()
        )
    } else {
        format!("Today is {date}, and you are running on a {arch}/{os} system.\n\n")
    }
}

fn success_message(msg: &str) -> String {
    if is_terminal() {
        format!("✅ {}", msg.bright_green())
    } else {
        msg.to_string()
    }
}

fn info_message(msg: &str) -> String {
    if is_terminal() {
        format!("ℹ️  {}", msg.bright_blue())
    } else {
        msg.to_string()
    }
}

fn read_config_with_path(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))
}

fn resolve_config_path(config_override: Option<&Path>) -> Result<PathBuf, String> {
    config_override.map_or_else(config_path, config_path_override)
}

fn library_path_for_config_override(
    config_override: Option<&Path>,
    resolved_config: &Path,
) -> Result<PathBuf, String> {
    if config_override.is_some() {
        library_dir_for_config(resolved_config)
    } else {
        library_dir()
    }
}

/// Parse TOML configuration into a Config structure.
///
/// Processes TOML input containing profile definitions and their dependencies,
/// handling multi-line arrays and comment stripping.
///
/// # Arguments
/// * `input` - TOML configuration text
///
/// # Returns
/// * `Ok(Config)` - Successfully parsed configuration
/// * `Err(String)` - Error message describing parsing failure
///
/// # Errors
/// Returns an error if:
/// - TOML syntax is invalid
/// - Profile sections are malformed
/// - `depends_on` arrays have invalid syntax
pub fn parse_config_toml(input: &str) -> Result<Config, String> {
    let mut profiles: HashMap<String, Vec<String>> = HashMap::new();
    let mut current: Option<String> = None;
    let mut post_prompt: Option<String> = None;

    let mut collecting = false;
    let mut buffer = String::new();

    for raw_line in input.lines() {
        let line = strip_comments(raw_line).trim().to_string();
        if line.is_empty() {
            continue;
        }

        if collecting {
            buffer.push(' ');
            buffer.push_str(&line);
            if contains_closing_bracket_outside_quotes(&buffer) {
                let items = parse_array_items(&buffer).map_err(|e| {
                    format!(
                        "Invalid depends_on array for [{}]: {}",
                        current.clone().unwrap_or_default(),
                        e
                    )
                })?;
                let name = current
                    .clone()
                    .ok_or_else(|| "depends_on outside of a profile section".to_string())?;
                profiles.insert(name, items);
                collecting = false;
                buffer.clear();
            }
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let name = line[1..line.len() - 1].trim().to_string();
            if name.is_empty() {
                return Err("Empty section name []".into());
            }
            current = Some(name);
            continue;
        }

        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();

            if key == "post_prompt" {
                if !value.starts_with('"') || !value.ends_with('"') {
                    return Err("post_prompt must be a string".into());
                }
                let unquoted = &value[1..value.len() - 1];
                post_prompt = Some(unescape(unquoted));
                continue;
            }

            if key != "depends_on" {
                continue;
            }
            if !value.starts_with('[') {
                return Err("depends_on must be an array".into());
            }
            buffer.clear();
            buffer.push_str(value);
            if contains_closing_bracket_outside_quotes(&buffer) {
                let items = parse_array_items(&buffer).map_err(|e| {
                    format!(
                        "Invalid depends_on array for [{}]: {}",
                        current.clone().unwrap_or_default(),
                        e
                    )
                })?;
                let name = current
                    .clone()
                    .ok_or_else(|| "depends_on outside of a profile section".to_string())?;
                profiles.insert(name, items);
                buffer.clear();
            } else {
                collecting = true;
            }
        }
    }

    Ok(Config {
        profiles,
        post_prompt,
    })
}

fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_str = false;
    for c in s.chars() {
        if c == '"' {
            out.push(c);
            in_str = !in_str;
            continue;
        }
        if !in_str && c == '#' {
            break;
        }
        out.push(c);
    }
    out
}

fn contains_closing_bracket_outside_quotes(s: &str) -> bool {
    let mut in_str = false;
    for c in s.chars() {
        if c == '"' {
            in_str = !in_str;
        }
        if !in_str && c == ']' {
            return true;
        }
    }
    false
}

fn parse_array_items(s: &str) -> Result<Vec<String>, String> {
    let mut items = Vec::new();
    let mut in_str = false;
    let mut buf = String::new();
    let mut escaped = false;
    let mut started = false;

    for c in s.chars() {
        if !started {
            if c == '[' {
                started = true;
            }
            continue;
        }
        if c == ']' && !in_str {
            break;
        }
        if in_str {
            if escaped {
                buf.push(c);
                escaped = false;
                continue;
            }
            if c == '\\' {
                escaped = true;
                continue;
            }
            if c == '"' {
                in_str = false;
                items.push(buf.clone());
                buf.clear();
                continue;
            }
            buf.push(c);
        } else if c == '"' {
            in_str = true;
        }
    }

    if in_str {
        return Err("Unterminated string in array".into());
    }
    Ok(items)
}

/// Errors that can occur during profile resolution.
///
/// These errors represent various failure modes when resolving
/// profile dependencies and validating file references.
#[derive(Debug, PartialEq, Eq)]
pub enum ResolveError {
    /// Referenced profile name does not exist in configuration
    UnknownProfile(String),
    /// Circular dependency detected in profile references
    Cycle(Vec<String>),
    /// Referenced markdown file does not exist
    MissingFile(PathBuf, String), // (path, referenced_by)
}

/// Node type in the dependency tree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TreeNodeType {
    /// Profile node
    Profile,
    /// Fragment (markdown file) node
    Fragment,
}

/// Tree node representing a profile or fragment in the dependency tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    /// Type of node (profile or fragment)
    #[serde(rename = "type")]
    pub node_type: TreeNodeType,
    /// Name of profile or path of fragment
    pub name: String,
    /// Children of this node
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TreeNode>,
}

/// Complete tree structure for JSON output
#[derive(Debug, Serialize, Deserialize)]
pub struct TreeOutput {
    /// List of root trees (top-level profiles)
    pub trees: Vec<TreeNode>,
}

/// Recursively resolve a profile's dependencies into a list of file paths.
///
/// Performs depth-first traversal of profile dependencies, handling both
/// direct file references and recursive profile dependencies. Implements
/// cycle detection and file deduplication.
///
/// # Arguments
/// * `name` - Profile name to resolve
/// * `cfg` - Configuration containing profile definitions
/// * `lib` - Library root directory for resolving file paths
/// * `seen_files` - Set tracking already included files for deduplication
/// * `stack` - Stack for cycle detection during recursion
/// * `out` - Output vector to collect resolved file paths
///
/// # Returns
/// * `Ok(())` - Profile successfully resolved
/// * `Err(ResolveError)` - Resolution failed due to missing files, cycles, or unknown profiles
///
/// # Errors
/// Returns an error if:
/// - Profile name is not found in configuration
/// - Circular dependency is detected
/// - Referenced markdown file does not exist
#[allow(clippy::implicit_hasher)]
pub fn resolve_profile(
    name: &str,
    cfg: &Config,
    lib: &Path,
    seen_files: &mut HashSet<PathBuf>,
    stack: &mut Vec<String>,
    out: &mut Vec<PathBuf>,
) -> Result<(), ResolveError> {
    if stack.contains(&name.to_string()) {
        let mut cycle = stack.clone();
        cycle.push(name.to_string());
        return Err(ResolveError::Cycle(cycle));
    }
    let deps = cfg
        .profiles
        .get(name)
        .ok_or_else(|| ResolveError::UnknownProfile(name.to_string()))?;
    stack.push(name.to_string());
    for dep in deps {
        if std::path::Path::new(dep)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            let path = lib.join(dep);
            if !path.exists() {
                return Err(ResolveError::MissingFile(path, name.to_string()));
            }
            if seen_files.insert(path.clone()) {
                out.push(path);
            }
        } else {
            resolve_profile(dep, cfg, lib, seen_files, stack, out)?;
        }
    }
    stack.pop();
    Ok(())
}

/// JSON output structure for list command
#[derive(Debug, Serialize)]
struct ListOutput {
    profiles: Vec<ProfileInfo>,
    fragments: Vec<String>,
}

/// Profile information for JSON output
#[derive(Debug, Serialize)]
struct ProfileInfo {
    name: String,
    dependencies: Vec<String>,
}

/// List all available profiles to a writer.
///
/// Outputs all profile names from the configuration in alphabetical order,
/// one per line (text mode) or as JSON (json mode).
///
/// # Arguments
/// * `cfg` - Configuration containing profile definitions
/// * `lib` - Library root directory for finding fragments
/// * `json` - Whether to output JSON format
/// * `w` - Writer to output profile names to
///
/// # Returns
/// * `Ok(())` - All profiles listed successfully
/// * `Err(String)` - Operation failed
///
/// # Errors
/// Returns an error if writing to the output fails.
pub fn list_profiles(
    cfg: &Config,
    lib: &Path,
    json: bool,
    mut w: impl Write,
) -> Result<(), String> {
    if json {
        // Collect all fragments from library directory
        let mut fragments = Vec::new();
        if lib.exists() {
            collect_fragments(lib, lib, &mut fragments)?;
        }
        fragments.sort();

        // Build profile info
        let mut profiles: Vec<ProfileInfo> = cfg
            .profiles
            .iter()
            .map(|(name, deps)| ProfileInfo {
                name: name.clone(),
                dependencies: deps.clone(),
            })
            .collect();
        profiles.sort_by(|a, b| a.name.cmp(&b.name));

        let output = ListOutput {
            profiles,
            fragments,
        };
        let json_output = serde_json::to_string_pretty(&output)
            .map_err(|e| format!("JSON serialization error: {e}"))?;
        writeln!(&mut w, "{json_output}").map_err(|e| format!("Write error: {e}"))?;
    } else {
        let mut names: Vec<_> = cfg.profiles.keys().cloned().collect();
        names.sort();
        for n in names {
            writeln!(&mut w, "{n}").map_err(|e| format!("Write error: {e}"))?;
        }
    }
    Ok(())
}

/// Recursively collect all .md files from a directory
fn collect_fragments(root: &Path, dir: &Path, fragments: &mut Vec<String>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let path = entry.path();

        if path.is_dir() {
            collect_fragments(root, &path, fragments)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            if let Ok(rel_path) = path.strip_prefix(root) {
                fragments.push(rel_path.display().to_string());
            }
        }
    }

    Ok(())
}

/// Validate configuration and library file references.
///
/// Checks that all profile dependencies are valid, including:
/// - Referenced profiles exist in configuration
/// - Referenced markdown files exist in library
/// - No circular dependencies exist
///
/// # Arguments
/// * `cfg` - Configuration to validate
/// * `lib` - Library root directory for file validation
///
/// # Returns
/// * `Ok(())` - Configuration is valid
/// * `Err(String)` - Validation errors found
///
/// # Errors
/// Returns an error if:
/// - Referenced profiles don't exist
/// - Referenced files don't exist
/// - Circular dependencies are detected
pub fn validate(cfg: &Config, lib: &Path) -> Result<(), String> {
    let mut errors: Vec<String> = Vec::new();

    for (profile, deps) in &cfg.profiles {
        for dep in deps {
            if std::path::Path::new(dep)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
            {
                let path = lib.join(dep);
                if !path.exists() {
                    errors.push(format!(
                        "Missing file: {} (referenced by [{}])",
                        path.display(),
                        profile
                    ));
                }
            } else if !cfg.profiles.contains_key(dep) {
                errors.push(format!(
                    "Unknown profile: {dep} (referenced by [{profile}])"
                ));
            }
        }
    }

    for name in cfg.profiles.keys() {
        let mut seen_files = HashSet::new();
        let mut stack = Vec::new();
        let mut out = Vec::new();
        if let Err(ResolveError::Cycle(cycle)) =
            resolve_profile(name, cfg, lib, &mut seen_files, &mut stack, &mut out)
        {
            let chain = cycle.join(" -> ");
            errors.push(format!("Cycle detected: {chain}"));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// Build a tree node for a profile or fragment
fn build_tree_node(name: &str, cfg: &Config) -> TreeNode {
    // Check if it's a fragment (ends with .md)
    if std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    {
        return TreeNode {
            node_type: TreeNodeType::Fragment,
            name: name.to_string(),
            children: Vec::new(),
        };
    }

    // It's a profile - recursively build children
    let children = cfg
        .profiles
        .get(name)
        .map(|deps| deps.iter().map(|dep| build_tree_node(dep, cfg)).collect())
        .unwrap_or_default();

    TreeNode {
        node_type: TreeNodeType::Profile,
        name: name.to_string(),
        children,
    }
}

/// Find root profiles (profiles that are not referenced by any other profile)
fn find_root_profiles(cfg: &Config) -> Vec<String> {
    let mut referenced = HashSet::new();

    // Collect all profiles that are referenced by others
    for deps in cfg.profiles.values() {
        for dep in deps {
            // Only track profile references (not .md files)
            if !std::path::Path::new(dep)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
            {
                referenced.insert(dep.clone());
            }
        }
    }

    // Find profiles that are never referenced
    let mut roots: Vec<String> = cfg
        .profiles
        .keys()
        .filter(|profile| !referenced.contains(*profile))
        .cloned()
        .collect();

    roots.sort();
    roots
}

/// Build complete tree structure for all root profiles
fn build_trees(cfg: &Config) -> TreeOutput {
    let root_profiles = find_root_profiles(cfg);
    let trees = root_profiles
        .iter()
        .map(|profile| build_tree_node(profile, cfg))
        .collect();

    TreeOutput { trees }
}

/// Print tree structure in traditional tree format
fn print_tree(node: &TreeNode, prefix: &str, is_last: bool, w: &mut impl Write) -> io::Result<()> {
    // Print current node with appropriate connector
    let connector = if is_last { "└── " } else { "├── " };
    writeln!(w, "{prefix}{connector}{}", node.name)?;

    // Prepare prefix for children
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    // Print children
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        print_tree(child, &child_prefix, is_last_child, w)?;
    }

    Ok(())
}

/// Show tree structure for all profiles
pub fn show_tree(cfg: &Config, json: bool, mut w: impl Write) -> Result<(), String> {
    let trees = build_trees(cfg);

    if json {
        let json_output = serde_json::to_string_pretty(&trees)
            .map_err(|e| format!("JSON serialization error: {e}"))?;
        writeln!(&mut w, "{json_output}").map_err(|e| format!("Write error: {e}"))?;
    } else {
        for (i, tree) in trees.trees.iter().enumerate() {
            // Print root profile name
            writeln!(&mut w, "{}", tree.name).map_err(|e| format!("Write error: {e}"))?;

            // Print children with tree structure
            for (j, child) in tree.children.iter().enumerate() {
                let is_last = j == tree.children.len() - 1;
                print_tree(child, "", is_last, &mut w).map_err(|e| format!("Write error: {e}"))?;
            }

            // Add blank line between trees (except after last one)
            if i < trees.trees.len() - 1 {
                writeln!(&mut w).map_err(|e| format!("Write error: {e}"))?;
            }
        }
    }

    Ok(())
}

/// Show tree structure to stdout
pub fn run_tree_stdout(config_override: Option<&Path>, json: bool) -> Result<(), String> {
    let cfg_path = resolve_config_path(config_override)?;
    let cfg_text = read_config_with_path(&cfg_path)?;
    let cfg = parse_config_toml(&cfg_text)?;
    show_tree(&cfg, json, io::stdout())
}

/// Initialize default configuration and library structure.
///
/// Creates the default directory structure and configuration files
/// for prompter, including sample profiles and library files.
/// Only creates files that don't already exist (non-destructive).
///
/// # Returns
/// * `Ok(())` - Initialization completed successfully
/// * `Err(String)` - Initialization failed
///
/// # Errors
/// Returns an error if:
/// - Directory creation fails
/// - File writing fails
/// - HOME environment variable is not set
///
/// # Panics
/// Panics if the progress bar template is invalid (should not happen with the
/// hardcoded template string).
pub fn init_scaffold() -> Result<(), String> {
    let pb = if is_terminal() {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Initializing prompter...");
        pb.enable_steady_tick(std::time::Duration::from_millis(120));
        Some(pb)
    } else {
        None
    };

    let cfg_path = config_path()?;
    let cfg_dir = cfg_path
        .parent()
        .ok_or_else(|| "Invalid config path".to_string())?;

    if let Some(ref pb) = pb {
        pb.set_message("Creating config directory...");
    }
    fs::create_dir_all(cfg_dir)
        .map_err(|e| format!("Failed to create {}: {}", cfg_dir.display(), e))?;

    let lib = library_dir()?;
    if let Some(ref pb) = pb {
        pb.set_message("Creating library directory...");
    }
    fs::create_dir_all(&lib).map_err(|e| format!("Failed to create {}: {}", lib.display(), e))?;

    if !cfg_path.exists() {
        if let Some(ref pb) = pb {
            pb.set_message("Writing default config...");
        }
        let default_cfg = r#"# Prompter configuration
# Profiles map to sets of markdown files and/or other profiles.
# Files are relative to $HOME/.local/prompter/library

[python.api]
depends_on = ["a/b/c.md", "f/g/h.md"]

[general.testing]
depends_on = ["python.api", "a/b/d.md"]
"#;
        fs::write(&cfg_path, default_cfg)
            .map_err(|e| format!("Failed to write {}: {}", cfg_path.display(), e))?;
    }

    let paths_and_contents: Vec<(PathBuf, &str)> = vec![
        (
            lib.join("a/b/c.md"),
            "# a/b/c.md\nExample snippet for python.api.\n",
        ),
        (lib.join("a/b.md"), "# a/b.md\nFolder-level notes.\n"),
        (
            lib.join("a/b/d.md"),
            "# a/b/d.md\nGeneral testing snippet.\n",
        ),
        (lib.join("f/g/h.md"), "# f/g/h.md\nShared helper snippet.\n"),
    ];

    for (path, contents) in paths_and_contents {
        if let Some(ref pb) = pb {
            pb.set_message(format!(
                "Creating {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
        }
        if !path.exists() {
            fs::write(&path, contents)
                .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
        }
    }

    if let Some(pb) = pb {
        pb.finish_with_message("Initialization complete!");
        std::thread::sleep(std::time::Duration::from_millis(200)); // Brief pause to show completion
    }

    println!(
        "{}",
        success_message(&format!("Initialized config at {}", cfg_path.display()))
    );
    println!(
        "{}",
        info_message(&format!("Library root at {}", lib.display()))
    );
    Ok(())
}

/// List profiles to stdout.
///
/// Convenience function that reads configuration and lists all profiles
/// to standard output.
///
/// # Arguments
/// * `config_override` - Optional configuration file override
/// * `json` - Whether to output in JSON format
///
/// # Returns
/// * `Ok(())` - Profiles listed successfully
/// * `Err(String)` - Operation failed
///
/// # Errors
/// Returns an error if:
/// - Configuration file cannot be read or parsed
/// - Writing to stdout fails
pub fn run_list_stdout(config_override: Option<&Path>, json: bool) -> Result<(), String> {
    let cfg_path = resolve_config_path(config_override)?;
    let cfg_text = read_config_with_path(&cfg_path)?;
    let cfg = parse_config_toml(&cfg_text)?;
    let lib = library_path_for_config_override(config_override, &cfg_path)?;
    list_profiles(&cfg, &lib, json, io::stdout())
}

/// JSON output for successful validation
#[derive(Debug, Serialize)]
struct ValidateOutput {
    valid: bool,
}

/// Validate configuration and output results to stdout.
///
/// Convenience function that reads configuration and validates it,
/// outputting any errors found.
///
/// # Arguments
/// * `config_override` - Optional configuration file override
/// * `json` - Whether to output in JSON format
///
/// # Returns
/// * `Ok(())` - Configuration is valid
/// * `Err(String)` - Validation errors found
///
/// # Errors
/// Returns an error if:
/// - Configuration file cannot be read or parsed
/// - Validation finds missing files or circular dependencies
pub fn run_validate_stdout(config_override: Option<&Path>, json: bool) -> Result<(), String> {
    let cfg_path = resolve_config_path(config_override)?;
    let cfg_text = read_config_with_path(&cfg_path)?;
    let cfg = parse_config_toml(&cfg_text)?;
    let lib = library_path_for_config_override(config_override, &cfg_path)?;
    validate(&cfg, &lib)?;

    if json {
        let output = ValidateOutput { valid: true };
        let json_output = serde_json::to_string_pretty(&output)
            .map_err(|e| format!("JSON serialization error: {e}"))?;
        println!("{json_output}");
    }

    Ok(())
}

/// JSON structure for a single fragment
#[derive(Debug, Serialize)]
struct FragmentOutput {
    path: String,
    content: String,
}

/// JSON output structure for render command
#[derive(Debug, Serialize)]
struct RenderOutput {
    profile: String,
    pre_prompt: String,
    system_info: String,
    fragments: Vec<FragmentOutput>,
}

/// Render one or more profiles' content to a writer.
///
/// Resolves profile dependencies and writes the concatenated content
/// to the provided writer, including pre-prompt, system info, file
/// contents with optional separators, and post-prompt. When multiple
/// profiles are provided, files are deduplicated across all profiles
/// (first occurrence wins).
///
/// # Arguments
/// * `cfg` - Configuration containing profile definitions
/// * `lib` - Library root directory for file resolution
/// * `w` - Writer to output rendered content to
/// * `profiles` - Profile names to render (deduplicated in order)
/// * `separator` - Optional separator between files
/// * `pre_prompt` - Optional custom pre-prompt (defaults to LLM instructions)
/// * `post_prompt` - Optional custom post-prompt (defaults to @AGENTS/@CLAUDE instructions)
/// * `json` - Whether to output in JSON format
///
/// # Returns
/// * `Ok(())` - Profiles rendered successfully
/// * `Err(String)` - Rendering failed
///
/// # Errors
/// Returns an error if:
/// - Profile resolution fails (missing files, cycles, unknown profiles)
/// - Writing to output fails
/// - File reading fails
pub fn render_to_writer(
    cfg: &Config,
    lib: &Path,
    mut w: impl Write,
    profiles: &[String],
    separator: Option<&str>,
    pre_prompt: Option<&str>,
    post_prompt: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let mut seen_files = HashSet::new();
    let mut files = Vec::new();

    // Resolve all profiles with shared deduplication
    for profile in profiles {
        let mut stack = Vec::new();
        resolve_profile(profile, cfg, lib, &mut seen_files, &mut stack, &mut files).map_err(
            |e| match e {
                ResolveError::UnknownProfile(p) => format!("Unknown profile: {p}"),
                ResolveError::Cycle(c) => format!("Cycle detected: {}", c.join(" -> ")),
                ResolveError::MissingFile(path, prof) => format!(
                    "Missing file: {} (referenced by [{}])",
                    path.display(),
                    prof
                ),
            },
        )?;
    }

    if json {
        // JSON output mode
        let default_pre = default_pre_prompt();
        let pre_prompt_text = pre_prompt.unwrap_or(&default_pre).to_string();

        let date = Local::now().format("%Y-%m-%d").to_string();
        let os = env::consts::OS;
        let arch = env::consts::ARCH;
        let system_info = format!("Today is {date}, and you are running on a {arch}/{os} system.");

        let mut fragments = Vec::new();
        for path in &files {
            let content = fs::read_to_string(path)
                .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
            let rel_path = path.strip_prefix(lib).unwrap_or(path).display().to_string();
            fragments.push(FragmentOutput {
                path: rel_path,
                content,
            });
        }

        let output = RenderOutput {
            profile: profiles.join(", "),
            pre_prompt: pre_prompt_text,
            system_info,
            fragments,
        };

        let json_output = serde_json::to_string_pretty(&output)
            .map_err(|e| format!("JSON serialization error: {e}"))?;
        writeln!(&mut w, "{json_output}").map_err(|e| format!("Write error: {e}"))?;
    } else {
        // Text output mode
        // Write pre-prompt (defaults if not provided)
        let default_pre = default_pre_prompt();
        let pre_prompt_text = pre_prompt.unwrap_or(&default_pre);
        w.write_all(pre_prompt_text.as_bytes())
            .map_err(|e| format!("Write error: {e}"))?;

        // Write system prefix with two newlines before
        w.write_all(b"\n")
            .map_err(|e| format!("Write error: {e}"))?;
        let prefix = format_system_prefix();
        w.write_all(prefix.as_bytes())
            .map_err(|e| format!("Write error: {e}"))?;

        let sep = separator.unwrap_or("");
        for path in files {
            // Two newlines before each file
            w.write_all(b"\n")
                .map_err(|e| format!("Write error: {e}"))?;

            match fs::read(&path) {
                Ok(bytes) => w
                    .write_all(&bytes)
                    .map_err(|e| format!("Write error: {e}"))?,
                Err(e) => return Err(format!("Failed to read {}: {}", path.display(), e)),
            }

            // Write separator after each file if provided
            if !sep.is_empty() {
                w.write_all(sep.as_bytes())
                    .map_err(|e| format!("Write error: {e}"))?;
            }
        }

        // Write post-prompt (defaults if not provided)
        let default_post = default_post_prompt();
        let post_prompt_text = post_prompt
            .or(cfg.post_prompt.as_deref())
            .unwrap_or(&default_post);

        // Two newlines before post-prompt
        w.write_all(b"\n\n")
            .map_err(|e| format!("Write error: {e}"))?;
        w.write_all(post_prompt_text.as_bytes())
            .map_err(|e| format!("Write error: {e}"))?;
    }

    Ok(())
}

/// Render one or more profiles to stdout.
///
/// Convenience function that reads configuration and renders the specified
/// profiles to standard output with optional separator, pre-prompt, and post-prompt.
/// When multiple profiles are provided, files are deduplicated across all profiles.
///
/// # Arguments
/// * `profiles` - Profile names to render (deduplicated in order)
/// * `separator` - Optional separator between files
/// * `pre_prompt` - Optional custom pre-prompt text
/// * `post_prompt` - Optional custom post-prompt text
/// * `config_override` - Optional configuration file override
/// * `json` - Whether to output in JSON format
///
/// # Returns
/// * `Ok(())` - Profiles rendered successfully
/// * `Err(String)` - Rendering failed
///
/// # Errors
/// Returns an error if:
/// - Configuration file cannot be read or parsed
/// - Profile resolution fails
/// - Writing to stdout fails
pub fn run_render_stdout(
    profiles: &[String],
    separator: Option<&str>,
    pre_prompt: Option<&str>,
    post_prompt: Option<&str>,
    config_override: Option<&Path>,
    json: bool,
) -> Result<(), String> {
    let cfg_path = resolve_config_path(config_override)?;
    let cfg_text = read_config_with_path(&cfg_path)?;
    let cfg = parse_config_toml(&cfg_text)?;
    let lib = library_path_for_config_override(config_override, &cfg_path)?;
    let stdout = io::stdout();
    let handle = stdout.lock();
    render_to_writer(
        &cfg,
        &lib,
        handle,
        profiles,
        separator,
        pre_prompt,
        post_prompt,
        json,
    )
}

/// Render composed profiles to a byte vector.
///
/// Convenience wrapper around [`render_to_writer`] that handles config
/// resolution and returns the rendered output as bytes. Intended for
/// use by other crates that need prompt composition as a library.
///
/// # Arguments
/// * `profiles` - Profile names to compose
/// * `config_override` - Optional path to custom config file
///
/// # Returns
/// * `Ok(Vec<u8>)` - Rendered prompt content
/// * `Err(String)` - Error message
///
/// # Errors
/// Returns an error if config resolution, profile resolution, or rendering fails.
pub fn render_to_vec(
    profiles: &[String],
    config_override: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let cfg_path = resolve_config_path(config_override)?;
    let cfg_text = read_config_with_path(&cfg_path)?;
    let cfg = parse_config_toml(&cfg_text)?;
    let lib = library_path_for_config_override(config_override, &cfg_path)?;
    let mut buf = Vec::new();
    render_to_writer(&cfg, &lib, &mut buf, profiles, None, None, None, false)?;
    Ok(buf)
}

/// List available profile names.
///
/// Returns all profile names from the configuration as a sorted vector.
/// Intended for use by other crates that need to validate profile names.
///
/// # Arguments
/// * `config_override` - Optional path to custom config file
///
/// # Returns
/// * `Ok(Vec<String>)` - Sorted profile names
/// * `Err(String)` - Error message
///
/// # Errors
/// Returns an error if config resolution or parsing fails.
pub fn available_profiles(
    config_override: Option<&Path>,
) -> Result<Vec<String>, String> {
    let cfg_path = resolve_config_path(config_override)?;
    let cfg_text = read_config_with_path(&cfg_path)?;
    let cfg = parse_config_toml(&cfg_text)?;
    let mut names: Vec<String> = cfg.profiles.keys().cloned().collect();
    names.sort();
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_tmp(prefix: &str) -> PathBuf {
        let mut p = env::temp_dir();
        let unique = format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        p.push(unique);
        p
    }

    #[test]
    fn test_unescape() {
        assert_eq!(unescape("a\\nb\\t\\\"\\\\c"), "a\nb\t\"\\c");
        assert_eq!(unescape("line1\\rline2"), "line1\rline2");
        assert_eq!(unescape("noesc"), "noesc");
    }

    #[test]
    fn test_strip_comments_and_brackets_detection() {
        let s = r"ab#cd";
        assert_eq!(strip_comments(s), "ab");
        let s = r#""ab#cd" # trailing"#;
        assert_eq!(strip_comments(s), "\"ab#cd\" ");
        assert!(contains_closing_bracket_outside_quotes("[\"not]here\"]]"));
        assert!(!contains_closing_bracket_outside_quotes("[\"no]close\""));
    }

    #[test]
    fn test_parse_array_items_escape_and_error() {
        let s = r#"["a\"b", "c"]"#;
        let items = parse_array_items(s).unwrap();
        assert_eq!(items, vec!["a\"b", "c"]);
        let err = parse_array_items("[\"unterminated").unwrap_err();
        assert!(err.contains("Unterminated"));
    }

    #[test]
    fn test_parse_config_errors() {
        let err = parse_config_toml("[]\n").unwrap_err();
        assert!(err.contains("Empty section name"));
        let err = parse_config_toml("[p]\ndepends_on = \"x\"\n").unwrap_err();
        assert!(err.contains("must be an array"));
        let err = parse_config_toml("depends_on = [\"a.md\"]\n").unwrap_err();
        assert!(err.contains("outside of a profile section"));
    }

    #[test]
    fn test_validate_success_and_unknowns() {
        let cfg = Config {
            profiles: HashMap::from([
                ("p1".into(), vec!["a.md".into()]),
                ("p2".into(), vec!["p1".into(), "b.md".into()]),
            ]),
            post_prompt: None,
        };
        let lib = mk_tmp("prompter_validate_ok");
        fs::create_dir_all(&lib).unwrap();
        fs::write(lib.join("a.md"), b"A").unwrap();
        fs::write(lib.join("b.md"), b"B").unwrap();
        assert!(validate(&cfg, &lib).is_ok());
        let cfg2 = Config {
            profiles: HashMap::from([("root".into(), vec!["nope".into()])]),
            post_prompt: None,
        };
        let err = validate(&cfg2, &lib).unwrap_err();
        assert!(err.contains("Unknown profile"));
    }

    #[test]
    fn test_resolve_errors_and_dedup() {
        let cfg = Config {
            profiles: HashMap::from([("root".into(), vec!["missing.md".into()])]),
            post_prompt: None,
        };
        let lib = mk_tmp("prompter_resolve_errs");
        fs::create_dir_all(&lib).unwrap();
        let mut seen = HashSet::new();
        let mut stack = Vec::new();
        let mut out = Vec::new();
        let err = resolve_profile("root", &cfg, &lib, &mut seen, &mut stack, &mut out).unwrap_err();
        match err {
            ResolveError::MissingFile(_, p) => assert_eq!(p, "root"),
            _ => panic!("expected missing file"),
        }

        let cfg2 = Config {
            profiles: HashMap::from([
                ("A".into(), vec!["a/b.md".into()]),
                ("B".into(), vec!["A".into(), "a/b.md".into()]),
            ]),
            post_prompt: None,
        };
        fs::create_dir_all(lib.join("a")).unwrap();
        fs::write(lib.join("a/b.md"), b"X").unwrap();
        let mut seen = HashSet::new();
        let mut stack = Vec::new();
        let mut out = Vec::new();
        resolve_profile("B", &cfg2, &lib, &mut seen, &mut stack, &mut out).unwrap();
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn test_parse_args_errors() {
        // unknown flag
        let args = vec!["prompter".into(), "--bogus".into()];
        let err = parse_args_from(args).unwrap_err();
        assert!(err.contains("unexpected argument"));
        // missing required subcommand
        let args = vec!["prompter".into()];
        let err = parse_args_from(args).unwrap_err();
        assert!(err.contains("Usage:") || err.contains("COMMAND"));
    }

    #[test]
    fn test_list_profiles_order() {
        let cfg = Config {
            profiles: HashMap::from([("b".into(), vec![]), ("a".into(), vec![])]),
            post_prompt: None,
        };
        let lib = mk_tmp("prompter_list_order");
        fs::create_dir_all(&lib).unwrap();
        let mut out = Vec::new();
        super::list_profiles(&cfg, &lib, false, &mut out).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "a\nb\n");
    }

    #[test]
    fn test_validate_cycle_detected() {
        let cfg = Config {
            profiles: HashMap::from([
                ("A".into(), vec!["B".into()]),
                ("B".into(), vec!["A".into()]),
            ]),
            post_prompt: None,
        };
        let lib = mk_tmp("prompter_cycle");
        fs::create_dir_all(&lib).unwrap();
        let err = validate(&cfg, &lib).unwrap_err();
        assert!(err.contains("Cycle detected"));
    }

    #[test]
    fn test_parse_config_multiline_long() {
        let cfg = r#"
[profile.x]
depends_on = [
  "a/b.md",
  "c/d.md",
  "e/f.md",
]
"#;
        let parsed = parse_config_toml(cfg).unwrap();
        assert_eq!(parsed.profiles.get("profile.x").unwrap().len(), 3);
    }

    #[test]
    fn test_render_to_writer_basic() {
        // library and files
        let lib = mk_tmp("prompter_render_to_writer");
        fs::create_dir_all(lib.join("a")).unwrap();
        fs::create_dir_all(lib.join("f")).unwrap();
        fs::write(lib.join("a/x.md"), b"AX\n").unwrap();
        fs::write(lib.join("f/y.md"), b"FY\n").unwrap();
        // config with nested profile and duplicate file reference
        let cfg = Config {
            profiles: HashMap::from([
                ("child".into(), vec!["a/x.md".into()]),
                (
                    "root".into(),
                    vec!["child".into(), "f/y.md".into(), "a/x.md".into()],
                ),
            ]),
            post_prompt: None,
        };
        let mut out = Vec::new();
        super::render_to_writer(
            &cfg,
            &lib,
            &mut out,
            &["root".to_string()],
            Some("\n--\n"),
            None,
            None,
            false,
        )
        .unwrap();

        let output_str = String::from_utf8(out).unwrap();
        // Should start with default pre-prompt
        assert!(output_str.starts_with("You are an LLM coding agent."));
        // Should contain system prefix
        assert!(output_str.contains("Today is "));
        assert!(output_str.contains(", and you are running on a "));
        assert!(output_str.contains(" system.\n\n"));
        // Should contain the file contents with separator
        assert!(output_str.contains("AX\n"));
        assert!(output_str.contains("\n--\n"));
        assert!(output_str.contains("FY\n"));
        // Should end with default post-prompt
        assert!(output_str.ends_with(
            "Now, read the @AGENTS.md and @CLAUDE.md files in this directory, if they exist."
        ));
    }

    #[test]
    fn test_render_to_writer_custom_pre_prompt() {
        // library and files
        let lib = mk_tmp("prompter_render_custom_pre");
        fs::create_dir_all(lib.join("a")).unwrap();
        fs::write(lib.join("a/x.md"), b"Content\n").unwrap();
        // config
        let cfg = Config {
            profiles: HashMap::from([("test".into(), vec!["a/x.md".into()])]),
            post_prompt: None,
        };
        let mut out = Vec::new();
        super::render_to_writer(
            &cfg,
            &lib,
            &mut out,
            &["test".to_string()],
            None,
            Some("Custom pre-prompt\n\n"),
            None,
            false,
        )
        .unwrap();

        let output_str = String::from_utf8(out).unwrap();
        // Should start with custom pre-prompt
        assert!(output_str.starts_with("Custom pre-prompt\n\n"));
        // Should contain system prefix
        assert!(output_str.contains("Today is "));
        // Should contain file content
        assert!(output_str.contains("Content\n"));
        // Should end with default post-prompt
        assert!(output_str.ends_with(
            "Now, read the @AGENTS.md and @CLAUDE.md files in this directory, if they exist."
        ));
    }

    #[test]
    fn test_render_to_writer_custom_post_prompt() {
        // library and files
        let lib = mk_tmp("prompter_render_custom_post");
        fs::create_dir_all(lib.join("a")).unwrap();
        fs::write(lib.join("a/x.md"), b"Content\n").unwrap();
        // config with custom post_prompt
        let cfg = Config {
            profiles: HashMap::from([("test".into(), vec!["a/x.md".into()])]),
            post_prompt: Some("Custom config post-prompt".to_string()),
        };
        let mut out = Vec::new();
        super::render_to_writer(
            &cfg,
            &lib,
            &mut out,
            &["test".to_string()],
            None,
            None,
            None,
            false,
        )
        .unwrap();

        let output_str = String::from_utf8(out).unwrap();
        // Should end with config post-prompt
        assert!(output_str.ends_with("Custom config post-prompt"));

        // Test CLI post-prompt overriding config
        let mut out2 = Vec::new();
        super::render_to_writer(
            &cfg,
            &lib,
            &mut out2,
            &["test".to_string()],
            None,
            None,
            Some("CLI post-prompt"),
            false,
        )
        .unwrap();

        let output_str2 = String::from_utf8(out2).unwrap();
        // Should end with CLI post-prompt
        assert!(output_str2.ends_with("CLI post-prompt"));
    }

    #[test]
    fn test_render_multiple_profiles_with_deduplication() {
        // Create library with files that will be shared across profiles
        let lib = mk_tmp("prompter_multi_profile_dedup");
        fs::create_dir_all(lib.join("shared")).unwrap();
        fs::create_dir_all(lib.join("a")).unwrap();
        fs::create_dir_all(lib.join("b")).unwrap();

        fs::write(lib.join("shared/common.md"), b"COMMON\n").unwrap();
        fs::write(lib.join("a/specific.md"), b"A_SPECIFIC\n").unwrap();
        fs::write(lib.join("b/specific.md"), b"B_SPECIFIC\n").unwrap();

        // Create config where both profiles depend on the common file
        let cfg = Config {
            profiles: HashMap::from([
                (
                    "profile_a".into(),
                    vec!["shared/common.md".into(), "a/specific.md".into()],
                ),
                (
                    "profile_b".into(),
                    vec!["shared/common.md".into(), "b/specific.md".into()],
                ),
            ]),
            post_prompt: None,
        };

        // Render both profiles together
        let mut out = Vec::new();
        super::render_to_writer(
            &cfg,
            &lib,
            &mut out,
            &["profile_a".to_string(), "profile_b".to_string()],
            Some("\n---\n"),
            None,
            None,
            false,
        )
        .unwrap();

        let output_str = String::from_utf8(out).unwrap();

        // Verify the common file appears only once
        let common_count = output_str.matches("COMMON").count();
        assert_eq!(
            common_count, 1,
            "Common file should appear exactly once, found {common_count}"
        );

        // Verify both specific files appear
        assert!(output_str.contains("A_SPECIFIC"));
        assert!(output_str.contains("B_SPECIFIC"));

        // Verify order: common should appear first (from profile_a), then a_specific, then b_specific
        let common_pos = output_str.find("COMMON").unwrap();
        let a_pos = output_str.find("A_SPECIFIC").unwrap();
        let b_pos = output_str.find("B_SPECIFIC").unwrap();

        assert!(
            common_pos < a_pos,
            "Common file should come before A_SPECIFIC"
        );
        assert!(a_pos < b_pos, "A_SPECIFIC should come before B_SPECIFIC");
    }

    #[test]
    fn test_parse_config_with_post_prompt() {
        let cfg = r#"
post_prompt = "Custom post prompt from config"

[profile]
depends_on = ["file.md"]
"#;
        let parsed = parse_config_toml(cfg).unwrap();
        assert_eq!(
            parsed.post_prompt,
            Some("Custom post prompt from config".to_string())
        );
        assert_eq!(parsed.profiles.get("profile").unwrap().len(), 1);
    }

    #[test]
    fn test_array_items_escaped_backslash() {
        let s = r#"["a\\"]"#; // a single backslash in content
        let items = parse_array_items(s).unwrap();
        assert_eq!(items, vec!["a\\"]);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_args_from() {
        let args = vec![
            "prompter".into(),
            "run".into(),
            "--separator".into(),
            "\\n--\\n".into(),
            "profile".into(),
        ];
        match parse_args_from(args).unwrap() {
            AppMode::Run {
                profiles,
                separator,
                pre_prompt,
                post_prompt,
                config,
                json,
            } => {
                assert_eq!(profiles, vec!["profile".to_string()]);
                assert_eq!(separator, Some("\n--\n".into()));
                assert_eq!(pre_prompt, None);
                assert_eq!(post_prompt, None);
                assert!(config.is_none());
                assert!(!json);
            }
            _ => panic!("expected run"),
        }

        let args = vec![
            "prompter".into(),
            "run".into(),
            "--pre-prompt".into(),
            "Custom pre-prompt".into(),
            "profile".into(),
        ];
        match parse_args_from(args).unwrap() {
            AppMode::Run {
                profiles,
                separator,
                pre_prompt,
                post_prompt,
                config,
                json,
            } => {
                assert_eq!(profiles, vec!["profile".to_string()]);
                assert_eq!(separator, None);
                assert_eq!(pre_prompt, Some("Custom pre-prompt".into()));
                assert_eq!(post_prompt, None);
                assert!(config.is_none());
                assert!(!json);
            }
            _ => panic!("expected run"),
        }

        // Test multiple profiles with run subcommand
        let args = vec![
            "prompter".into(),
            "run".into(),
            "profile1".into(),
            "profile2".into(),
            "profile3.nested".into(),
        ];
        match parse_args_from(args).unwrap() {
            AppMode::Run {
                profiles,
                separator,
                pre_prompt,
                post_prompt,
                config,
                json,
            } => {
                assert_eq!(
                    profiles,
                    vec![
                        "profile1".to_string(),
                        "profile2".to_string(),
                        "profile3.nested".to_string()
                    ]
                );
                assert_eq!(separator, None);
                assert_eq!(pre_prompt, None);
                assert_eq!(post_prompt, None);
                assert!(config.is_none());
                assert!(!json);
            }
            _ => panic!("expected run"),
        }

        let args = vec!["prompter".into(), "list".into()];
        assert!(matches!(
            parse_args_from(args).unwrap(),
            AppMode::List {
                config: None,
                json: false
            }
        ));
        let args = vec!["prompter".into(), "validate".into()];
        assert!(matches!(
            parse_args_from(args).unwrap(),
            AppMode::Validate {
                config: None,
                json: false
            }
        ));
        let args = vec!["prompter".into(), "init".into()];
        assert!(matches!(parse_args_from(args).unwrap(), AppMode::Init));
        let args = vec!["prompter".into(), "version".into()];
        assert!(matches!(
            parse_args_from(args).unwrap(),
            AppMode::Version { json: false }
        ));

        let args = vec![
            "prompter".into(),
            "--config".into(),
            "custom/config.toml".into(),
            "list".into(),
        ];
        match parse_args_from(args).unwrap() {
            AppMode::List { config, json } => {
                assert_eq!(config, Some(PathBuf::from("custom/config.toml")));
                assert!(!json);
            }
            other => panic!("unexpected mode: {other:?}"),
        }

        let args = vec![
            "prompter".into(),
            "run".into(),
            "--config".into(),
            "custom/config.toml".into(),
            "profile".into(),
        ];
        match parse_args_from(args).unwrap() {
            AppMode::Run { config, json, .. } => {
                assert_eq!(config, Some(PathBuf::from("custom/config.toml")));
                assert!(!json);
            }
            other => panic!("unexpected mode: {other:?}"),
        }
    }

    struct FailAfterN {
        writes_done: usize,
        fail_on: usize,
    }

    impl Write for FailAfterN {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.writes_done += 1;
            if self.writes_done == self.fail_on {
                Err(io::Error::other("synthetic write failure"))
            } else {
                Ok(buf.len())
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_render_to_writer_write_error_on_separator() {
        let lib = mk_tmp("prompter_write_err_sep");
        fs::create_dir_all(lib.join("a")).unwrap();
        fs::write(lib.join("a/x.md"), b"AX").unwrap();
        fs::write(lib.join("a/y.md"), b"AY").unwrap();
        let cfg = Config {
            profiles: HashMap::from([("p".into(), vec!["a/x.md".into(), "a/y.md".into()])]),
            post_prompt: None,
        };
        let mut w = FailAfterN {
            writes_done: 0,
            fail_on: 3,
        }; // pre-prompt ok, system prefix ok, fail on separator
        let err = super::render_to_writer(
            &cfg,
            &lib,
            &mut w,
            &["p".to_string()],
            Some("--"),
            None,
            None,
            false,
        )
        .unwrap_err();
        assert!(err.contains("Write error"), "err={err}");
    }

    #[test]
    fn test_render_to_writer_write_error_on_file() {
        let lib = mk_tmp("prompter_write_err_file");
        fs::create_dir_all(lib.join("a")).unwrap();
        fs::write(lib.join("a/x.md"), b"AX").unwrap();
        let cfg = Config {
            profiles: HashMap::from([("p".into(), vec!["a/x.md".into()])]),
            post_prompt: None,
        };
        let mut w = FailAfterN {
            writes_done: 0,
            fail_on: 1,
        }; // fail on first write (pre-prompt)
        let err = super::render_to_writer(
            &cfg,
            &lib,
            &mut w,
            &["p".to_string()],
            Some("--"),
            None,
            None,
            false,
        )
        .unwrap_err();
        assert!(err.contains("Write error"), "err={err}");
    }

    #[test]
    #[ignore = "Fails on CI due to HOME environment variable concurrency issues"]
    #[allow(unsafe_code)]
    fn test_run_list_and_validate_with_home_injection() {
        let home = mk_tmp("prompter_home_unit_ok");
        let cfg_dir = home.join(".config/prompter");
        let lib_dir = home.join(".local/prompter/library");
        fs::create_dir_all(&cfg_dir).unwrap();
        fs::create_dir_all(lib_dir.join("a")).unwrap();
        fs::create_dir_all(lib_dir.join("f")).unwrap();
        fs::write(lib_dir.join("a/x.md"), b"AX\n").unwrap();
        fs::write(lib_dir.join("f/y.md"), b"FY\n").unwrap();
        let cfg = r#"
[child]
depends_on = ["a/x.md"]

[root]
depends_on = ["child", "f/y.md"]
"#;
        fs::write(cfg_dir.join("config.toml"), cfg).unwrap();
        let prev_home = env::var("HOME").ok();
        unsafe {
            env::set_var("HOME", &home);
        }
        assert!(super::run_validate_stdout(None, false).is_ok());
        assert!(super::run_list_stdout(None, false).is_ok());
        if let Some(prev) = prev_home {
            unsafe {
                env::set_var("HOME", prev);
            }
        } else {
            unsafe {
                env::remove_var("HOME");
            }
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn test_run_validate_with_home_injection_failure() {
        let home = mk_tmp("prompter_home_unit_bad");
        let cfg_dir = home.join(".config/prompter");
        let lib_dir = home.join(".local/prompter/library");
        fs::create_dir_all(&cfg_dir).unwrap();
        fs::create_dir_all(&lib_dir).unwrap();
        let cfg = r#"
[root]
depends_on = ["missing.md", "unknown_profile"]
"#;
        fs::write(cfg_dir.join("config.toml"), cfg).unwrap();
        let prev_home = env::var("HOME").ok();
        unsafe {
            env::set_var("HOME", &home);
        }
        let err = super::run_validate_stdout(None, false).unwrap_err();
        assert!(
            err.contains("Missing file") && err.contains("Unknown profile"),
            "err={err}"
        );
        if let Some(prev) = prev_home {
            unsafe {
                env::set_var("HOME", prev);
            }
        } else {
            unsafe {
                env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn render_to_vec_returns_bytes() {
        let base = mk_tmp("prompter_render_to_vec");
        let lib = base.join("library");
        fs::create_dir_all(lib.join("a")).unwrap();
        fs::write(lib.join("a/hello.md"), b"HELLO_CONTENT\n").unwrap();
        let cfg_toml = r#"
[myprofile]
depends_on = ["a/hello.md"]
"#;
        let cfg_path = base.join("config.toml");
        fs::write(&cfg_path, cfg_toml).unwrap();

        let result =
            render_to_vec(&["myprofile".to_string()], Some(&cfg_path)).unwrap();
        let text = String::from_utf8(result).unwrap();
        assert!(
            text.contains("HELLO_CONTENT"),
            "render_to_vec output should contain the library file content"
        );
    }

    #[test]
    fn available_profiles_returns_sorted() {
        let base = mk_tmp("prompter_avail_profiles");
        let lib = base.join("library");
        fs::create_dir_all(lib.join("a")).unwrap();
        fs::write(lib.join("a/x.md"), b"X\n").unwrap();
        let cfg_toml = r#"
[zebra]
depends_on = ["a/x.md"]

[alpha]
depends_on = ["a/x.md"]

[middle]
depends_on = ["a/x.md"]
"#;
        let cfg_path = base.join("config.toml");
        fs::write(&cfg_path, cfg_toml).unwrap();

        let profiles = available_profiles(Some(&cfg_path)).unwrap();
        assert_eq!(profiles, vec!["alpha", "middle", "zebra"]);
    }
}
