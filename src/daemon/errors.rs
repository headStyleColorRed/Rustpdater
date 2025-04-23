use thiserror::Error;
use tokio::task::JoinError;

pub type Result<T> = std::result::Result<T, WatchError>;

#[derive(Error, Debug)]
pub enum WatchError {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("config error: could not load config file '{path}' - {source}")]
    Config { path: String, source: std::io::Error },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("task join error: {0}")]
    Join(#[from] JoinError),
}
