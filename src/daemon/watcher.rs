use super::errors::Result;
use super::repo_config::RepoCfg;
use git2::{Repository, Cred, RemoteCallbacks};
use std::process::Command;
use std::time::Duration;
use tokio::{task, time};

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
