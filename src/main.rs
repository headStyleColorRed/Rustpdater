mod daemon;

use clap::Parser;
use std::error::Error;
use std::time::Duration;
use daemon::config::Config;
use daemon::watcher;

/// Simple Git repo auto-updater.
#[derive(Parser, Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_cli_default_config_file() {
        let cli = Cli::parse_from(&["rustpdater"]);
        assert_eq!(cli.config_file, "etc/watcher.toml");
    }

    #[test]
    fn test_cli_custom_config_file() {
        let cli = Cli::parse_from(&["rustpdater", "--config-file", "custom.toml"]);
        assert_eq!(cli.config_file, "custom.toml");
    }

    #[test]
    fn test_cli_short_config_file() {
        let cli = Cli::parse_from(&["rustpdater", "-c", "short.toml"]);
        assert_eq!(cli.config_file, "short.toml");
    }

    #[test]
    fn test_cli_help() {
        let result = Cli::try_parse_from(&["rustpdater", "--help"]);
        assert!(result.is_err()); // clap exits with error for help
    }

        #[tokio::test]
    async fn test_run_with_valid_config() {
        let config_content = r#"
            [[repos]]
            path = "/tmp/test-repo"
            branch = "main"
            interval = 60
        "#;

        let temp_file = "test_run_config.toml";
        fs::write(temp_file, config_content).unwrap();

        let args = Cli {
            config_file: temp_file.to_string(),
        };

        // This will fail because the repo doesn't exist, but we can test the config loading
        // We need to use a timeout to avoid the infinite loop in the watcher
        let result = tokio::time::timeout(Duration::from_millis(100), run(args)).await;

        // Should timeout due to infinite loop in watcher, which is expected
        assert!(result.is_err());

        // Cleanup
        fs::remove_file(temp_file).unwrap();
    }

    #[tokio::test]
    async fn test_run_with_nonexistent_config() {
        let args = Cli {
            config_file: "nonexistent_config.toml".to_string(),
        };

        let result = run(args).await;
        assert!(result.is_err());

        let error_string = result.unwrap_err().to_string();
        assert!(error_string.contains("config error"));
    }

    #[tokio::test]
    async fn test_run_with_invalid_config() {
        let config_content = r#"
            [[repos]]
            path = "/tmp/test-repo"
            interval = "not_a_number"  # Invalid: should be u64
        "#;

        let temp_file = "test_run_invalid_config.toml";
        fs::write(temp_file, config_content).unwrap();

        let args = Cli {
            config_file: temp_file.to_string(),
        };

        let result = run(args).await;
        assert!(result.is_err());

        let error_string = result.unwrap_err().to_string();
        assert!(error_string.contains("toml error"));

        // Cleanup
        fs::remove_file(temp_file).unwrap();
    }

    #[tokio::test]
    async fn test_run_with_empty_config() {
        let config_content = "";

        let temp_file = "test_run_empty_config.toml";
        fs::write(temp_file, config_content).unwrap();

        let args = Cli {
            config_file: temp_file.to_string(),
        };

        let result = run(args).await;
        assert!(result.is_err());

        // Cleanup
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_cli_struct_debug() {
        let cli = Cli {
            config_file: "test.toml".to_string(),
        };

        let debug_str = format!("{:?}", cli);
        assert!(debug_str.contains("test.toml"));
    }

    #[test]
    fn test_cli_struct_clone() {
        let cli = Cli {
            config_file: "test.toml".to_string(),
        };

        let cloned = cli.clone();
        assert_eq!(cloned.config_file, cli.config_file);
    }

    #[test]
    fn test_config_file_path_validation() {
        let cli = Cli {
            config_file: "/absolute/path/config.toml".to_string(),
        };

        assert!(cli.config_file.starts_with('/'));
        assert!(cli.config_file.ends_with(".toml"));
    }

    #[test]
    fn test_config_file_relative_path() {
        let cli = Cli {
            config_file: "./relative/config.toml".to_string(),
        };

        assert!(cli.config_file.starts_with("./"));
        assert!(cli.config_file.ends_with(".toml"));
    }

        #[tokio::test]
    async fn test_run_with_multiple_repos_config() {
        let config_content = r#"
            [[repos]]
            path = "/tmp/repo1"
            branch = "main"
            interval = 60

            [[repos]]
            path = "/tmp/repo2"
            branch = "develop"
            interval = 120
            on_change = "echo 'updated'"
        "#;

        let temp_file = "test_run_multiple_config.toml";
        fs::write(temp_file, config_content).unwrap();

        let args = Cli {
            config_file: temp_file.to_string(),
        };

        // This will fail because the repos don't exist, but we can test the config loading
        // We need to use a timeout to avoid the infinite loop in the watcher
        let result = tokio::time::timeout(Duration::from_millis(100), run(args)).await;

        // Should timeout due to infinite loop in watcher, which is expected
        assert!(result.is_err());

        // Cleanup
        fs::remove_file(temp_file).unwrap();
    }
}
