//! AGENTS.md enumeration module
//!
//! Searches for AGENTS.md files across the filesystem.
//! These files are used by various AI agents (Codex CLI, etc.) to provide
//! project-specific instructions and context.

use crate::enumeration::{find_files_recursive, get_user_home};
use std::path::PathBuf;

/// Find all AGENTS.md files for a user
pub fn find_agents_md_files(user: &str) -> Vec<String> {
    let mut results = Vec::new();
    let user_home = get_user_home(user);

    // Search for AGENTS.md files
    let agents_files = find_files_recursive(&user_home, "AGENTS.md", 5);
    for path in agents_files {
        results.push(path.to_string_lossy().to_string());
    }

    // Also search for AGENTS.override.md
    let override_files = find_files_recursive(&user_home, "AGENTS.override.md", 5);
    for path in override_files {
        let path_str = path.to_string_lossy().to_string();
        if !results.contains(&path_str) {
            results.push(path_str);
        }
    }

    results
}

/// Check if a specific AGENTS.md file exists and return its path
pub fn check_agents_md(path: &str) -> Option<String> {
    let path = PathBuf::from(path);
    if path.exists() && path.is_file() {
        Some(path.to_string_lossy().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_agents_md_nonexistent() {
        let result = check_agents_md("C:\\nonexistent\\path\\AGENTS.md");
        assert!(result.is_none());
    }
}
