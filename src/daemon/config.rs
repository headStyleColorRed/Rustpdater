use super::repo_config::RepoCfg;
use super::errors::{Result, WatchError};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub repos: Vec<RepoCfg>
}

impl Config {
    pub fn load_config(path: &str) -> Result<Config> {
        let file_text = std::fs::read_to_string(path).map_err(|e| WatchError::Config {
            path: path.to_string(),
            source: e,
        })?;
        let config: Config = toml::from_str(&file_text)?;
        Ok(config)
    }
}
