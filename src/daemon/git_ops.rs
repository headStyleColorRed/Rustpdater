use super::errors::Result;
use super::repo_config::RepoCfg;
use git2::{Repository, Cred, RemoteCallbacks, Error as GitError};
use std::process::Command;
use std::path::Path;
use std::fs;
use std::env;
use log::{debug, info};

pub fn try_update(repo: &RepoCfg) -> Result<()> {
    debug!("Checking repo {} for updates", repo.path.display());

    let repository = Repository::open(&repo.path)?;

    // Fetch with authentication
    let mut remote = repository.find_remote("origin")?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, username_from_url, _| {
        // Use SSH agent for authentication
        Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    info!("Fetching '{}' for {}", repo.branch, repo.path.display());
    remote.fetch(&[&repo.branch], Some(&mut fetch_options), None)?;

    // Get HEADs
    let fetch_head = repository.find_reference("FETCH_HEAD")?.target().unwrap();
    let local_head = repository.head()?.target().unwrap();

    // If there's nothing new, escape
    if fetch_head == local_head {
        debug!("No changes detected for {}", repo.path.display());
        return Ok(());
    };

    // Let's do a fast forward merge
    repository.set_head_detached(fetch_head)?;
    repository.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    info!("Fast-forwarded repo {} to new HEAD", repo.path.display());

    if let Some(cmd) = &repo.on_change {
        info!("Running on_change hook for {}: {}", repo.path.display(), cmd);
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(&repo.path)
            .status()?;
    }

    Ok(())
}

/// Test git pull in a temporary folder to verify git operations work
pub fn test_git_pull_in_tmp(repo_path: &Path) -> Result<()> {
    // Get the remote URL from the existing repository
    let repo = Repository::open(repo_path)?;
    let remote = repo.find_remote("origin")?;
    let remote_url = remote.url().ok_or_else(|| {
        GitError::from_str("Could not get remote URL")
    })?;

    // Create a temporary directory
    let temp_dir = env::temp_dir().join(format!("rustpdater_test_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));

    fs::create_dir(&temp_dir)?;

    // Clone the repository into the temp directory
    info!("Testing git pull by cloning {} into temporary directory", remote_url);
    let _temp_repo = Repository::clone(remote_url, &temp_dir)?;

    // Clean up the temporary directory
    fs::remove_dir_all(&temp_dir)?;

    info!("Git pull test successful in temporary directory");
    Ok(())
}
