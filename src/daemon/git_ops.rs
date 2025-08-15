use super::errors::Result;
use super::repo_config::RepoCfg;
use std::process::Command;
use std::path::Path;
use std::fs;
use std::env;
use log::{debug, info, warn};

/// Normalize git remote URL to fix malformed URLs but preserve SSH URLs
fn normalize_git_url(url: &str) -> String {
    // Handle malformed URLs with duplicate paths like "git@github.com:/github.com/user/repo.git"
    if url.contains(":/github.com/") {
        // Extract the actual repository path and keep SSH format
        if let Some(repo_part) = url.split(":/github.com/").nth(1) {
            return format!("git@github.com:{}", repo_part);
        }
    }

    // Return original URL if no normalization needed
    url.to_string()
}

/// Execute a git command and return the result
fn execute_git_command(repo_path: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(super::errors::WatchError::GitCommandFailed {
            command: format!("git {}", args.join(" ")),
            stderr: stderr.to_string(),
        });
    }

    Ok(())
}

/// Get the remote URL for a repository
fn get_remote_url(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(super::errors::WatchError::GitCommandFailed {
            command: "git remote get-url origin".to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let url = String::from_utf8(output.stdout)?
        .trim()
        .to_string();

    Ok(url)
}

/// Get the current HEAD commit hash
fn get_current_head(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(super::errors::WatchError::GitCommandFailed {
            command: "git rev-parse HEAD".to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let head = String::from_utf8(output.stdout)?
        .trim()
        .to_string();

    Ok(head)
}

/// Get the FETCH_HEAD commit hash
fn get_fetch_head(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "FETCH_HEAD"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        return Err(super::errors::WatchError::GitCommandFailed {
            command: "git rev-parse FETCH_HEAD".to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let fetch_head = String::from_utf8(output.stdout)?
        .trim()
        .to_string();

    Ok(fetch_head)
}

pub fn try_update(repo: &RepoCfg) -> Result<()> {
    debug!("Checking repo {} for updates", repo.path.display());

    // Fetch with authentication (SSH agent will be used automatically)
    info!("Fetching '{}' for {}", repo.branch, repo.path.display());
    execute_git_command(&repo.path, &["fetch", "origin", &repo.branch])?;

    // Get current HEAD and FETCH_HEAD
    let local_head = get_current_head(&repo.path)?;
    let fetch_head = get_fetch_head(&repo.path)?;

    // If there's nothing new, escape
    if fetch_head == local_head {
        debug!("No changes detected for {}", repo.path.display());
        return Ok(());
    }

    // Reset to the new HEAD (fast-forward)
    info!("Fast-forwarding repo {} to new HEAD", repo.path.display());
    execute_git_command(&repo.path, &["reset", "--hard", &fetch_head])?;

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
    info!("_ _ _ _ _ _ _ _ _ _  TESTING GIT OPERATIONS  _ _ _ _ _ _ _ _ _ _");
    info!("Testing git pull in a temporary folder to verify git operations work");


    // Get the remote URL from the existing repository
    let remote_url = get_remote_url(repo_path)?;

    // Normalize the URL to fix malformed URLs
    let normalized_url = normalize_git_url(&remote_url);
    info!("Original remote URL: {}", remote_url);
    info!("Normalized URL: {}", normalized_url);

    // Create a temporary directory
    let temp_dir = env::temp_dir().join(format!("rustpdater_test_{}",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()));

    fs::create_dir(&temp_dir)?;

    // Clone the repository into the temp directory
    info!("Testing git pull by cloning {} into temporary directory", normalized_url);
    let clone_output = Command::new("git")
        .args(["clone", &normalized_url, temp_dir.to_str().unwrap()])
        .output()?;

    if !clone_output.status.success() {
        let stderr = String::from_utf8_lossy(&clone_output.stderr);
        warn!("Git clone failed: {}", stderr);
        return Err(super::errors::WatchError::GitCommandFailed {
            command: format!("git clone {}", normalized_url),
            stderr: stderr.to_string(),
        });
    }

    // Clean up the temporary directory
    match fs::remove_dir_all(&temp_dir) {
        Ok(_) => info!("Temporary directory removed successfully"),
        Err(e) => warn!("Failed to remove temporary directory: {}", e),
    }

    info!("Git pull test successful in temporary directory");
    info!("_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _\n");
    Ok(())
}
