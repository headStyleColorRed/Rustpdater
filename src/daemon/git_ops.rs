use super::errors::Result;
use super::repo_config::RepoCfg;
use git2::{Repository, Cred, RemoteCallbacks, Error as GitError};
use std::process::Command;
use std::fs;
use std::path::Path;
use std::io::{self, Write};
use log::{debug, info, warn};

pub fn try_update(repo: &RepoCfg) -> Result<()> {
    debug!("Checking repo {} for updates", repo.path.display());

    let repository = Repository::open(&repo.path)?;

    // Fetch with authentication
    let mut remote = repository.find_remote("origin")?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|url, username_from_url, _allowed_types| {
        // Check if this is an SSH URL
        if url.starts_with("git@") || url.starts_with("ssh://") {
            info!("Attempting SSH authentication for {}", url);

            // Try SSH key from SSH agent first
            if let Ok(ssh_key) = Cred::ssh_key_from_agent(username_from_url.unwrap_or("git")) {
                info!("SSH authentication successful via SSH agent");
                return Ok(ssh_key);
            }

            // Try default SSH key locations
            let ssh_key_paths = [
                format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap_or_else(|_| "~".to_string())),
                format!("{}/.ssh/id_ed25519", std::env::var("HOME").unwrap_or_else(|_| "~".to_string())),
                format!("{}/.ssh/id_ecdsa", std::env::var("HOME").unwrap_or_else(|_| "~".to_string())),
            ];

            for key_path in &ssh_key_paths {
                if Path::new(key_path).exists() {
                    if let Ok(ssh_key) = Cred::ssh_key(username_from_url.unwrap_or("git"), None, Path::new(key_path), None) {
                        info!("SSH authentication successful with key: {}", key_path);
                        return Ok(ssh_key);
                    }
                }
            }

            // If SSH authentication fails, fall back to git-credentials
            warn!("SSH authentication failed, falling back to git-credentials");
            handle_https_credentials(username_from_url)
        } else {
            // HTTPS URL - use git-credentials
            info!("Using git-credentials for HTTPS URL: {}", url);
            handle_https_credentials(username_from_url)
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

/// Test if SSH authentication works for a given SSH URL and username.
/// Returns Ok(()) if authentication is successful, otherwise returns an error.
pub fn test_ssh_connection(ssh_url: &str, username: Option<&str>) -> std::result::Result<(), String> {
    // Only allow SSH URLs
    if !(ssh_url.starts_with("git@") || ssh_url.starts_with("ssh://")) {
        return Err("Provided URL is not an SSH URL".to_string());
    }

    let username = username.unwrap_or("git");

    // Try SSH key from SSH agent first
    if let Ok(_ssh_key) = Cred::ssh_key_from_agent(username) {
        // Try to open a session using ssh -T
        let output = Command::new("ssh")
            .arg("-T")
            .arg(format!("{}@{}", username, extract_host_from_ssh_url(ssh_url)))
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    return Ok(());
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("SSH agent authentication failed: {}", stderr));
                }
            }
            Err(e) => {
                return Err(format!("Failed to run ssh command: {}", e));
            }
        }
    }

    // Try default SSH key locations
    let ssh_key_paths = [
        format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap_or_else(|_| "~".to_string())),
        format!("{}/.ssh/id_ed25519", std::env::var("HOME").unwrap_or_else(|_| "~".to_string())),
        format!("{}/.ssh/id_ecdsa", std::env::var("HOME").unwrap_or_else(|_| "~".to_string())),
    ];

    for key_path in &ssh_key_paths {
        if Path::new(key_path).exists() {
            let output = Command::new("ssh")
                .arg("-i")
                .arg(key_path)
                .arg("-T")
                .arg(format!("{}@{}", username, extract_host_from_ssh_url(ssh_url)))
                .output();

            match output {
                Ok(output) => {
                    if output.status.success() {
                        return Ok(());
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        // Try next key
                        continue;
                    }
                }
                Err(e) => {
                    // Try next key
                    continue;
                }
            }
        }
    }

    Err("SSH authentication failed for all known methods".to_string())
}

/// Helper to extract the host from an SSH URL (git@host:path or ssh://host/path)
fn extract_host_from_ssh_url(url: &str) -> String {
    if url.starts_with("git@") {
        // git@host:path
        if let Some(at_pos) = url.find('@') {
            if let Some(colon_pos) = url[at_pos..].find(':') {
                return url[at_pos + 1..at_pos + colon_pos].to_string();
            }
        }
    } else if url.starts_with("ssh://") {
        // ssh://host/path
        let without_prefix = &url[6..];
        if let Some(slash_pos) = without_prefix.find('/') {
            return without_prefix[..slash_pos].to_string();
        }
    }
    // Fallback: return the whole url
    url.to_string()
}

fn handle_https_credentials(username_from_url: Option<&str>) -> std::result::Result<Cred, GitError> {
    match read_git_credentials() {
        Ok(Some(credentials)) => {
            // Use username from credentials file, fallback to URL username
            let username = username_from_url.unwrap_or(&credentials.username);
            Cred::userpass_plaintext(username, &credentials.password)
        }
        Ok(None) => {
            // File exists but no valid credentials found
            warn!("No valid credentials found in ~/.git-credentials, prompting for new credentials");
            match prompt_and_create_credentials() {
                Ok(credentials) => {
                    let username = username_from_url.unwrap_or(&credentials.username);
                    Cred::userpass_plaintext(username, &credentials.password)
                }
                Err(_) => Err(GitError::from_str("Failed to get credentials from user"))
            }
        }
        Err(CredentialsError::FileNotFound) => {
            // File doesn't exist, prompt to create it
            warn!("~/.git-credentials doesn't exist, prompting to create it");
            match prompt_and_create_credentials() {
                Ok(credentials) => {
                    let username = username_from_url.unwrap_or(&credentials.username);
                    Cred::userpass_plaintext(username, &credentials.password)
                }
                Err(_) => Err(GitError::from_str("Failed to create credentials file"))
            }
        }
        Err(CredentialsError::ReadError) => {
            // File exists but couldn't be read
            Err(GitError::from_str("Could not read ~/.git-credentials"))
        }
    }
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

fn prompt_and_create_credentials() -> std::result::Result<GitCredentials, Box<dyn std::error::Error>> {
    println!("\n=== Git Credentials Setup ===");
    println!("The ~/.git-credentials file is missing or empty.");
    println!("This file is needed to authenticate with GitHub repositories.");

    // Ask if user wants to create the file
    print!("Would you like to create the ~/.git-credentials file? (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().to_lowercase().starts_with('y') {
        return Err("User declined to create credentials file".into());
    }

    // Get username
    print!("Enter your GitHub username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim().to_string();

    if username.is_empty() {
        return Err("Username cannot be empty".into());
    }

    // Get token
    print!("Enter your GitHub personal access token: ");
    io::stdout().flush()?;
    let mut token = String::new();
    io::stdin().read_line(&mut token)?;
    let token = token.trim().to_string();

    if token.is_empty() {
        return Err("Token cannot be empty".into());
    }

    // Create credentials
    let credentials = GitCredentials {
        username: username.clone(),
        password: token.clone(),
    };

    // Create the file
    let credentials_path = get_credentials_path();
    let credentials_content = format!("https://{}:{}@github.com\n", username, token);

    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(&credentials_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&credentials_path, credentials_content)?;

    // Set proper permissions (read/write for owner only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&credentials_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&credentials_path, perms)?;
    }

    println!("✓ Created ~/.git-credentials with your credentials");
    println!("✓ File permissions set to owner-only access");

    Ok(credentials)
}

fn get_credentials_path() -> String {
    std::env::var("HOME")
        .ok()
        .map(|home| format!("{}/.git-credentials", home))
        .unwrap_or_else(|| "~/.git-credentials".to_string())
}

fn read_git_credentials() -> std::result::Result<Option<GitCredentials>, CredentialsError> {
    let credentials_path = get_credentials_path();

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
