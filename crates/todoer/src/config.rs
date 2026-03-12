use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    pub db_path: Option<String>,
}

pub fn resolve_config_path() -> anyhow::Result<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Ok(PathBuf::from(xdg).join("todoer/config.toml"))
    } else {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
        Ok(home.join(".config/todoer/config.toml"))
    }
}

pub fn resolve_db_path(config: &Config) -> anyhow::Result<PathBuf> {
    if let Some(path) = &config.db_path {
        return Ok(PathBuf::from(path));
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        Ok(PathBuf::from(xdg).join("todoer/todoer.db"))
    } else {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
        Ok(home.join(".local/share/todoer/todoer.db"))
    }
}

pub fn load_config() -> anyhow::Result<Config> {
    let path = resolve_config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let contents = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&contents)?)
}
