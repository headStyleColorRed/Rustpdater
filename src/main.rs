mod daemon;

use clap::Parser;
use std::error::Error;
use daemon::config::Config;
use daemon::watcher;

/// Simple Git repo auto-updater.
#[derive(Parser)]
struct Cli {
    /// Path to config TOML
    #[arg(short, long, default_value = "etc/watcher.toml")]
    config_file: String,
}

#[tokio::main]
async fn main() {
    // Parse CLI arguments (We only need the config file path)
    let args = Cli::parse();

    // Load the config file and run the daemon
    if let Err(e) = run(args).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run(args: Cli) -> Result<(), Box<dyn Error>> {
    // Load the config file
    let config = Config::load_config(&args.config_file)?;

    // Start the daemon
    watcher::start_watching_repos(&config.repos).await?;

    Ok(())
}
