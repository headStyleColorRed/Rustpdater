use std::path::PathBuf;
use serde::Deserialize;


fn default_branch() -> String { "master".to_string() }
fn default_interval() -> u64 { 300 /*5 minutes in seconds */ }
fn default_remote() -> String { "origin".to_string() }

#[derive(Debug, Deserialize, Clone)]
pub struct RepoCfg {
    /// Local checkout path
    pub path: PathBuf,
    /// Branch to watch (default main)
    #[serde(default = "default_branch")]
    pub branch: String,
    /// Poll interval in seconds
    #[serde(default = "default_interval")]
    pub interval: u64,
    /// Remote name to fetch from (default origin)
    #[serde(default = "default_remote")]
    pub remote: String,
    /// Command to run after update (optional)
    pub on_change: Option<String>,
}


