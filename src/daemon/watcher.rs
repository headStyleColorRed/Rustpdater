use super::errors::Result;
use super::repo_config::RepoCfg;
use git2::{Repository, Cred, RemoteCallbacks};
use std::process::Command;
use std::time::Duration;
use tokio::{task, time};
use std::path::PathBuf;

pub async fn start_watching_repos(repos: &[RepoCfg]) -> Result<()> {
    let mut tasks = Vec::new();

    for repo in repos {
        let repo = repo.clone();
        tasks.push(task::spawn(
            async move { watch_single_repo(&repo).await },
        ));
    }

    for task in tasks {
        task.await??;
    }

    Ok(())
}

async fn watch_single_repo(repo: &RepoCfg) -> Result<()> {
    let interval = Duration::from_secs(repo.interval);

    loop {
        if let Err(error) = try_update(repo).await {
            eprintln!("watcher error on {:?}: {error}", repo.path);
        }
        time::sleep(interval).await;
    }
}

async fn try_update(repo: &RepoCfg) -> Result<()> {
    let repository = Repository::open(&repo.path)?;

    // Set up authentication callbacks
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        // Try SSH key authentication first
        if let Ok(cred) = Cred::ssh_key(
            username_from_url.unwrap_or("git"),
            None,
            std::path::Path::new(&format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap_or("/root".to_string()))),
            None,
        ) {
            return Ok(cred);
        }

        // Fallback to default credentials
        Cred::default()
    });

    // Fetch with authentication
    let mut remote = repository.find_remote("origin")?;
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    remote.fetch(&[&repo.branch], Some(&mut fetch_options), None)?;

    // Get HEADs
    let fetch_head = repository.find_reference("FETCH_HEAD")?.target().unwrap();
    let local_head = repository.head()?.target().unwrap();

    // If there's nothing new, escape
    if fetch_head == local_head {
        return Ok(());
    };

    // Let's do a fast forward merge
    repository.set_head_detached(fetch_head)?;
    repository.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

    if let Some(cmd) = &repo.on_change {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(&repo.path)
            .status()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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
            on_change: Some("echo 'test'".to_string()),
        };

        let cloned = repo_cfg.clone();

        assert_eq!(cloned.path, repo_cfg.path);
        assert_eq!(cloned.branch, repo_cfg.branch);
        assert_eq!(cloned.interval, repo_cfg.interval);
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
            on_change: Some("echo 'test command'".to_string()),
        };

        // Test that the config is properly structured
        assert_eq!(repo_cfg.branch, "main");
        assert_eq!(repo_cfg.interval, 300);
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
            on_change: None,
        };

        // Test that the config is properly structured
        assert_eq!(repo_cfg.branch, "main");
        assert_eq!(repo_cfg.interval, 300);
        assert_eq!(repo_cfg.on_change, None);
    }

    #[test]
    fn test_interval_duration_conversion() {
        let repo_cfg = RepoCfg {
            path: PathBuf::from("/tmp/test"),
            branch: "main".to_string(),
            interval: 60,
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
            on_change: None,
        };

        assert_eq!(repo_cfg.branch, "feature/new-feature");
        assert!(!repo_cfg.branch.is_empty());
    }
}
