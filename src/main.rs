mod daemon;

use clap::Parser;
use std::error::Error;
use daemon::config::Config;
use daemon::watcher;

/// Simple Git repo auto-updater.
#[derive(Parser)]
struct Cli {
    /// Path to config TOML
    #[arg(short, long, default_value = "watcher.toml")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments (We only need the config file path)
    let args = Cli::parse();

    // Load the config file
    let config: Config = Config::load_config(&args.config_file).unwrap();

    // Start the daemon
    watcher::start_watching_repos(&config.repos).await?;

    Ok(())
}
