//! Cursor IDE enumeration module
//!
//! Enumerates Cursor IDE configuration files:
//! - %APPDATA%/Cursor/User/globalStorage/state.vscdb (SQLite settings)
//! - .cursor/rules/ directory (AI rules)
//! - .cursor/environment.json (environment configuration)
//! - .cursorrules file (legacy rules)

use crate::enumeration::{
    ConfigFile, find_config_dirs, find_files_recursive, get_user_home, read_file_if_exists,
};
use serde::Serialize;
use std::fs;

/// Cursor IDE configuration for a user
#[derive(Debug, Serialize)]
pub struct CursorConfig {
    pub user: String,
    pub state_db_path: Option<String>,
    pub rules_files: Vec<String>,
    pub environment_json: Option<ConfigFile>,
    pub cursorrules_files: Vec<String>,
    pub settings_json: Option<ConfigFile>,
}

impl CursorConfig {
    fn new(user: &str) -> Self {
        Self {
            user: user.to_string(),
            state_db_path: None,
            rules_files: Vec::new(),
            environment_json: None,
            cursorrules_files: Vec::new(),
            settings_json: None,
        }
    }

    fn has_any_config(&self) -> bool {
        self.state_db_path.is_some()
            || !self.rules_files.is_empty()
            || self.environment_json.is_some()
            || !self.cursorrules_files.is_empty()
            || self.settings_json.is_some()
    }
}

/// Enumerate Cursor IDE configurations for a user
pub fn enumerate_cursor(user: &str) -> Option<CursorConfig> {
    let mut config = CursorConfig::new(user);
    let user_home = get_user_home(user);

    // Check for Cursor state database
    // Windows: %APPDATA%/Cursor/User/globalStorage/state.vscdb
    // macOS: ~/Library/Application Support/Cursor/User/globalStorage/state.vscdb
    let appdata = crate::platform::get_app_support_dir(&user_home);
    let cursor_db_path = appdata
        .join("Cursor")
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");

    if cursor_db_path.exists() {
        config.state_db_path = Some(cursor_db_path.to_string_lossy().to_string());
    }

    // Also check Local AppData (Windows-specific, but check on all platforms)
    let local_appdata = crate::platform::get_local_app_data_dir(&user_home);
    let cursor_local_db = local_appdata
        .join("Cursor")
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");

    if config.state_db_path.is_none() && cursor_local_db.exists() {
        config.state_db_path = Some(cursor_local_db.to_string_lossy().to_string());
    }

    // Check for user settings.json
    let settings_path = appdata.join("Cursor").join("User").join("settings.json");
    if let Some(content) = read_file_if_exists(&settings_path) {
        config.settings_json = Some(ConfigFile::new(&settings_path.to_string_lossy(), content));
    }

    // Search for .cursor directories with rules
    let cursor_dirs = find_config_dirs(&user_home, ".cursor");
    for dir in cursor_dirs {
        // Check for rules directory
        let rules_dir = dir.join("rules");
        if rules_dir.exists()
            && rules_dir.is_dir()
            && let Ok(entries) = fs::read_dir(&rules_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    config.rules_files.push(path.to_string_lossy().to_string());
                }
            }
        }

        // Check for environment.json
        let env_json = dir.join("environment.json");
        if config.environment_json.is_none()
            && let Some(content) = read_file_if_exists(&env_json)
        {
            config.environment_json = Some(ConfigFile::new(&env_json.to_string_lossy(), content));
        }
    }

    // Search for .cursorrules files (legacy format)
    let cursorrules_files = find_files_recursive(&user_home, ".cursorrules", 4);
    for rules_path in cursorrules_files {
        config
            .cursorrules_files
            .push(rules_path.to_string_lossy().to_string());
    }

    if config.has_any_config() {
        Some(config)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_config_creation() {
        let config = CursorConfig::new("testuser");
        assert_eq!(config.user, "testuser");
        assert!(!config.has_any_config());
    }

    #[test]
    fn test_enumerate_cursor() {
        let result = enumerate_cursor("testuser");
        // Result may or may not exist depending on system state
        let _ = result;
    }
}
