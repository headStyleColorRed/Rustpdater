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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_load_valid_config() {
        let config_content = r#"
            [[repos]]
            path = "/tmp/test-repo"
            branch = "main"
            interval = 60
            on_change = "echo 'updated'"
        "#;

        let temp_file = "test_config.toml";
        fs::write(temp_file, config_content).unwrap();

        let result = Config::load_config(temp_file);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.repos.len(), 1);
        assert_eq!(config.repos[0].path, Path::new("/tmp/test-repo"));
        assert_eq!(config.repos[0].branch, "main");
        assert_eq!(config.repos[0].interval, 60);
        assert_eq!(config.repos[0].on_change, Some("echo 'updated'".to_string()));

        // Cleanup
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_config_with_defaults() {
        let config_content = r#"
            [[repos]]
            path = "/tmp/test-repo"
        "#;

        let temp_file = "test_config_defaults.toml";
        fs::write(temp_file, config_content).unwrap();

        let result = Config::load_config(temp_file);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.repos.len(), 1);
        assert_eq!(config.repos[0].branch, "master"); // default
        assert_eq!(config.repos[0].interval, 300); // default (5 minutes)
        assert_eq!(config.repos[0].on_change, None);

        // Cleanup
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_config_multiple_repos() {
        let config_content = r#"
            [[repos]]
            path = "/tmp/repo1"
            branch = "develop"
            interval = 120

            [[repos]]
            path = "/tmp/repo2"
            branch = "main"
            on_change = "npm install"
        "#;

        let temp_file = "test_config_multiple.toml";
        fs::write(temp_file, config_content).unwrap();

        let result = Config::load_config(temp_file);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.repos.len(), 2);

        assert_eq!(config.repos[0].path, Path::new("/tmp/repo1"));
        assert_eq!(config.repos[0].branch, "develop");
        assert_eq!(config.repos[0].interval, 120);
        assert_eq!(config.repos[0].on_change, None);

        assert_eq!(config.repos[1].path, Path::new("/tmp/repo2"));
        assert_eq!(config.repos[1].branch, "main");
        assert_eq!(config.repos[1].interval, 300); // default
        assert_eq!(config.repos[1].on_change, Some("npm install".to_string()));

        // Cleanup
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_nonexistent_config() {
        let result = Config::load_config("nonexistent_config.toml");
        assert!(result.is_err());

        match result.unwrap_err() {
            WatchError::Config { path, .. } => {
                assert_eq!(path, "nonexistent_config.toml");
            }
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_load_invalid_toml() {
        let config_content = r#"
            [[repos]]
            path = "/tmp/test-repo"
            branch = "main"
            interval = "not_a_number"  # Invalid: should be u64
        "#;

        let temp_file = "test_config_invalid.toml";
        fs::write(temp_file, config_content).unwrap();

        let result = Config::load_config(temp_file);
        assert!(result.is_err());

        match result.unwrap_err() {
            WatchError::Toml(_) => {
                // Expected TOML parsing error
            }
            _ => panic!("Expected Toml error"),
        }

        // Cleanup
        fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_load_empty_config() {
        let config_content = "";

        let temp_file = "test_config_empty.toml";
        fs::write(temp_file, config_content).unwrap();

        let result = Config::load_config(temp_file);
        assert!(result.is_err());

        match result.unwrap_err() {
            WatchError::Toml(_) => {
                // Expected TOML parsing error for empty content
            }
            _ => panic!("Expected Toml error"),
        }

        // Cleanup
        fs::remove_file(temp_file).unwrap();
    }
}
