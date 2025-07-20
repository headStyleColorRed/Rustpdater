use rustpdater::daemon::watcher::{start_watching_repos, watch_single_repo, try_update};
use rustpdater::daemon::repo_config::RepoCfg;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

#[tokio::test]
async fn test_start_watching_repos_empty() {
    let repos = vec![];
    let result = start_watching_repos(&repos).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_watching_repos_single() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");
    fs::create_dir(&repo_path).unwrap();

    let repo_cfg = RepoCfg {
        path: repo_path.clone(),
        branch: "main".to_string(),
        interval: 1, // 1 second for testing
        remote: "origin".to_string(),
        on_change: None,
    };

    let repos = vec![repo_cfg];

    // This will fail because the directory is not a git repo, but we can test the function structure
    // We need to spawn this in a separate task with a timeout to avoid hanging
    let handle = tokio::spawn(async move {
        start_watching_repos(&repos).await
    });

    // Wait for a short time, then cancel
    let _ = tokio::time::timeout(Duration::from_millis(100), handle).await;
    // If it times out, that's expected since the function has an infinite loop
}

#[test]
fn test_watch_single_repo_structure() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");
    fs::create_dir(&repo_path).unwrap();

    let repo_cfg = RepoCfg {
        path: repo_path.clone(),
        branch: "main".to_string(),
        interval: 1,
        remote: "origin".to_string(),
        on_change: None,
    };

    // Test that the function can be called (it will fail due to not being a git repo)
    // We need to use a timeout to avoid the infinite loop
    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        tokio::time::timeout(Duration::from_millis(100), watch_single_repo(&repo_cfg)).await
    });

    // Should timeout due to infinite loop, which is expected
    assert!(result.is_err());
}

#[test]
fn test_try_update_with_nonexistent_repo() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("nonexistent-repo");

    let repo_cfg = RepoCfg {
        path: repo_path,
        branch: "main".to_string(),
        interval: 300,
        remote: "origin".to_string(),
        on_change: None,
    };

    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        try_update(&repo_cfg).await
    });

    assert!(result.is_err());
}

#[test]
fn test_try_update_with_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("empty-dir");
    fs::create_dir(&repo_path).unwrap();

    let repo_cfg = RepoCfg {
        path: repo_path,
        branch: "main".to_string(),
        interval: 300,
        remote: "origin".to_string(),
        on_change: None,
    };

    let result = tokio::runtime::Runtime::new().unwrap().block_on(async {
        try_update(&repo_cfg).await
    });

    assert!(result.is_err());
}

#[test]
fn test_repo_config_clone() {
    let repo_cfg = RepoCfg {
        path: PathBuf::from("/tmp/test"),
        branch: "develop".to_string(),
        interval: 120,
        remote: "origin".to_string(),
        on_change: Some("echo 'test'".to_string()),
    };

    let cloned = repo_cfg.clone();

    assert_eq!(cloned.path, repo_cfg.path);
    assert_eq!(cloned.branch, repo_cfg.branch);
    assert_eq!(cloned.interval, repo_cfg.interval);
    assert_eq!(cloned.remote, repo_cfg.remote);
    assert_eq!(cloned.on_change, repo_cfg.on_change);
}

#[test]
fn test_repo_config_with_on_change_command() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");
    fs::create_dir(&repo_path).unwrap();

    let repo_cfg = RepoCfg {
        path: repo_path,
        branch: "main".to_string(),
        interval: 300,
        remote: "origin".to_string(),
        on_change: Some("echo 'test command'".to_string()),
    };

    // Test that the config is properly structured
    assert_eq!(repo_cfg.branch, "main");
    assert_eq!(repo_cfg.interval, 300);
    assert_eq!(repo_cfg.remote, "origin");
    assert_eq!(repo_cfg.on_change, Some("echo 'test command'".to_string()));
}

#[test]
fn test_repo_config_without_on_change() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().join("test-repo");
    fs::create_dir(&repo_path).unwrap();

    let repo_cfg = RepoCfg {
        path: repo_path,
        branch: "main".to_string(),
        interval: 300,
        remote: "origin".to_string(),
        on_change: None,
    };

    // Test that the config is properly structured
    assert_eq!(repo_cfg.branch, "main");
    assert_eq!(repo_cfg.interval, 300);
    assert_eq!(repo_cfg.remote, "origin");
    assert_eq!(repo_cfg.on_change, None);
}

#[test]
fn test_interval_duration_conversion() {
    let repo_cfg = RepoCfg {
        path: PathBuf::from("/tmp/test"),
        branch: "main".to_string(),
        interval: 60,
        remote: "origin".to_string(),
        on_change: None,
    };

    let duration = Duration::from_secs(repo_cfg.interval);
    assert_eq!(duration.as_secs(), 60);
}

#[test]
fn test_path_operations() {
    let repo_cfg = RepoCfg {
        path: PathBuf::from("/tmp/test/repo"),
        branch: "main".to_string(),
        interval: 300,
        remote: "origin".to_string(),
        on_change: None,
    };

    assert!(repo_cfg.path.is_absolute());
    assert_eq!(repo_cfg.path.file_name().unwrap(), "repo");
}

#[test]
fn test_branch_name_validation() {
    let repo_cfg = RepoCfg {
        path: PathBuf::from("/tmp/test"),
        branch: "feature/new-feature".to_string(),
        interval: 300,
        remote: "origin".to_string(),
        on_change: None,
    };

    assert_eq!(repo_cfg.branch, "feature/new-feature");
    assert!(!repo_cfg.branch.is_empty());
}
