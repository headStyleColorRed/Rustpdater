use super::repo_config::RepoCfg;
use super::errors::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub repos: Vec<RepoCfg>
}

impl Config {
    pub fn load_config(path: &str) -> Result<Config> {
        let file_text: String = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&file_text)?;
        Ok(config)
    }
}
