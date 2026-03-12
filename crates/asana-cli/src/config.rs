//! Configuration management utilities for the Asana CLI.
//!
//! Phase 1 establishes the persistent configuration surface and token storage.
//! Subsequent phases will expand the persisted settings and runtime validation.

use crate::error::Result;
use anyhow::{Context, anyhow};
use directories::ProjectDirs;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const ENV_TOKEN: &str = "ASANA_PAT";
const ENV_BASE_URL: &str = "ASANA_BASE_URL";
const ENV_WORKSPACE: &str = "ASANA_WORKSPACE";
const ENV_ASSIGNEE: &str = "ASANA_ASSIGNEE";
const ENV_PROJECT: &str = "ASANA_PROJECT";
const ENV_CONFIG_HOME: &str = "ASANA_CLI_CONFIG_HOME";
const ENV_DATA_HOME: &str = "ASANA_CLI_DATA_HOME";
/// Default Asana API base URL when no override is provided.
pub const DEFAULT_API_BASE_URL: &str = "https://app.asana.com/api/1.0";

/// Persisted configuration document.
#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(default)]
pub struct FileConfig {
    /// Optional custom API base URL for private deployments.
    pub api_base_url: Option<String>,
    /// Preferred default workspace identifier.
    pub default_workspace: Option<String>,
    /// Preferred default assignee identifier (email or gid).
    pub default_assignee: Option<String>,
    /// Preferred default project identifier.
    pub default_project: Option<String>,
    /// Stored Personal Access Token (if persisted on disk).
    pub personal_access_token: Option<String>,
}

impl fmt::Debug for FileConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileConfig")
            .field("api_base_url", &self.api_base_url)
            .field("default_workspace", &self.default_workspace)
            .field("default_assignee", &self.default_assignee)
            .field("default_project", &self.default_project)
            .field(
                "personal_access_token",
                &self.personal_access_token.as_ref().map(|_| "REDACTED"),
            )
            .finish()
    }
}

/// Runtime configuration including environment overrides and persisted settings.
pub struct Config {
    file: FileConfig,
    overrides: Overrides,
    paths: ConfigPaths,
}

impl Config {
    /// Load configuration from disk and environment.
    ///
    /// # Errors
    /// Returns an error if configuration directories cannot be created or files cannot be read.
    pub fn load() -> Result<Self> {
        let paths = resolve_paths()?;
        if let Some(parent) = paths.config_file.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create config directory at {}", parent.display())
            })?;
        }
        fs::create_dir_all(&paths.cache_dir).with_context(|| {
            format!(
                "failed to create cache directory at {}",
                paths.cache_dir.display()
            )
        })?;

        let file = read_config_file(&paths.config_file)?;
        let overrides = Overrides::collect();

        Ok(Self {
            file,
            overrides,
            paths,
        })
    }

    /// Return the path to the configuration file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.paths.config_file
    }

    /// Directory used for API response caching.
    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.paths.cache_dir
    }

    /// Directory used for persistent data assets (templates, filters, etc.).
    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.paths.data_dir
    }

    /// Directory where user templates are stored.
    #[must_use]
    pub fn templates_dir(&self) -> PathBuf {
        self.paths.data_dir.join("templates")
    }

    /// Directory storing reusable filter definitions.
    #[must_use]
    pub fn filters_dir(&self) -> PathBuf {
        self.paths.data_dir.join("filters")
    }

    /// Return the computed API base URL, considering environment overrides.
    #[must_use]
    pub fn api_base_url(&self) -> Option<&str> {
        self.overrides
            .api_base_url
            .as_deref()
            .or(self.file.api_base_url.as_deref())
    }

    /// Return the effective API base URL, falling back to the default value.
    #[must_use]
    pub fn effective_api_base_url(&self) -> &str {
        self.api_base_url().unwrap_or(DEFAULT_API_BASE_URL)
    }

    /// Return the default workspace identifier.
    #[must_use]
    pub fn default_workspace(&self) -> Option<&str> {
        self.overrides
            .default_workspace
            .as_deref()
            .or(self.file.default_workspace.as_deref())
    }

    /// Update the stored default workspace identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be saved to disk.
    pub fn set_default_workspace(&mut self, workspace: Option<String>) -> Result<()> {
        self.file.default_workspace = workspace;
        self.save()
    }

    /// Return the default assignee identifier.
    #[must_use]
    pub fn default_assignee(&self) -> Option<&str> {
        self.overrides
            .default_assignee
            .as_deref()
            .or(self.file.default_assignee.as_deref())
    }

    /// Update the stored default assignee identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be saved to disk.
    pub fn set_default_assignee(&mut self, assignee: Option<String>) -> Result<()> {
        self.file.default_assignee = assignee;
        self.save()
    }

    /// Return the default project identifier.
    #[must_use]
    pub fn default_project(&self) -> Option<&str> {
        self.overrides
            .default_project
            .as_deref()
            .or(self.file.default_project.as_deref())
    }

    /// Update the stored default project identifier.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file cannot be saved to disk.
    pub fn set_default_project(&mut self, project: Option<String>) -> Result<()> {
        self.file.default_project = project;
        self.save()
    }

    /// Persist the in-memory configuration to disk.
    ///
    /// # Errors
    /// Returns an error when the configuration cannot be encoded or written to disk.
    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.paths.config_file.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create configuration directory {}",
                    parent.display()
                )
            })?;
            secure_directory(parent)?;
        }

        let serialized =
            toml::to_string_pretty(&self.file).context("failed to encode configuration to TOML")?;
        fs::write(&self.paths.config_file, serialized).with_context(|| {
            format!(
                "failed to write configuration to {}",
                self.paths.config_file.display()
            )
        })?;
        secure_file(&self.paths.config_file)?;
        Ok(())
    }

    /// Store the provided Personal Access Token in the configuration file.
    ///
    /// # Errors
    /// Returns an error if the configuration file cannot be updated.
    pub fn store_personal_access_token(&mut self, token: &SecretString) -> Result<()> {
        self.file
            .personal_access_token
            .replace(token.expose_secret().to_owned());
        self.save()
    }

    /// Retrieve the Personal Access Token, taking environment overrides into account.
    #[must_use]
    pub fn personal_access_token(&self) -> Option<SecretString> {
        if let Some(token) = self.overrides.personal_access_token.clone() {
            return Some(token);
        }
        self.file.personal_access_token.as_ref().and_then(|value| {
            if value.trim().is_empty() {
                None
            } else {
                Some(SecretString::new(value.clone().into()))
            }
        })
    }

    /// Remove any stored Personal Access Token.
    ///
    /// # Errors
    /// Returns an error when stored secrets cannot be removed.
    pub fn delete_personal_access_token(&mut self) -> Result<()> {
        self.file.personal_access_token = None;
        self.save()
    }

    /// Determine whether a token is persisted in the configuration file.
    #[must_use]
    pub fn has_persisted_token(&self) -> bool {
        self.file
            .personal_access_token
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
    }

    /// Determine whether a token is provided by environment overrides.
    #[must_use]
    pub fn environment_token_available(&self) -> bool {
        self.overrides
            .personal_access_token
            .as_ref()
            .is_some_and(|value| !value.expose_secret().trim().is_empty())
    }

    /// Expose the underlying file configuration for mutation.
    pub fn file_config_mut(&mut self) -> &mut FileConfig {
        &mut self.file
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("file", &self.file)
            .field("overrides", &self.overrides)
            .field("paths", &self.paths)
            .finish()
    }
}

fn secure_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let metadata = fs::metadata(path)
            .with_context(|| format!("failed to inspect permissions for {}", path.display()))?;
        if metadata.is_dir() {
            let perms = metadata.permissions();
            let mode = perms.mode();
            if mode & 0o077 != 0 {
                let mut tightened = perms;
                tightened.set_mode(0o700);
                fs::set_permissions(path, tightened).with_context(|| {
                    format!(
                        "failed to tighten directory permissions for {}",
                        path.display()
                    )
                })?;
            }
        }
    }
    Ok(())
}

fn secure_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let metadata = fs::metadata(path)
            .with_context(|| format!("failed to inspect permissions for {}", path.display()))?;
        if metadata.is_file() {
            let perms = metadata.permissions();
            let mode = perms.mode();
            if mode & 0o177 != 0o100 {
                let mut tightened = perms;
                tightened.set_mode(0o600);
                fs::set_permissions(path, tightened).with_context(|| {
                    format!("failed to tighten file permissions for {}", path.display())
                })?;
            }
        }
    }
    Ok(())
}

#[derive(Clone, Default)]
struct Overrides {
    api_base_url: Option<String>,
    default_workspace: Option<String>,
    default_assignee: Option<String>,
    default_project: Option<String>,
    personal_access_token: Option<SecretString>,
}

impl fmt::Debug for Overrides {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Overrides")
            .field("api_base_url", &self.api_base_url)
            .field("default_workspace", &self.default_workspace)
            .field("default_assignee", &self.default_assignee)
            .field("default_project", &self.default_project)
            .field(
                "personal_access_token",
                &self.personal_access_token.as_ref().map(|_| "REDACTED"),
            )
            .finish()
    }
}

impl Overrides {
    fn collect() -> Self {
        Self {
            api_base_url: env::var(ENV_BASE_URL).ok(),
            default_workspace: env::var(ENV_WORKSPACE).ok(),
            default_assignee: env::var(ENV_ASSIGNEE).ok(),
            default_project: env::var(ENV_PROJECT).ok(),
            personal_access_token: env::var(ENV_TOKEN)
                .ok()
                .map(|s| SecretString::new(s.into())),
        }
    }
}

#[derive(Clone)]
struct ConfigPaths {
    config_file: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
}

impl fmt::Debug for ConfigPaths {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigPaths")
            .field("config_file", &self.config_file)
            .field("data_dir", &self.data_dir)
            .field("cache_dir", &self.cache_dir)
            .finish()
    }
}

fn resolve_paths() -> Result<ConfigPaths> {
    let project_dirs = if let Some(path) = env::var_os(ENV_CONFIG_HOME) {
        let base = PathBuf::from(path);
        ProjectDirs::from_path(base.clone()).ok_or_else(|| {
            anyhow!(
                "failed to construct project directories from {} (check {ENV_CONFIG_HOME})",
                base.display()
            )
        })?
    } else {
        ProjectDirs::from("com", "asana", "asana-cli")
            .ok_or_else(|| anyhow!("unable to resolve standard project directories"))?
    };

    let config_dir = env::var_os(ENV_CONFIG_HOME)
        .map_or_else(|| project_dirs.config_dir().to_path_buf(), PathBuf::from);

    let data_dir = env::var_os(ENV_DATA_HOME).map_or_else(
        || project_dirs.data_local_dir().to_path_buf(),
        PathBuf::from,
    );
    let cache_dir = data_dir.join("cache");

    Ok(ConfigPaths {
        config_file: config_dir.join("config.toml"),
        data_dir,
        cache_dir,
    })
}

fn read_config_file(path: &Path) -> Result<FileConfig> {
    if !path.exists() {
        debug!(config_path = %path.display(), "configuration file not found; using defaults");
        return Ok(FileConfig::default());
    }

    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read configuration file {}", path.display()))?;
    let parsed: FileConfig = toml::from_str(&contents)
        .with_context(|| format!("failed to parse configuration file {}", path.display()))?;
    Ok(parsed)
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::ffi::OsStr;
    use tempfile::TempDir;

    fn set_env<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) {
        unsafe {
            env::set_var(key, value);
        }
    }

    fn remove_env<K: AsRef<OsStr>>(key: K) {
        unsafe {
            env::remove_var(key);
        }
    }

    fn with_temp_env<F>(config_home: &TempDir, data_home: &TempDir, f: F)
    where
        F: FnOnce(),
    {
        set_env(ENV_CONFIG_HOME, config_home.path());
        set_env(ENV_DATA_HOME, data_home.path());
        f();
        remove_env(ENV_CONFIG_HOME);
        remove_env(ENV_DATA_HOME);
        remove_env(ENV_TOKEN);
        remove_env(ENV_BASE_URL);
        remove_env(ENV_WORKSPACE);
        remove_env(ENV_ASSIGNEE);
        remove_env(ENV_PROJECT);
    }

    #[test]
    #[serial]
    fn load_creates_directories_and_defaults() {
        let config_home = TempDir::new().unwrap();
        let data_home = TempDir::new().unwrap();

        with_temp_env(&config_home, &data_home, || {
            let cfg = Config::load().expect("load config");
            let parent = cfg.path().parent().expect("config path has parent");
            assert!(parent.exists(), "config directory should exist");
            assert!(
                !cfg.path().exists(),
                "config file should not be created automatically"
            );
            assert!(cfg.personal_access_token().is_none());
            assert!(cfg.api_base_url().is_none());
        });
    }

    #[test]
    #[serial]
    fn environment_overrides_take_precedence() {
        let config_home = TempDir::new().unwrap();
        let data_home = TempDir::new().unwrap();

        with_temp_env(&config_home, &data_home, || {
            set_env(ENV_BASE_URL, "https://override.example.com");
            set_env(ENV_WORKSPACE, "workspace-123");
            set_env(ENV_ASSIGNEE, "owner@example.com");
            set_env(ENV_TOKEN, "env-token");

            let mut cfg = Config::load().expect("load config");
            cfg.file_config_mut().api_base_url = Some("https://file.example.com".into());
            cfg.file_config_mut().default_workspace = Some("workspace-456".into());
            cfg.file_config_mut().default_assignee = Some("file@example.com".into());
            cfg.file_config_mut().personal_access_token = Some("file-token".into());
            cfg.save().expect("save config");

            let cfg = Config::load().expect("reload config");
            assert_eq!(cfg.api_base_url(), Some("https://override.example.com"));
            assert_eq!(cfg.default_workspace(), Some("workspace-123"));
            assert_eq!(cfg.default_assignee(), Some("owner@example.com"));
            let token = cfg.personal_access_token().expect("token present");
            assert_eq!(token.expose_secret(), "env-token");
        });
    }

    #[test]
    #[serial]
    fn token_round_trip_in_config_file() {
        let config_home = TempDir::new().unwrap();
        let data_home = TempDir::new().unwrap();

        with_temp_env(&config_home, &data_home, || {
            let mut cfg = Config::load().expect("load config");
            let token = SecretString::new("token-value".into());
            cfg.store_personal_access_token(&token)
                .expect("store token");

            let reloaded = Config::load().expect("reload config");
            let loaded = reloaded.personal_access_token();
            assert_eq!(
                loaded.as_ref().map(|s| s.expose_secret().to_string()),
                Some("token-value".into())
            );
        });
    }

    #[test]
    #[serial]
    fn default_assignee_round_trip() {
        let config_home = TempDir::new().unwrap();
        let data_home = TempDir::new().unwrap();

        with_temp_env(&config_home, &data_home, || {
            let mut cfg = Config::load().expect("load config");
            cfg.set_default_assignee(Some("user@example.com".into()))
                .expect("store assignee");

            let cfg = Config::load().expect("reload config");
            assert_eq!(cfg.default_assignee(), Some("user@example.com"));

            let mut cfg = Config::load().expect("load config to clear");
            cfg.set_default_assignee(None).expect("clear assignee");
            let cfg = Config::load().expect("reload after clear");
            assert!(cfg.default_assignee().is_none());
        });
    }
}
