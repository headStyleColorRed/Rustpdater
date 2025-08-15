use super::errors::Result;
use super::repo_config::RepoCfg;
use git2::{Repository, Cred, RemoteCallbacks};
use std::process::Command;
use std::time::Duration;
use std::fs;
use std::path::Path;
use tokio::{task, time};
use log::{debug, error, info};

pub async fn start_watching_repos(repos: &[RepoCfg]) -> Result<()> {
    let mut tasks = Vec::new();

    info!("Starting watcher with {} repos", repos.len());

    for repo in repos {
        let repo = repo.clone();
        tasks.push(task::spawn(async move { watch_single_repo(&repo).await }));
    }

    for task in tasks {
        task.await??;
    }

    Ok(())
}

async fn watch_single_repo(repo: &RepoCfg) -> Result<()> {
    let interval = Duration::from_secs(repo.interval);
    info!("Watching repo '{}' (branch '{}') every {}s", repo.path.display(), repo.branch, repo.interval);

    loop {
        if let Err(error) = try_update(repo) {
            error!("watcher error on {}: {}", repo.path.display(), error);
            std::process::exit(1);
        }
        time::sleep(interval).await;
    }
}

fn try_update(repo: &RepoCfg) -> Result<()> {
    debug!("Checking repo {} for updates", repo.path.display());

    let repository = Repository::open(&repo.path)?;

    // Fetch with authentication
    let mut remote = repository.find_remote("origin")?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        match read_git_credentials() {
            Ok(Some(credentials)) => {
                // Use username from credentials file, fallback to URL username
                let username = username_from_url.unwrap_or(&credentials.username);
                Cred::userpass_plaintext(username, &credentials.password)
            }
            Ok(None) => {
                // File exists but no valid credentials found
                Err(git2::Error::from_str("No valid credentials found in ~/.git-credentials"))
            }
            Err(CredentialsError::FileNotFound) => {
                // File doesn't exist
                Err(git2::Error::from_str("~/.git-credentials doesn't exist"))
            }
            Err(CredentialsError::ReadError) => {
                // File exists but couldn't be read
                Err(git2::Error::from_str("Could not read ~/.git-credentials"))
            }
        }
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

#[derive(Debug)]
struct GitCredentials {
    username: String,
    password: String,
}

#[derive(Debug)]
enum CredentialsError {
    FileNotFound,
    ReadError,
}

fn read_git_credentials() -> std::result::Result<Option<GitCredentials>, CredentialsError> {
    let credentials_path = std::env::var("HOME")
        .ok()
        .map(|home| format!("{}/.git-credentials", home))
        .unwrap_or_else(|| "~/.git-credentials".to_string());

    // Handle tilde expansion manually since we don't want to add shellexpand dependency
    let credentials_path = if credentials_path.starts_with("~/") {
        std::env::var("HOME")
            .ok()
            .map(|home| format!("{}/{}", home, &credentials_path[2..]))
            .unwrap_or(credentials_path)
    } else {
        credentials_path
    };

    if !Path::new(&credentials_path).exists() {
        return Err(CredentialsError::FileNotFound);
    }

    let content = match fs::read_to_string(&credentials_path) {
        Ok(content) => content,
        Err(_) => return Err(CredentialsError::ReadError),
    };

    // Parse git-credentials format: https://username:token@hostname
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(credentials) = parse_git_credential_line(line) {
            return Ok(Some(credentials));
        }
    }

    Ok(None)
}

fn parse_git_credential_line(line: &str) -> Option<GitCredentials> {
    // Handle format: https://username:token@hostname
    if let Some(auth_part) = line.split("://").nth(1) {
        if let Some(at_pos) = auth_part.find('@') {
            let auth = &auth_part[..at_pos];
            if let Some(colon_pos) = auth.find(':') {
                let username = auth[..colon_pos].to_string();
                let password = auth[colon_pos + 1..].to_string();
                return Some(GitCredentials { username, password });
            }
        }
    }

    None
}
