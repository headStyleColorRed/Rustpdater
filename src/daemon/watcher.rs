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

pub async fn watch_single_repo(repo: &RepoCfg) -> Result<()> {
    let interval = Duration::from_secs(repo.interval);

    loop {
        if let Err(error) = try_update(repo).await {
            eprintln!("watcher error on {:?}: {error}", repo.path);
        }
        time::sleep(interval).await;
    }
}

pub async fn try_update(repo: &RepoCfg) -> Result<()> {
    // Clone the repo config for the blocking task
    let repo_path = repo.path.clone();
    let repo_branch = repo.branch.clone();
    let repo_remote = repo.remote.clone();
    let repo_on_change = repo.on_change.clone();

    // Wrap all blocking git operations in spawn_blocking
    let result = task::spawn_blocking(move || {
        perform_git_update(&repo_path, &repo_branch, &repo_remote, &repo_on_change)
    }).await?;

    result
}

fn perform_git_update(
    repo_path: &PathBuf,
    branch: &str,
    remote_name: &str,
    on_change: &Option<String>,
) -> Result<()> {
    let repository = Repository::open(repo_path)?;

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

    // Fetch with authentication using configurable remote
    let mut remote = repository.find_remote(remote_name)?;
    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);
    remote.fetch(&[branch], Some(&mut fetch_options), None)?;

    // Get HEADs
    let fetch_head = repository.find_reference("FETCH_HEAD")?.target().unwrap();
    let local_head = repository.head()?.target().unwrap();

    // If there's nothing new, escape
    if fetch_head == local_head {
        return Ok(());
    }

    // Perform proper branch-based fast-forward merge instead of detached HEAD
    let branch_ref_name = format!("refs/heads/{}", branch);

        // Check if the branch exists locally
    match repository.find_branch(branch, git2::BranchType::Local) {
        Ok(_) => {
            // Branch exists, update it to point to the new commit
            let mut reference = repository.find_reference(&branch_ref_name)?;
            reference.set_target(fetch_head, "Fast-forward merge by rustpdater")?;

            // Set HEAD to point to the updated branch
            repository.set_head(&branch_ref_name)?;
        }
        Err(_) => {
            // Branch doesn't exist locally, create it
            let commit = repository.find_commit(fetch_head)?;
            repository.branch(branch, &commit, false)?;
            repository.set_head(&branch_ref_name)?;
        }
    }

    // Checkout the updated branch
    repository.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;

    // Run on_change command if specified
    if let Some(cmd) = on_change {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(repo_path)
            .status()?;
    }

    Ok(())
}


