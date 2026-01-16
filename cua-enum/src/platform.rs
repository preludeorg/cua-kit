//! Platform-specific path and user enumeration utilities
//!
//! Provides cross-platform abstractions for:
//! - User directory enumeration
//! - Home directory resolution
//! - Application data directory paths

use std::path::{Path, PathBuf};

/// Get the root directory containing user home directories
#[cfg(windows)]
pub fn get_users_root() -> &'static Path {
    Path::new("C:\\Users")
}

#[cfg(target_os = "macos")]
pub fn get_users_root() -> &'static Path {
    Path::new("/Users")
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_users_root() -> &'static Path {
    Path::new("/home")
}

/// Get the home directory path for a given user
#[cfg(windows)]
pub fn get_user_home(user: &str) -> PathBuf {
    PathBuf::from(format!("C:\\Users\\{}", user))
}

#[cfg(target_os = "macos")]
pub fn get_user_home(user: &str) -> PathBuf {
    PathBuf::from(format!("/Users/{}", user))
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_user_home(user: &str) -> PathBuf {
    PathBuf::from(format!("/home/{}", user))
}

/// Get the list of system user names to skip during enumeration
#[cfg(windows)]
pub fn get_skip_users() -> &'static [&'static str] {
    &["Public", "Default", "Default User", "All Users"]
}

#[cfg(target_os = "macos")]
pub fn get_skip_users() -> &'static [&'static str] {
    &["Shared", "Guest"]
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_skip_users() -> &'static [&'static str] {
    &[]
}

/// Check if a username should be skipped (system user)
#[cfg(target_os = "macos")]
pub fn should_skip_user(name: &str) -> bool {
    // On macOS, skip users that start with underscore (system users)
    // as well as the explicit skip list
    name.starts_with('_') || get_skip_users().contains(&name)
}

#[cfg(not(target_os = "macos"))]
pub fn should_skip_user(name: &str) -> bool {
    get_skip_users().contains(&name)
}

/// Get the Application Support / AppData directory for a user
/// Windows: %APPDATA% (C:\Users\{user}\AppData\Roaming)
/// macOS: ~/Library/Application Support
#[cfg(windows)]
pub fn get_app_support_dir(user_home: &Path) -> PathBuf {
    user_home.join("AppData").join("Roaming")
}

#[cfg(target_os = "macos")]
pub fn get_app_support_dir(user_home: &Path) -> PathBuf {
    user_home.join("Library").join("Application Support")
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_app_support_dir(user_home: &Path) -> PathBuf {
    // Linux uses XDG_CONFIG_HOME or ~/.config
    user_home.join(".config")
}

/// Get the local application data directory for a user
/// Windows: %LOCALAPPDATA% (C:\Users\{user}\AppData\Local)
/// macOS: ~/Library/Caches
#[cfg(windows)]
pub fn get_local_app_data_dir(user_home: &Path) -> PathBuf {
    user_home.join("AppData").join("Local")
}

#[cfg(target_os = "macos")]
pub fn get_local_app_data_dir(user_home: &Path) -> PathBuf {
    user_home.join("Library").join("Caches")
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_local_app_data_dir(user_home: &Path) -> PathBuf {
    // Linux uses XDG_CACHE_HOME or ~/.cache
    user_home.join(".cache")
}

/// Get the system-wide managed settings directory for Claude Code
/// Windows: C:\Program Files\ClaudeCode
/// macOS: /Library/Application Support/ClaudeCode
#[cfg(windows)]
pub fn get_managed_settings_dir() -> PathBuf {
    PathBuf::from("C:\\Program Files\\ClaudeCode")
}

#[cfg(target_os = "macos")]
pub fn get_managed_settings_dir() -> PathBuf {
    PathBuf::from("/Library/Application Support/ClaudeCode")
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn get_managed_settings_dir() -> PathBuf {
    // Linux doesn't have an official path, but follow XDG conventions
    PathBuf::from("/etc/claude-code")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_users_root() {
        let root = get_users_root();
        #[cfg(windows)]
        assert_eq!(root, Path::new("C:\\Users"));
        #[cfg(target_os = "macos")]
        assert_eq!(root, Path::new("/Users"));
    }

    #[test]
    fn test_get_user_home() {
        let home = get_user_home("testuser");
        #[cfg(windows)]
        assert!(home.to_string_lossy().contains("C:\\Users\\testuser"));
        #[cfg(target_os = "macos")]
        assert!(home.to_string_lossy().contains("/Users/testuser"));
    }

    #[test]
    fn test_should_skip_user() {
        #[cfg(windows)]
        {
            assert!(should_skip_user("Public"));
            assert!(should_skip_user("Default"));
            assert!(!should_skip_user("matt"));
        }
        #[cfg(target_os = "macos")]
        {
            assert!(should_skip_user("Shared"));
            assert!(should_skip_user("Guest"));
            assert!(should_skip_user("_spotlight"));
            assert!(!should_skip_user("matt"));
        }
    }

    #[test]
    fn test_get_app_support_dir() {
        let home = get_user_home("testuser");
        let app_support = get_app_support_dir(&home);
        #[cfg(windows)]
        assert!(app_support.to_string_lossy().contains("AppData\\Roaming"));
        #[cfg(target_os = "macos")]
        assert!(
            app_support
                .to_string_lossy()
                .contains("Library/Application Support")
        );
    }

    #[test]
    fn test_get_managed_settings_dir() {
        let managed = get_managed_settings_dir();
        #[cfg(windows)]
        assert!(
            managed
                .to_string_lossy()
                .contains("Program Files\\ClaudeCode")
        );
        #[cfg(target_os = "macos")]
        assert!(
            managed
                .to_string_lossy()
                .contains("/Library/Application Support/ClaudeCode")
        );
    }
}
