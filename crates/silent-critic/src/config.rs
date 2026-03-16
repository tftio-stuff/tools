use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    pub db_dir: Option<String>,
    pub project_name: Option<String>,
}

pub fn resolve_config_path() -> Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(xdg).join("silent-critic/config.toml"))
    } else {
        let home = directories::BaseDirs::new()
            .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
        Ok(home.config_dir().join("silent-critic/config.toml"))
    }
}

pub fn resolve_db_path(config: &Config, project_hash: &str) -> Result<PathBuf> {
    if let Some(dir) = &config.db_dir {
        return Ok(PathBuf::from(dir).join(project_hash).join("db.sqlite"));
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        Ok(PathBuf::from(xdg)
            .join("silent-critic")
            .join(project_hash)
            .join("db.sqlite"))
    } else {
        let home = directories::BaseDirs::new()
            .ok_or_else(|| anyhow::anyhow!("cannot determine home directory"))?;
        Ok(home
            .data_local_dir()
            .join("silent-critic")
            .join(project_hash)
            .join("db.sqlite"))
    }
}

pub fn load_config() -> Result<Config> {
    let path = resolve_config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = std::fs::read_to_string(&path)?;
    Ok(toml::from_str(&contents)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert!(config.db_dir.is_none());
        assert!(config.project_name.is_none());
    }

    #[test]
    fn db_path_with_override() {
        let config = Config {
            db_dir: Some("/tmp/sc-data".to_string()),
            ..Config::default()
        };
        let path = resolve_db_path(&config, "abc123").unwrap();
        assert_eq!(path, PathBuf::from("/tmp/sc-data/abc123/db.sqlite"));
    }
}
