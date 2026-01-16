//! Output formatting module
//!
//! Handles both plaintext and JSON output formats for enumeration results.

use crate::beacon;
use crate::enumeration::EnumerationResults;

/// Output format selection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Plain,
    Json,
}

/// Print enumeration results in the specified format
pub fn print_results(results: &EnumerationResults, format: OutputFormat) {
    match format {
        OutputFormat::Plain => print_plain(results),
        OutputFormat::Json => print_json(results),
    }
}

fn print_plain(results: &EnumerationResults) {
    beacon::output_line("=== CUA-Enum: Computer Use Agent Enumeration ===\n");

    // Claude Code results
    if !results.claude_code.is_empty() {
        beacon::output_line("[*] Claude Code Configurations:");
        beacon::output_line("--------------------------------");
        for config in &results.claude_code {
            beacon::output_line(&format!("  User: {}", config.user));

            if let Some(ref settings) = config.global_settings {
                beacon::output_line(&format!("  Global Settings: {}", settings.path));
                print_settings_summary(&settings.content);
            }

            if let Some(ref settings) = config.project_settings {
                beacon::output_line(&format!("  Project Settings: {}", settings.path));
                print_settings_summary(&settings.content);
            }

            if let Some(ref mcp) = config.mcp_config {
                beacon::output_line(&format!("  MCP Config: {}", mcp.path));
                print_mcp_servers(&mcp.content);
            }

            if let Some(ref claude_json) = config.claude_json {
                beacon::output_line(&format!("  Claude.json: {}", claude_json.path));
            }

            if !config.claude_md_files.is_empty() {
                beacon::output_line("  CLAUDE.md files:");
                for md in &config.claude_md_files {
                    beacon::output_line(&format!("    - {}", md));
                }
            }

            beacon::output_line("");
        }
    }

    // Codex CLI results
    if !results.codex_cli.is_empty() {
        beacon::output_line("[*] OpenAI Codex CLI Configurations:");
        beacon::output_line("-------------------------------------");
        for config in &results.codex_cli {
            beacon::output_line(&format!("  User: {}", config.user));

            if let Some(ref settings) = config.config_toml {
                beacon::output_line(&format!("  Config: {}", settings.path));
            }

            if !config.agents_md_files.is_empty() {
                beacon::output_line("  AGENTS.md files:");
                for md in &config.agents_md_files {
                    beacon::output_line(&format!("    - {}", md));
                }
            }

            if !config.skills.is_empty() {
                beacon::output_line("  Skills:");
                for skill in &config.skills {
                    beacon::output_line(&format!("    - {}", skill));
                }
            }

            beacon::output_line("");
        }
    }

    // Cursor results
    if !results.cursor.is_empty() {
        beacon::output_line("[*] Cursor IDE Configurations:");
        beacon::output_line("------------------------------");
        for config in &results.cursor {
            beacon::output_line(&format!("  User: {}", config.user));

            if let Some(ref db_path) = config.state_db_path {
                beacon::output_line(&format!("  State DB: {}", db_path));
            }

            if !config.rules_files.is_empty() {
                beacon::output_line("  Rules files:");
                for rule in &config.rules_files {
                    beacon::output_line(&format!("    - {}", rule));
                }
            }

            if let Some(ref env) = config.environment_json {
                beacon::output_line(&format!("  Environment: {}", env.path));
            }

            beacon::output_line("");
        }
    }

    // Gemini CLI results
    if !results.gemini_cli.is_empty() {
        beacon::output_line("[*] Gemini CLI Configurations:");
        beacon::output_line("------------------------------");
        for config in &results.gemini_cli {
            beacon::output_line(&format!("  User: {}", config.user));

            if let Some(ref settings) = config.global_settings {
                beacon::output_line(&format!("  Global Settings: {}", settings.path));
            }

            if let Some(ref settings) = config.project_settings {
                beacon::output_line(&format!("  Project Settings: {}", settings.path));
            }

            if !config.extensions.is_empty() {
                beacon::output_line("  Extensions:");
                for ext in &config.extensions {
                    beacon::output_line(&format!("    - {}", ext));
                }
            }

            if !config.commands.is_empty() {
                beacon::output_line("  Custom Commands:");
                for cmd in &config.commands {
                    beacon::output_line(&format!("    - {}", cmd));
                }
            }

            beacon::output_line("");
        }
    }

    // AGENTS.md results
    if !results.agents_md.is_empty() {
        beacon::output_line("[*] AGENTS.md Files Found:");
        beacon::output_line("--------------------------");
        for path in &results.agents_md {
            beacon::output_line(&format!("  - {}", path));
        }
        beacon::output_line("");
    }

    // Summary
    let total = results.claude_code.len()
        + results.codex_cli.len()
        + results.cursor.len()
        + results.gemini_cli.len();

    beacon::output_line(&format!(
        "[*] Summary: {} agent configurations found",
        total
    ));
    beacon::output_line(&format!(
        "    {} AGENTS.md files found",
        results.agents_md.len()
    ));
}

fn print_settings_summary(content: &str) {
    // Parse JSON and show key security-relevant settings
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(permissions) = json.get("permissions") {
            if let Some(allow) = permissions.get("allow") {
                beacon::output_line(&format!("    Allowed: {}", allow));
            }
            if let Some(deny) = permissions.get("deny") {
                beacon::output_line(&format!("    Denied: {}", deny));
            }
        }
        if let Some(env) = json.get("env") {
            beacon::output_line(&format!("    Environment vars: {}", env));
        }
    }
}

fn print_mcp_servers(content: &str) {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content)
        && let Some(servers) = json.get("mcpServers")
        && let Some(obj) = servers.as_object()
    {
        for (name, config) in obj {
            let server_type = config
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            beacon::output_line(&format!("    - {} ({})", name, server_type));

            // Check for sensitive data
            if let Some(env) = config.get("env")
                && let Some(env_obj) = env.as_object()
            {
                for key in env_obj.keys() {
                    if key.to_lowercase().contains("key")
                        || key.to_lowercase().contains("token")
                        || key.to_lowercase().contains("secret")
                    {
                        beacon::output_line(&format!("      [!] Sensitive env var: {}", key));
                    }
                }
            }
        }
    }
}

fn print_json(results: &EnumerationResults) {
    match serde_json::to_string_pretty(results) {
        Ok(json) => beacon::output_line(&json),
        Err(e) => beacon::output_error(&format!("Failed to serialize results: {}", e)),
    }
}
