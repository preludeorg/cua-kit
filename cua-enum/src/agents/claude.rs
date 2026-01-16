//! Claude Code enumeration module
//!
//! Enumerates Anthropic's Claude Code CLI configuration files:
//! - ~/.claude/settings.json (user settings)
//! - .claude/settings.json (project settings)
//! - .claude/settings.local.json (local project settings)
//! - ~/.claude.json (preferences, OAuth, MCP servers)
//! - .mcp.json (project MCP configuration)
//! - CLAUDE.md files (instructions/context)
//! - ~/.claude/agents/ (subagent configurations)
//! - .claude/skills/ (skill definitions)

use crate::enumeration::{
    ConfigFile, find_config_dirs, find_files_recursive, get_user_home, read_file_if_exists,
};
use serde::Serialize;

/// Claude Code configuration for a user
#[derive(Debug, Serialize)]
pub struct ClaudeCodeConfig {
    pub user: String,
    pub global_settings: Option<ConfigFile>,
    pub project_settings: Option<ConfigFile>,
    pub local_settings: Option<ConfigFile>,
    pub claude_json: Option<ConfigFile>,
    pub mcp_config: Option<ConfigFile>,
    pub claude_md_files: Vec<String>,
    pub agents_dir: Option<String>,
    pub managed_settings: Option<ConfigFile>,
    pub managed_mcp: Option<ConfigFile>,
    pub skills_files: Vec<String>,
}

impl ClaudeCodeConfig {
    fn new(user: &str) -> Self {
        Self {
            user: user.to_string(),
            global_settings: None,
            project_settings: None,
            local_settings: None,
            claude_json: None,
            mcp_config: None,
            claude_md_files: Vec::new(),
            agents_dir: None,
            managed_settings: None,
            managed_mcp: None,
            skills_files: Vec::new(),
        }
    }

    fn has_any_config(&self) -> bool {
        self.global_settings.is_some()
            || self.project_settings.is_some()
            || self.local_settings.is_some()
            || self.claude_json.is_some()
            || self.mcp_config.is_some()
            || !self.claude_md_files.is_empty()
            || self.agents_dir.is_some()
            || self.managed_settings.is_some()
            || self.managed_mcp.is_some()
            || !self.skills_files.is_empty()
    }
}

/// Enumerate Claude Code configurations for a user
pub fn enumerate_claude_code(user: &str) -> Option<ClaudeCodeConfig> {
    let mut config = ClaudeCodeConfig::new(user);
    let user_home = get_user_home(user);

    // Check for global settings: ~/.claude/settings.json
    let global_settings_path = user_home.join(".claude").join("settings.json");
    if let Some(content) = read_file_if_exists(&global_settings_path) {
        config.global_settings = Some(ConfigFile::new(
            &global_settings_path.to_string_lossy(),
            content,
        ));
    }

    // Check for ~/.claude.json (preferences, OAuth, MCP)
    let claude_json_path = user_home.join(".claude.json");
    if let Some(content) = read_file_if_exists(&claude_json_path) {
        config.claude_json = Some(ConfigFile::new(
            &claude_json_path.to_string_lossy(),
            content,
        ));
    }

    // Check for agents directory
    let agents_dir = user_home.join(".claude").join("agents");
    if agents_dir.exists() && agents_dir.is_dir() {
        config.agents_dir = Some(agents_dir.to_string_lossy().to_string());
    }

    // Check for global CLAUDE.md
    let global_claude_md = user_home.join(".claude").join("CLAUDE.md");
    if global_claude_md.exists() {
        config
            .claude_md_files
            .push(global_claude_md.to_string_lossy().to_string());
    }

    // Check managed settings location (platform-specific)
    let managed_dir = crate::platform::get_managed_settings_dir();
    let managed_settings_path = managed_dir.join("managed-settings.json");
    if let Some(content) = read_file_if_exists(&managed_settings_path) {
        config.managed_settings = Some(ConfigFile::new(
            &managed_settings_path.to_string_lossy(),
            content,
        ));
    }

    // Check managed MCP location (platform-specific)
    let managed_mcp_path = managed_dir.join("managed-mcp.json");
    if let Some(content) = read_file_if_exists(&managed_mcp_path) {
        config.managed_mcp = Some(ConfigFile::new(
            &managed_mcp_path.to_string_lossy(),
            content,
        ));
    }

    // Search for project-level configurations in common directories
    let claude_dirs = find_config_dirs(&user_home, ".claude");
    for dir in claude_dirs {
        // Project settings.json
        let project_settings = dir.join("settings.json");
        if config.project_settings.is_none()
            && let Some(content) = read_file_if_exists(&project_settings)
        {
            // Skip if it's the global settings we already found
            if project_settings != global_settings_path {
                config.project_settings = Some(ConfigFile::new(
                    &project_settings.to_string_lossy(),
                    content,
                ));
            }
        }

        // Local settings.local.json
        let local_settings = dir.join("settings.local.json");
        if config.local_settings.is_none()
            && let Some(content) = read_file_if_exists(&local_settings)
        {
            config.local_settings =
                Some(ConfigFile::new(&local_settings.to_string_lossy(), content));
        }

        // CLAUDE.md in project .claude directory
        let project_claude_md = dir.join("CLAUDE.md");
        if project_claude_md.exists() {
            let path_str = project_claude_md.to_string_lossy().to_string();
            if !config.claude_md_files.contains(&path_str) {
                config.claude_md_files.push(path_str);
            }
        }

        // Skills in project .claude/skills directory
        let skills_dir = dir.join("skills");
        if skills_dir.exists()
            && skills_dir.is_dir()
            && let Ok(entries) = std::fs::read_dir(&skills_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let path_str = path.to_string_lossy().to_string();
                    if !config.skills_files.contains(&path_str) {
                        config.skills_files.push(path_str);
                    }
                }
            }
        }
    }

    // Search for .mcp.json files in project directories
    let mcp_files = find_files_recursive(&user_home, ".mcp.json", 4);
    for mcp_path in mcp_files {
        if config.mcp_config.is_none()
            && let Some(content) = read_file_if_exists(&mcp_path)
        {
            config.mcp_config = Some(ConfigFile::new(&mcp_path.to_string_lossy(), content));
        }
    }

    // Search for CLAUDE.md files in project roots
    let claude_md_files = find_files_recursive(&user_home, "CLAUDE.md", 4);
    for md_path in claude_md_files {
        let path_str = md_path.to_string_lossy().to_string();
        if !config.claude_md_files.contains(&path_str) {
            config.claude_md_files.push(path_str);
        }
    }

    // Also search for CLAUDE.local.md
    let claude_local_md_files = find_files_recursive(&user_home, "CLAUDE.local.md", 4);
    for md_path in claude_local_md_files {
        let path_str = md_path.to_string_lossy().to_string();
        if !config.claude_md_files.contains(&path_str) {
            config.claude_md_files.push(path_str);
        }
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
    fn test_claude_config_creation() {
        let config = ClaudeCodeConfig::new("testuser");
        assert_eq!(config.user, "testuser");
        assert!(config.global_settings.is_none());
        assert!(!config.has_any_config());
    }
}
