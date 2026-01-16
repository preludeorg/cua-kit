//! OpenAI Codex CLI enumeration module
//!
//! Enumerates OpenAI Codex CLI configuration files:
//! - ~/.codex/config.toml (main configuration)
//! - ~/.codex/AGENTS.md or AGENTS.override.md (global instructions)
//! - ~/.codex/skills/**/SKILL.md (skills)
//! - ~/.codex/rules/ (execution policy rules)
//! - ~/.codex/history.jsonl (command history)
//! - Project AGENTS.md files

use crate::enumeration::{ConfigFile, find_files_recursive, get_user_home, read_file_if_exists};
use serde::Serialize;
use std::fs;

/// Codex CLI configuration for a user
#[derive(Debug, Serialize)]
pub struct CodexConfig {
    pub user: String,
    pub config_toml: Option<ConfigFile>,
    pub global_agents_md: Option<ConfigFile>,
    pub agents_override_md: Option<ConfigFile>,
    pub agents_md_files: Vec<String>,
    pub skills: Vec<String>,
    pub rules_files: Vec<String>,
    pub history_file: Option<String>,
    pub mcp_servers: Vec<McpServerConfig>,
}

/// MCP server configuration from config.toml
#[derive(Debug, Serialize)]
pub struct McpServerConfig {
    pub name: String,
    pub server_type: String,
    pub command: Option<String>,
    pub url: Option<String>,
}

impl CodexConfig {
    fn new(user: &str) -> Self {
        Self {
            user: user.to_string(),
            config_toml: None,
            global_agents_md: None,
            agents_override_md: None,
            agents_md_files: Vec::new(),
            skills: Vec::new(),
            rules_files: Vec::new(),
            history_file: None,
            mcp_servers: Vec::new(),
        }
    }

    fn has_any_config(&self) -> bool {
        self.config_toml.is_some()
            || self.global_agents_md.is_some()
            || self.agents_override_md.is_some()
            || !self.agents_md_files.is_empty()
            || !self.skills.is_empty()
            || !self.rules_files.is_empty()
            || self.history_file.is_some()
    }
}

/// Enumerate Codex CLI configurations for a user
pub fn enumerate_codex(user: &str) -> Option<CodexConfig> {
    let mut config = CodexConfig::new(user);
    let user_home = get_user_home(user);
    let codex_home = user_home.join(".codex");

    // Check for config.toml
    let config_toml_path = codex_home.join("config.toml");
    if let Some(content) = read_file_if_exists(&config_toml_path) {
        // Parse MCP servers from config
        config.mcp_servers = parse_mcp_servers(&content);
        config.config_toml = Some(ConfigFile::new(
            &config_toml_path.to_string_lossy(),
            content,
        ));
    }

    // Check for AGENTS.md in codex home
    let agents_md_path = codex_home.join("AGENTS.md");
    if let Some(content) = read_file_if_exists(&agents_md_path) {
        config.global_agents_md = Some(ConfigFile::new(&agents_md_path.to_string_lossy(), content));
    }

    // Check for AGENTS.override.md (takes precedence)
    let agents_override_path = codex_home.join("AGENTS.override.md");
    if let Some(content) = read_file_if_exists(&agents_override_path) {
        config.agents_override_md = Some(ConfigFile::new(
            &agents_override_path.to_string_lossy(),
            content,
        ));
    }

    // Search for skills in ~/.codex/skills/**/SKILL.md
    let skills_dir = codex_home.join("skills");
    if skills_dir.exists() {
        let skill_files = find_files_recursive(&skills_dir, "SKILL.md", 5);
        for skill_path in skill_files {
            config.skills.push(skill_path.to_string_lossy().to_string());
        }
    }

    // Search for rules in ~/.codex/rules/
    let rules_dir = codex_home.join("rules");
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

    // Check for history file
    let history_path = codex_home.join("history.jsonl");
    if history_path.exists() {
        config.history_file = Some(history_path.to_string_lossy().to_string());
    }

    // Search for project AGENTS.md files
    let agents_files = find_files_recursive(&user_home, "AGENTS.md", 4);
    for agents_path in agents_files {
        let path_str = agents_path.to_string_lossy().to_string();
        // Don't include the global ones we already found
        if !path_str.contains(".codex") {
            config.agents_md_files.push(path_str);
        }
    }

    if config.has_any_config() {
        Some(config)
    } else {
        None
    }
}

/// Parse MCP server configurations from config.toml content
fn parse_mcp_servers(content: &str) -> Vec<McpServerConfig> {
    let mut servers = Vec::new();

    // Simple TOML parsing for [mcp_servers.NAME] sections
    let mut current_server: Option<String> = None;
    let mut current_type: Option<String> = None;
    let mut current_command: Option<String> = None;
    let mut current_url: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        // Check for section header like [mcp_servers.myserver]
        if line.starts_with("[mcp_servers.") && line.ends_with(']') {
            // Save previous server if exists
            if let Some(name) = current_server.take() {
                servers.push(McpServerConfig {
                    name,
                    server_type: current_type.take().unwrap_or_else(|| "unknown".to_string()),
                    command: current_command.take(),
                    url: current_url.take(),
                });
            }

            // Extract server name
            let name = line
                .trim_start_matches("[mcp_servers.")
                .trim_end_matches(']');
            current_server = Some(name.to_string());
        } else if current_server.is_some() {
            // Parse key-value pairs
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "type" => current_type = Some(value.to_string()),
                    "command" => current_command = Some(value.to_string()),
                    "url" => current_url = Some(value.to_string()),
                    _ => {}
                }
            }
        }
    }

    // Don't forget the last server
    if let Some(name) = current_server {
        servers.push(McpServerConfig {
            name,
            server_type: current_type.unwrap_or_else(|| "unknown".to_string()),
            command: current_command,
            url: current_url,
        });
    }

    servers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_config_creation() {
        let config = CodexConfig::new("testuser");
        assert_eq!(config.user, "testuser");
        assert!(!config.has_any_config());
    }

    #[test]
    fn test_parse_mcp_servers() {
        let content = r#"
[mcp_servers.github]
type = "stdio"
command = "/usr/local/bin/github-mcp"

[mcp_servers.api]
type = "http"
url = "https://api.example.com/mcp"
"#;

        let servers = parse_mcp_servers(content);
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].name, "github");
        assert_eq!(servers[0].server_type, "stdio");
        assert!(servers[0].command.is_some());
        assert_eq!(servers[1].name, "api");
        assert_eq!(servers[1].server_type, "http");
        assert!(servers[1].url.is_some());
    }

    #[test]
    fn test_enumerate_codex() {
        let result = enumerate_codex("testuser");
        // Result may or may not exist depending on system state
        // Just verify no panic occurs
        let _ = result;
    }
}
