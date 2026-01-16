//! Platform-specific path utilities for cua-poison
//!
//! Provides cross-platform abstractions for user home directory resolution.

use std::path::PathBuf;

/// Get the home directory for a specific user or the current user
pub fn get_home_dir(user: Option<&str>) -> PathBuf {
    if let Some(u) = user {
        get_user_home(u)
    } else {
        get_current_user_home()
    }
}

/// Get the home directory path for a specific user
#[cfg(windows)]
fn get_user_home(user: &str) -> PathBuf {
    PathBuf::from(format!("C:\\Users\\{}", user))
}

#[cfg(target_os = "macos")]
fn get_user_home(user: &str) -> PathBuf {
    PathBuf::from(format!("/Users/{}", user))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn get_user_home(user: &str) -> PathBuf {
    PathBuf::from(format!("/home/{}", user))
}

/// Get the home directory for the current user
fn get_current_user_home() -> PathBuf {
    // Try platform-specific environment variables first
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map_or_else(|_| PathBuf::from("C:\\Users\\Default"), PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").map_or_else(|_| get_default_home(), PathBuf::from)
    }
}

/// Get a fallback default home directory
#[cfg(windows)]
#[allow(dead_code)]
fn get_default_home() -> PathBuf {
    PathBuf::from("C:\\Users\\Default")
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn get_default_home() -> PathBuf {
    PathBuf::from("/Users/Shared")
}

#[cfg(all(unix, not(target_os = "macos")))]
#[allow(dead_code)]
fn get_default_home() -> PathBuf {
    PathBuf::from("/tmp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_home() {
        let home = get_user_home("testuser");
        #[cfg(windows)]
        assert!(home.to_string_lossy().contains("C:\\Users\\testuser"));
        #[cfg(target_os = "macos")]
        assert!(home.to_string_lossy().contains("/Users/testuser"));
    }

    #[test]
    fn test_get_current_user_home() {
        let home = get_current_user_home();
        // Should return some path
        assert!(!home.to_string_lossy().is_empty());
    }

    #[test]
    fn test_get_home_dir_with_user() {
        let home = get_home_dir(Some("testuser"));
        #[cfg(windows)]
        assert!(home.to_string_lossy().contains("testuser"));
        #[cfg(target_os = "macos")]
        assert!(home.to_string_lossy().contains("testuser"));
    }
}
