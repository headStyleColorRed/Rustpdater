use clap::Parser;
use std::error::Error;

/// Simple Git repo auto-updater.
#[derive(Parser)]
struct Cli {
    /// Path to config TOML
    #[arg(short, long, default_value = "watcher.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    println!("{}", args.config);
    Ok(())
}
