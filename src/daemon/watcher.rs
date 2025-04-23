use super::errors::Result;
use super::repo_config::RepoCfg;
use git2::Repository;
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
        if let Err(error) = try_update(repo) {
            eprintln!("watcher error on {:?}: {error}", repo.path);
        }
        time::sleep(interval).await;
    }
}

fn try_update(repo: &RepoCfg) -> Result<()> {
    let repository = Repository::open(&repo.path)?;

    // Fetch
    let mut remote = repository.find_remote("origin")?;
    remote.fetch(&[&repo.branch], None, None)?;

    // Get HEADs
    let fetch_head = repository.find_reference("FETCH_HEAD")?.target().unwrap();
    let local_head = repository.head()?.target().unwrap();

    // If there's nothing new, scape
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
