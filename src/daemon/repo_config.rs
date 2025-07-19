use std::path::PathBuf;
use serde::Deserialize;


fn default_branch() -> String { "master".to_string() }
fn default_interval() -> u64 { 300 /*5 minutes in seconds */ }

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
    /// Command to run after update (optional)
    pub on_change: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_config_deserialization_full() {
        let toml_content = r#"
            path = "/tmp/test-repo"
            branch = "develop"
            interval = 120
            on_change = "echo 'updated'"
        "#;

        let repo_cfg: RepoCfg = toml::from_str(toml_content).unwrap();

        assert_eq!(repo_cfg.path, PathBuf::from("/tmp/test-repo"));
        assert_eq!(repo_cfg.branch, "develop");
        assert_eq!(repo_cfg.interval, 120);
        assert_eq!(repo_cfg.on_change, Some("echo 'updated'".to_string()));
    }

    #[test]
    fn test_repo_config_deserialization_minimal() {
        let toml_content = r#"
            path = "/tmp/test-repo"
        "#;

        let repo_cfg: RepoCfg = toml::from_str(toml_content).unwrap();

        assert_eq!(repo_cfg.path, PathBuf::from("/tmp/test-repo"));
        assert_eq!(repo_cfg.branch, "master"); // default
        assert_eq!(repo_cfg.interval, 300); // default (5 minutes)
        assert_eq!(repo_cfg.on_change, None);
    }

    #[test]
    fn test_repo_config_deserialization_partial() {
        let toml_content = r#"
            path = "/tmp/test-repo"
            branch = "main"
            on_change = "npm install"
        "#;

        let repo_cfg: RepoCfg = toml::from_str(toml_content).unwrap();

        assert_eq!(repo_cfg.path, PathBuf::from("/tmp/test-repo"));
        assert_eq!(repo_cfg.branch, "main");
        assert_eq!(repo_cfg.interval, 300); // default
        assert_eq!(repo_cfg.on_change, Some("npm install".to_string()));
    }

    #[test]
    fn test_repo_config_deserialization_with_interval_only() {
        let toml_content = r#"
            path = "/tmp/test-repo"
            interval = 60
        "#;

        let repo_cfg: RepoCfg = toml::from_str(toml_content).unwrap();

        assert_eq!(repo_cfg.path, PathBuf::from("/tmp/test-repo"));
        assert_eq!(repo_cfg.branch, "master"); // default
        assert_eq!(repo_cfg.interval, 60);
        assert_eq!(repo_cfg.on_change, None);
    }

    #[test]
    fn test_repo_config_deserialization_with_on_change_only() {
        let toml_content = r#"
            path = "/tmp/test-repo"
            on_change = "git pull"
        "#;

        let repo_cfg: RepoCfg = toml::from_str(toml_content).unwrap();

        assert_eq!(repo_cfg.path, PathBuf::from("/tmp/test-repo"));
        assert_eq!(repo_cfg.branch, "master"); // default
        assert_eq!(repo_cfg.interval, 300); // default
        assert_eq!(repo_cfg.on_change, Some("git pull".to_string()));
    }

    #[test]
    fn test_repo_config_clone() {
        let repo_cfg = RepoCfg {
            path: PathBuf::from("/tmp/test-repo"),
            branch: "main".to_string(),
            interval: 120,
            on_change: Some("echo 'test'".to_string()),
        };

        let cloned = repo_cfg.clone();

        assert_eq!(cloned.path, repo_cfg.path);
        assert_eq!(cloned.branch, repo_cfg.branch);
        assert_eq!(cloned.interval, repo_cfg.interval);
        assert_eq!(cloned.on_change, repo_cfg.on_change);
    }

    #[test]
    fn test_repo_config_debug_format() {
        let repo_cfg = RepoCfg {
            path: PathBuf::from("/tmp/test-repo"),
            branch: "main".to_string(),
            interval: 120,
            on_change: Some("echo 'test'".to_string()),
        };

        let debug_str = format!("{:?}", repo_cfg);
        assert!(debug_str.contains("RepoCfg"));
        assert!(debug_str.contains("/tmp/test-repo"));
        assert!(debug_str.contains("main"));
        assert!(debug_str.contains("120"));
        assert!(debug_str.contains("echo 'test'"));
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_branch(), "master");
        assert_eq!(default_interval(), 300);
    }

    #[test]
    fn test_repo_config_with_relative_path() {
        let toml_content = r#"
            path = "./relative/path"
            branch = "feature"
            interval = 180
        "#;

        let repo_cfg: RepoCfg = toml::from_str(toml_content).unwrap();

        assert_eq!(repo_cfg.path, PathBuf::from("./relative/path"));
        assert_eq!(repo_cfg.branch, "feature");
        assert_eq!(repo_cfg.interval, 180);
        assert_eq!(repo_cfg.on_change, None);
    }

    #[test]
    fn test_repo_config_with_empty_on_change() {
        let toml_content = r#"
            path = "/tmp/test-repo"
            on_change = ""
        "#;

        let repo_cfg: RepoCfg = toml::from_str(toml_content).unwrap();

        assert_eq!(repo_cfg.path, PathBuf::from("/tmp/test-repo"));
        assert_eq!(repo_cfg.branch, "master"); // default
        assert_eq!(repo_cfg.interval, 300); // default
        assert_eq!(repo_cfg.on_change, Some("".to_string()));
    }
}
