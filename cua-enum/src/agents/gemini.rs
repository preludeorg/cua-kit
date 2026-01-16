//! Gemini CLI enumeration module
//!
//! Enumerates Google Gemini CLI configuration files:
//! - ~/.gemini/settings.json (global settings)
//! - .gemini/settings.json (project settings)
//! - ~/.gemini/extensions/ (extensions)
//! - ~/.gemini/commands/ (custom commands)
//! - .env files (API keys and environment)

use crate::enumeration::{
    ConfigFile, find_config_dirs, find_files_recursive, get_user_home, read_file_if_exists,
};
use serde::Serialize;
use std::fs;

/// Gemini CLI configuration for a user
#[derive(Debug, Serialize)]
pub struct GeminiConfig {
    pub user: String,
    pub global_settings: Option<ConfigFile>,
    pub project_settings: Option<ConfigFile>,
    pub extensions: Vec<String>,
    pub commands: Vec<String>,
    pub env_files: Vec<String>,
    pub mcp_servers: Vec<GeminiMcpServer>,
}

/// MCP server configuration from Gemini settings
#[derive(Debug, Serialize)]
pub struct GeminiMcpServer {
    pub name: String,
    pub config: String,
}

impl GeminiConfig {
    fn new(user: &str) -> Self {
        Self {
            user: user.to_string(),
            global_settings: None,
            project_settings: None,
            extensions: Vec::new(),
            commands: Vec::new(),
            env_files: Vec::new(),
            mcp_servers: Vec::new(),
        }
    }

    fn has_any_config(&self) -> bool {
        self.global_settings.is_some()
            || self.project_settings.is_some()
            || !self.extensions.is_empty()
            || !self.commands.is_empty()
            || !self.env_files.is_empty()
    }
}

/// Enumerate Gemini CLI configurations for a user
pub fn enumerate_gemini(user: &str) -> Option<GeminiConfig> {
    let mut config = GeminiConfig::new(user);
    let user_home = get_user_home(user);
    let gemini_home = user_home.join(".gemini");

    // Check for global settings.json
    let global_settings_path = gemini_home.join("settings.json");
    if let Some(content) = read_file_if_exists(&global_settings_path) {
        // Parse MCP servers from settings
        config.mcp_servers = parse_gemini_mcp_servers(&content);
        config.global_settings = Some(ConfigFile::new(
            &global_settings_path.to_string_lossy(),
            content,
        ));
    }

    // Check for extensions directory
    let extensions_dir = gemini_home.join("extensions");
    if extensions_dir.exists()
        && extensions_dir.is_dir()
        && let Ok(entries) = fs::read_dir(&extensions_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                config.extensions.push(path.to_string_lossy().to_string());
            }
        }
    }

    // Check for commands directory
    let commands_dir = gemini_home.join("commands");
    if commands_dir.exists()
        && commands_dir.is_dir()
        && let Ok(entries) = fs::read_dir(&commands_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                config.commands.push(path.to_string_lossy().to_string());
            }
        }
    }

    // Check for global .env file
    let global_env = user_home.join(".env");
    if global_env.exists() {
        config
            .env_files
            .push(global_env.to_string_lossy().to_string());
    }

    // Search for project-level .gemini directories
    let gemini_dirs = find_config_dirs(&user_home, ".gemini");
    for dir in gemini_dirs {
        // Skip the global gemini home
        if dir == gemini_home {
            continue;
        }

        // Project settings.json
        let project_settings = dir.join("settings.json");
        if config.project_settings.is_none()
            && let Some(content) = read_file_if_exists(&project_settings)
        {
            config.project_settings = Some(ConfigFile::new(
                &project_settings.to_string_lossy(),
                content,
            ));
        }

        // Project extensions
        let project_extensions = dir.join("extensions");
        if project_extensions.exists()
            && project_extensions.is_dir()
            && let Ok(entries) = fs::read_dir(&project_extensions)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let path_str = path.to_string_lossy().to_string();
                    if !config.extensions.contains(&path_str) {
                        config.extensions.push(path_str);
                    }
                }
            }
        }

        // Project commands
        let project_commands = dir.join("commands");
        if project_commands.exists()
            && project_commands.is_dir()
            && let Ok(entries) = fs::read_dir(&project_commands)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let path_str = path.to_string_lossy().to_string();
                    if !config.commands.contains(&path_str) {
                        config.commands.push(path_str);
                    }
                }
            }
        }
    }

    // Search for .env files in project directories
    let env_files = find_files_recursive(&user_home, ".env", 3);
    for env_path in env_files {
        let path_str = env_path.to_string_lossy().to_string();
        if !config.env_files.contains(&path_str) {
            config.env_files.push(path_str);
        }
    }

    if config.has_any_config() {
        Some(config)
    } else {
        None
    }
}

/// Parse MCP server configurations from Gemini settings.json
fn parse_gemini_mcp_servers(content: &str) -> Vec<GeminiMcpServer> {
    let mut servers = Vec::new();

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        // Check for mcpServers key
        if let Some(mcp_servers) = json.get("mcpServers")
            && let Some(obj) = mcp_servers.as_object()
        {
            for (name, config) in obj {
                servers.push(GeminiMcpServer {
                    name: name.clone(),
                    config: config.to_string(),
                });
            }
        }

        // Also check for tools.mcpServers (alternative location)
        if let Some(tools) = json.get("tools")
            && let Some(mcp_servers) = tools.get("mcpServers")
            && let Some(obj) = mcp_servers.as_object()
        {
            for (name, config) in obj {
                // Avoid duplicates
                if !servers.iter().any(|s| s.name == *name) {
                    servers.push(GeminiMcpServer {
                        name: name.clone(),
                        config: config.to_string(),
                    });
                }
            }
        }
    }

    servers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_config_creation() {
        let config = GeminiConfig::new("testuser");
        assert_eq!(config.user, "testuser");
        assert!(!config.has_any_config());
    }

    #[test]
    fn test_parse_gemini_mcp_servers() {
        let content = r#"{
            "mcpServers": {
                "filesystem": {
                    "command": "npx",
                    "args": ["-y", "@anthropic/mcp-filesystem"]
                }
            }
        }"#;

        let servers = parse_gemini_mcp_servers(content);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "filesystem");
    }

    #[test]
    fn test_enumerate_gemini() {
        let result = enumerate_gemini("testuser");
        // Result may or may not exist depending on system state
        let _ = result;
    }
}
