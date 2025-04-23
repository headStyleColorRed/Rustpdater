mod daemon;

use clap::Parser;
use std::error::Error;
use daemon::config::Config;

/// Simple Git repo auto-updater.
#[derive(Parser)]
struct Cli {
    /// Path to config TOML
    #[arg(short, long, default_value = "watcher.toml")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let config = Config::load_config(&args.config_file)?;
    println!("{:?}", config);
    Ok(())
}
