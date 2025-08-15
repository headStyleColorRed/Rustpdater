use super::errors::Result;
use super::repo_config::RepoCfg;
use super::git_ops;
use tokio::{task, time};
use std::time::Duration;
use log::{error, info};

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
        if let Err(error) = git_ops::try_update(repo) {
            error!("watcher error on {}: {}", repo.path.display(), error);
        }
        time::sleep(interval).await;
    }
}
