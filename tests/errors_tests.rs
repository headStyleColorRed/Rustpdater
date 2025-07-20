use rustpdater::daemon::errors::{WatchError, Result};
use std::io;

#[test]
fn test_git_error_conversion() {
    // Create a git2 error (this is a bit contrived since git2::Error is opaque)
    let git_error = git2::Error::from_str("test git error");
    let watch_error: WatchError = git_error.into();

    match watch_error {
        WatchError::Git(_) => {
            // Expected
        }
        _ => panic!("Expected Git error"),
    }
}

#[test]
fn test_config_error_creation() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let config_error = WatchError::Config {
        path: "test.toml".to_string(),
        source: io_error,
    };

    match config_error {
        WatchError::Config { path, source } => {
            assert_eq!(path, "test.toml");
            assert_eq!(source.kind(), io::ErrorKind::NotFound);
        }
        _ => panic!("Expected Config error"),
    }
}

#[test]
fn test_io_error_conversion() {
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
    let watch_error: WatchError = io_error.into();

    match watch_error {
        WatchError::Io(_) => {
            // Expected
        }
        _ => panic!("Expected Io error"),
    }
}

#[test]
fn test_toml_error_conversion() {
    let invalid_toml = "invalid = [";
    let toml_error = toml::from_str::<String>(invalid_toml).unwrap_err();
    let watch_error: WatchError = toml_error.into();

    match watch_error {
        WatchError::Toml(_) => {
            // Expected
        }
        _ => panic!("Expected Toml error"),
    }
}

#[tokio::test]
async fn test_join_error_conversion() {
    // Create a JoinError by spawning a task that panics
    let handle = tokio::spawn(async {
        panic!("test panic");
    });

    let join_error = handle.await.unwrap_err();
    let watch_error: WatchError = join_error.into();

    match watch_error {
        WatchError::Join(_) => {
            // Expected
        }
        _ => panic!("Expected Join error"),
    }
}

#[test]
fn test_error_display_formatting() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let config_error = WatchError::Config {
        path: "config.toml".to_string(),
        source: io_error,
    };

    let error_string = format!("{}", config_error);
    assert!(error_string.contains("config error: could not load config file 'config.toml'"));
    assert!(error_string.contains("file not found"));
}

#[test]
fn test_git_error_display() {
    let git_error = git2::Error::from_str("repository not found");
    let watch_error = WatchError::Git(git_error);

    let error_string = format!("{}", watch_error);
    assert!(error_string.contains("git error:"));
}

#[test]
fn test_io_error_display() {
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let watch_error = WatchError::Io(io_error);

    let error_string = format!("{}", watch_error);
    assert!(error_string.contains("io error:"));
}

#[test]
fn test_toml_error_display() {
    let invalid_toml = "invalid = [";
    let toml_error = toml::from_str::<String>(invalid_toml).unwrap_err();
    let watch_error = WatchError::Toml(toml_error);

    let error_string = format!("{}", watch_error);
    assert!(error_string.contains("toml error:"));
}

#[tokio::test]
async fn test_join_error_display() {
    let handle = tokio::spawn(async {
        panic!("test panic");
    });

    let join_error = handle.await.unwrap_err();
    let watch_error = WatchError::Join(join_error);

    let error_string = format!("{}", watch_error);
    assert!(error_string.contains("task join error:"));
}

#[test]
fn test_error_debug_formatting() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "test error");
    let config_error = WatchError::Config {
        path: "test.toml".to_string(),
        source: io_error,
    };

    let debug_string = format!("{:?}", config_error);
    assert!(debug_string.contains("Config"));
    assert!(debug_string.contains("test.toml"));
}

#[test]
fn test_result_type_alias() {
    // Test that Result<T> is properly aliased
    let _result: Result<()> = Ok(());
    let _result: Result<String> = Ok("test".to_string());
    let _result: Result<i32> = Err(WatchError::Io(io::Error::new(io::ErrorKind::Other, "test")));
}
