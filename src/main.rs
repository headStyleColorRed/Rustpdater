mod daemon;

use clap::Parser;
use std::error::Error;
use daemon::config::Config;
use daemon::watcher;
use log::{error, info};

/// Simple Git repo auto-updater.
#[derive(Parser)]
struct Cli {
    /// Path to config TOML
    #[arg(short, long, default_value = "/etc/watcher.toml")]
    config_file: String,
}

#[tokio::main]
async fn main() {
    // Initialize env_logger if not already set
    if std::env::var_os("RUST_LOG").is_none() {
        // Default to info if the user didn't set a level
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Parse CLI arguments (We only need the config file path)
    let args = Cli::parse();

    // Load the config file and run the daemon
    if let Err(e) = run(args).await {
        error!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run(args: Cli) -> Result<(), Box<dyn Error>> {
    // Load the config file
    let config = Config::load_config(&args.config_file)?;

    info!("Loaded config from {} ({} repos)", args.config_file, config.repos.len());

    // Start the daemon
    watcher::start_watching_repos(&config.repos).await?;

    Ok(())
}
