//! Core enumeration types and utilities
//!
//! Provides shared types and helper functions used by all agent enumerators.

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Results from enumerating all agents
#[derive(Debug, Serialize, Default)]
pub struct EnumerationResults {
    pub claude_code: Vec<crate::agents::claude::ClaudeCodeConfig>,
    pub codex_cli: Vec<crate::agents::codex::CodexConfig>,
    pub cursor: Vec<crate::agents::cursor::CursorConfig>,
    pub gemini_cli: Vec<crate::agents::gemini::GeminiConfig>,
    pub agents_md: Vec<String>,
}

impl EnumerationResults {
    pub fn new() -> Self {
        Self::default()
    }
}

/// A configuration file with path and content
#[derive(Debug, Serialize, Clone)]
pub struct ConfigFile {
    pub path: String,
    pub content: String,
}

impl ConfigFile {
    pub fn new(path: &str, content: String) -> Self {
        Self {
            path: path.to_string(),
            content,
        }
    }
}

/// Get all user directories on the system
pub fn get_all_users() -> Vec<String> {
    let mut users = Vec::new();

    let users_dir = crate::platform::get_users_root();
    if users_dir.exists()
        && let Ok(entries) = fs::read_dir(users_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Some(name) = path.file_name()
            {
                let name_str = name.to_string_lossy().to_string();
                // Skip system directories using platform-specific logic
                if !crate::platform::should_skip_user(&name_str) {
                    users.push(name_str);
                }
            }
        }
    }

    users
}

/// Get the home directory path for a given user
pub fn get_user_home(user: &str) -> PathBuf {
    crate::platform::get_user_home(user)
}

/// Read a file if it exists, returning None if not found or on error
pub fn read_file_if_exists(path: &Path) -> Option<String> {
    if path.exists() {
        fs::read_to_string(path).ok()
    } else {
        None
    }
}

/// Search recursively for files matching a pattern
pub fn find_files_recursive(start_dir: &Path, filename: &str, max_depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    find_files_recursive_inner(start_dir, filename, max_depth, 0, &mut results);
    results
}

fn find_files_recursive_inner(
    dir: &Path,
    filename: &str,
    max_depth: usize,
    current_depth: usize,
    results: &mut Vec<PathBuf>,
) {
    if current_depth > max_depth {
        return;
    }

    if !dir.is_dir() {
        return;
    }

    // Check for the target file in current directory
    let target = dir.join(filename);
    if target.exists() && target.is_file() {
        results.push(target);
    }

    // Recurse into subdirectories
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip common directories that won't contain configs
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if !matches!(
                        name_str.as_ref(),
                        "node_modules"
                            | ".git"
                            | "target"
                            | "bin"
                            | "obj"
                            | "__pycache__"
                            | ".venv"
                            | "venv"
                            | "dist"
                            | "build"
                    ) {
                        find_files_recursive_inner(
                            &path,
                            filename,
                            max_depth,
                            current_depth + 1,
                            results,
                        );
                    }
                }
            }
        }
    }
}

/// Search for files matching multiple possible names
pub fn find_any_file(dir: &Path, filenames: &[&str]) -> Option<PathBuf> {
    for name in filenames {
        let path = dir.join(name);
        if path.exists() && path.is_file() {
            return Some(path);
        }
    }
    None
}

/// Search for directories containing agent configurations
pub fn find_config_dirs(user_home: &Path, dir_name: &str) -> Vec<PathBuf> {
    let mut results = Vec::new();

    // Check direct path under home
    let direct = user_home.join(dir_name);
    if direct.exists() && direct.is_dir() {
        results.push(direct);
    }

    // Also check in common development directories
    for subdir in &["source", "repos", "projects", "dev", "code", "workspace"] {
        let dev_dir = user_home.join(subdir);
        if dev_dir.exists() {
            // Search for .claude, .codex, .cursor, .gemini directories
            find_config_dirs_recursive(&dev_dir, dir_name, 3, 0, &mut results);
        }
    }

    results
}

fn find_config_dirs_recursive(
    dir: &Path,
    target_name: &str,
    max_depth: usize,
    current_depth: usize,
    results: &mut Vec<PathBuf>,
) {
    if current_depth > max_depth {
        return;
    }

    if !dir.is_dir() {
        return;
    }

    // Check if current directory matches
    if let Some(name) = dir.file_name()
        && name.to_string_lossy() == target_name
    {
        results.push(dir.to_path_buf());
    }

    // Check subdirectories
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Some(name) = path.file_name()
            {
                let name_str = name.to_string_lossy();
                // Skip common non-config directories
                if !matches!(
                    name_str.as_ref(),
                    "node_modules" | ".git" | "target" | "__pycache__"
                ) {
                    find_config_dirs_recursive(
                        &path,
                        target_name,
                        max_depth,
                        current_depth + 1,
                        results,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_users() {
        let users = get_all_users();
        let users_root = crate::platform::get_users_root();
        // Should find at least one user on any system with a users directory
        assert!(!users.is_empty() || !users_root.exists());
    }

    #[test]
    fn test_get_user_home() {
        let home = get_user_home("testuser");
        #[cfg(windows)]
        assert_eq!(home.to_string_lossy(), "C:\\Users\\testuser");
        #[cfg(target_os = "macos")]
        assert_eq!(home.to_string_lossy(), "/Users/testuser");
        #[cfg(all(unix, not(target_os = "macos")))]
        assert_eq!(home.to_string_lossy(), "/home/testuser");
    }
}
