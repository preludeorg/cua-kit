//! Enumeration logic for no_std BOF

#![allow(unused_variables)]
#![allow(unused_assignments)]

extern crate alloc;

use super::beacon;
use super::fs;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Skip list for system user directories
const SKIP_USERS: &[&str] = &["Public", "Default", "Default User", "All Users"];

/// Directories to skip during recursive search
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "bin",
    "obj",
    "__pycache__",
    ".venv",
    "venv",
    "dist",
    "build",
];

/// Get all user directories, filtering out system accounts
fn get_all_users() -> Vec<String> {
    let dirs = fs::list_dirs("C:\\Users");
    dirs.into_iter()
        .filter(|name| !SKIP_USERS.contains(&name.as_str()))
        .collect()
}

/// Check if a directory name should be skipped
fn should_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

/// Main enumeration entry point
pub fn run_enumeration(json_output: bool, target_user: Option<&str>) {
    if json_output {
        run_json_enumeration(target_user);
    } else {
        run_plain_enumeration(target_user);
    }
}

fn run_plain_enumeration(target_user: Option<&str>) {
    beacon::output_line("=== CUA-Enum: Computer Use Agent Enumeration ===\n");

    let users = match target_user {
        Some(user) => alloc::vec![String::from(user)],
        None => get_all_users(),
    };

    let mut total_configs = 0;
    let mut total_agents_md: Vec<String> = Vec::new();

    for user in &users {
        // Claude Code
        if let Some(count) = enumerate_claude_code(user) {
            total_configs += count;
        }

        // Codex CLI
        if let Some(count) = enumerate_codex(user) {
            total_configs += count;
        }

        // Cursor IDE
        if let Some(count) = enumerate_cursor(user) {
            total_configs += count;
        }

        // Gemini CLI
        if let Some(count) = enumerate_gemini(user) {
            total_configs += count;
        }

        // Collect AGENTS.md files (don't print yet)
        let agents = find_all_agents_md(user);
        for path in agents {
            if !total_agents_md.contains(&path) {
                total_agents_md.push(path);
            }
        }
    }

    // Print AGENTS.md section
    if !total_agents_md.is_empty() {
        beacon::output_line("[*] AGENTS.md Files Found:");
        beacon::output_line("--------------------------");
        for path in &total_agents_md {
            beacon::output_line(&format!("  - {}", path));
        }
        beacon::output_line("");
    }

    beacon::output_line(&format!(
        "[*] Summary: {} agent configurations found",
        total_configs
    ));
    beacon::output_line(&format!(
        "    {} AGENTS.md files found",
        total_agents_md.len()
    ));
}

fn run_json_enumeration(target_user: Option<&str>) {
    let users = match target_user {
        Some(user) => alloc::vec![String::from(user)],
        None => get_all_users(),
    };

    beacon::output("{\"claude_code\":[");
    let mut first = true;
    for user in &users {
        if let Some(json) = get_claude_json(user) {
            if !first {
                beacon::output(",");
            }
            beacon::output(&json);
            first = false;
        }
    }
    beacon::output("],\"codex_cli\":[");

    first = true;
    for user in &users {
        if let Some(json) = get_codex_json(user) {
            if !first {
                beacon::output(",");
            }
            beacon::output(&json);
            first = false;
        }
    }
    beacon::output("],\"cursor\":[");

    first = true;
    for user in &users {
        if let Some(json) = get_cursor_json(user) {
            if !first {
                beacon::output(",");
            }
            beacon::output(&json);
            first = false;
        }
    }
    beacon::output("],\"gemini_cli\":[");

    first = true;
    for user in &users {
        if let Some(json) = get_gemini_json(user) {
            if !first {
                beacon::output(",");
            }
            beacon::output(&json);
            first = false;
        }
    }
    beacon::output("],\"agents_md\":[");

    first = true;
    for user in &users {
        for path in find_all_agents_md(user) {
            if !first {
                beacon::output(",");
            }
            beacon::output(&format!("\"{}\"", escape_json(&path)));
            first = false;
        }
    }
    beacon::output_line("]}");
}

// ============================================================================
// Claude Code Enumeration
// ============================================================================

fn enumerate_claude_code(user: &str) -> Option<usize> {
    let home = fs::get_user_home(user);
    let mut found = false;

    // Check global settings
    let global_settings = fs::join_path(&home, ".claude\\settings.json");
    let claude_json = fs::join_path(&home, ".claude.json");

    // Search for project settings from home directory (matching EXE behavior)
    let mut project_settings: Option<String> = None;
    let mut project_settings_content: Option<String> = None;

    if let Some((path, content)) = find_project_settings(&home, 4) {
        project_settings = Some(path);
        project_settings_content = Some(content);
    }

    // Search for CLAUDE.md files
    let claude_md_files = find_claude_md_files(user);

    // Search for skills files
    let skills_files = find_claude_skills(user);

    if fs::exists(&global_settings)
        || fs::exists(&claude_json)
        || project_settings.is_some()
        || !claude_md_files.is_empty()
        || !skills_files.is_empty()
    {
        beacon::output_line("[*] Claude Code Configurations:");
        beacon::output_line("--------------------------------");
        found = true;
        beacon::output_line(&format!("  User: {}", user));

        if fs::exists(&global_settings) {
            beacon::output_line(&format!("  Global Settings: {}", global_settings));
        }

        // Print project settings with Allowed/Denied
        if let Some(ref ps_path) = project_settings {
            beacon::output_line(&format!("  Project Settings: {}", ps_path));
            if let Some(ref content) = project_settings_content {
                print_settings_permissions(content);
            }
        }

        if fs::exists(&claude_json) {
            beacon::output_line(&format!("  Claude.json: {}", claude_json));
        }

        // Print CLAUDE.md files
        if !claude_md_files.is_empty() {
            beacon::output_line("  CLAUDE.md files:");
            for md_path in &claude_md_files {
                beacon::output_line(&format!("    - {}", md_path));
            }
        }

        // Print skills files
        if !skills_files.is_empty() {
            beacon::output_line("  Skills:");
            for skill_path in &skills_files {
                beacon::output_line(&format!("    - {}", skill_path));
            }
        }

        beacon::output_line("");
        return Some(1);
    }

    None
}

/// Find project settings.json file recursively
fn find_project_settings(base_dir: &str, max_depth: usize) -> Option<(String, String)> {
    find_project_settings_recursive(base_dir, max_depth, 0)
}

fn find_project_settings_recursive(
    dir: &str,
    max_depth: usize,
    depth: usize,
) -> Option<(String, String)> {
    if depth > max_depth || !fs::is_dir(dir) {
        return None;
    }

    // Check for .claude/settings.json in this directory
    let claude_dir = fs::join_path(dir, ".claude");
    if fs::is_dir(&claude_dir) {
        let settings_path = fs::join_path(&claude_dir, "settings.json");
        if fs::exists(&settings_path) {
            if let Some(content) = fs::read_to_string(&settings_path) {
                return Some((settings_path, content));
            }
        }
    }

    // Recurse into subdirectories
    for subdir in fs::list_dirs(dir) {
        if should_skip_dir(&subdir) {
            continue;
        }
        let subpath = fs::join_path(dir, &subdir);
        if let Some(result) = find_project_settings_recursive(&subpath, max_depth, depth + 1) {
            return Some(result);
        }
    }

    None
}

/// Parse and print Allowed/Denied permissions from settings JSON
fn print_settings_permissions(content: &str) {
    // Simple JSON parsing for permissions.allow and permissions.deny
    if let Some(allow) = extract_json_array(content, "allow") {
        beacon::output_line(&format!("    Allowed: {}", allow));
    }
    if let Some(deny) = extract_json_array(content, "deny") {
        beacon::output_line(&format!("    Denied: {}", deny));
    }
}

/// Extract a JSON array value by key (simple parser)
fn extract_json_array(json: &str, key: &str) -> Option<String> {
    // Look for "key": [ ... ]
    let search_pattern = format!("\"{}\"", key);
    let key_pos = json.find(&search_pattern)?;

    // Find the colon after the key
    let after_key = &json[key_pos + search_pattern.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = &after_key[colon_pos + 1..];

    // Skip whitespace and find opening bracket
    let trimmed = after_colon.trim_start();
    if !trimmed.starts_with('[') {
        return None;
    }

    // Find matching closing bracket
    let mut depth = 0;
    let mut end_pos = 0;
    for (i, c) in trimmed.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    end_pos = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if end_pos > 0 {
        Some(trimmed[..end_pos].to_string())
    } else {
        None
    }
}

/// Find CLAUDE.md files for a user
fn find_claude_md_files(user: &str) -> Vec<String> {
    let mut results = Vec::new();
    let home = fs::get_user_home(user);

    // Check global CLAUDE.md
    let global_claude_md = fs::join_path(&home, ".claude\\CLAUDE.md");
    if fs::exists(&global_claude_md) {
        results.push(global_claude_md);
    }

    // Search from home directory (matching EXE behavior)
    find_files_recursive(&home, "CLAUDE.md", 4, 0, &mut results);

    // Also search for CLAUDE.local.md
    find_files_recursive(&home, "CLAUDE.local.md", 4, 0, &mut results);

    results
}

/// Find Claude Code skills files in .claude/skills directories
fn find_claude_skills(user: &str) -> Vec<String> {
    let mut results = Vec::new();
    let home = fs::get_user_home(user);

    // Search from home directory for .claude/skills directories
    find_claude_skills_recursive(&home, 4, 0, &mut results);

    results
}

fn find_claude_skills_recursive(
    dir: &str,
    max_depth: usize,
    depth: usize,
    results: &mut Vec<String>,
) {
    if depth > max_depth || !fs::is_dir(dir) {
        return;
    }

    // Check for .claude/skills in this directory
    let claude_dir = fs::join_path(dir, ".claude");
    if fs::is_dir(&claude_dir) {
        let skills_dir = fs::join_path(&claude_dir, "skills");
        if fs::is_dir(&skills_dir) {
            // List all files in skills directory
            let files = fs::list_files(&skills_dir, "*");
            for file in files {
                let file_path = fs::join_path(&skills_dir, &file);
                if !results.contains(&file_path) {
                    results.push(file_path);
                }
            }
        }
    }

    // Recurse into subdirectories
    for subdir in fs::list_dirs(dir) {
        if should_skip_dir(&subdir) {
            continue;
        }
        let subpath = fs::join_path(dir, &subdir);
        find_claude_skills_recursive(&subpath, max_depth, depth + 1, results);
    }
}

/// Recursive file search helper
fn find_files_recursive(
    dir: &str,
    filename: &str,
    max_depth: usize,
    depth: usize,
    results: &mut Vec<String>,
) {
    if depth > max_depth || !fs::is_dir(dir) {
        return;
    }

    // Check for target file
    let target = fs::join_path(dir, filename);
    if fs::exists(&target) {
        if !results.contains(&target) {
            results.push(target);
        }
    }

    // Recurse into subdirectories
    for subdir in fs::list_dirs(dir) {
        if should_skip_dir(&subdir) {
            continue;
        }
        let subpath = fs::join_path(dir, &subdir);
        find_files_recursive(&subpath, filename, max_depth, depth + 1, results);
    }
}

fn get_claude_json(user: &str) -> Option<String> {
    let home = fs::get_user_home(user);
    let global_settings = fs::join_path(&home, ".claude\\settings.json");
    let claude_json = fs::join_path(&home, ".claude.json");
    let skills = find_claude_skills(user);

    if !fs::exists(&global_settings) && !fs::exists(&claude_json) && skills.is_empty() {
        return None;
    }

    let mut json = format!("{{\"user\":\"{}\",", escape_json(user));

    if fs::exists(&global_settings) {
        json.push_str(&format!(
            "\"global_settings\":{{\"path\":\"{}\"}},",
            escape_json(&global_settings)
        ));
    }
    if fs::exists(&claude_json) {
        json.push_str(&format!(
            "\"claude_json\":{{\"path\":\"{}\"}},",
            escape_json(&claude_json)
        ));
    }
    if !skills.is_empty() {
        json.push_str("\"skills_files\":[");
        let mut first = true;
        for skill in &skills {
            if !first {
                json.push(',');
            }
            json.push_str(&format!("\"{}\"", escape_json(skill)));
            first = false;
        }
        json.push_str("],");
    }

    // Remove trailing comma if present
    if json.ends_with(',') {
        json.pop();
    }
    json.push('}');

    Some(json)
}

// ============================================================================
// Codex CLI Enumeration
// ============================================================================

fn enumerate_codex(user: &str) -> Option<usize> {
    let home = fs::get_user_home(user);
    let config_path = fs::join_path(&home, ".codex\\config.toml");
    let codex_dir = fs::join_path(&home, ".codex");

    // Find skills
    let skills = find_codex_skills(user);

    if fs::exists(&config_path) || !skills.is_empty() {
        beacon::output_line("[*] OpenAI Codex CLI Configurations:");
        beacon::output_line("-------------------------------------");
        beacon::output_line(&format!("  User: {}", user));

        if fs::exists(&config_path) {
            beacon::output_line(&format!("  Config: {}", config_path));
        }

        // Print skills
        if !skills.is_empty() {
            beacon::output_line("  Skills:");
            for skill in &skills {
                beacon::output_line(&format!("    - {}", skill));
            }
        }

        beacon::output_line("");
        return Some(1);
    }

    None
}

/// Find SKILL.md files in .codex/skills directory
fn find_codex_skills(user: &str) -> Vec<String> {
    let mut results = Vec::new();
    let home = fs::get_user_home(user);
    let skills_dir = fs::join_path(&home, ".codex\\skills");

    if fs::is_dir(&skills_dir) {
        find_files_recursive(&skills_dir, "SKILL.md", 5, 0, &mut results);
    }

    results
}

fn get_codex_json(user: &str) -> Option<String> {
    let home = fs::get_user_home(user);
    let config_path = fs::join_path(&home, ".codex\\config.toml");

    if !fs::exists(&config_path) {
        return None;
    }

    Some(format!(
        "{{\"user\":\"{}\",\"config_toml\":{{\"path\":\"{}\"}}}}",
        escape_json(user),
        escape_json(&config_path)
    ))
}

// ============================================================================
// Cursor IDE Enumeration
// ============================================================================

fn enumerate_cursor(user: &str) -> Option<usize> {
    let home = fs::get_user_home(user);

    // Find .cursor/rules files in project directories
    let rules_files = find_cursor_rules(user);

    if !rules_files.is_empty() {
        beacon::output_line("[*] Cursor IDE Configurations:");
        beacon::output_line("------------------------------");
        beacon::output_line(&format!("  User: {}", user));

        beacon::output_line("  Rules files:");
        for rule in &rules_files {
            beacon::output_line(&format!("    - {}", rule));
        }

        beacon::output_line("");
        return Some(1);
    }

    None
}

/// Find Cursor rules files (.mdc files in .cursor/rules directories)
fn find_cursor_rules(user: &str) -> Vec<String> {
    let mut results = Vec::new();
    let home = fs::get_user_home(user);

    // Search from home directory (matching EXE behavior)
    find_cursor_rules_recursive(&home, 4, 0, &mut results);

    results
}

fn find_cursor_rules_recursive(
    dir: &str,
    max_depth: usize,
    depth: usize,
    results: &mut Vec<String>,
) {
    if depth > max_depth || !fs::is_dir(dir) {
        return;
    }

    // Check for .cursor/rules in this directory
    let cursor_dir = fs::join_path(dir, ".cursor");
    if fs::is_dir(&cursor_dir) {
        let rules_dir = fs::join_path(&cursor_dir, "rules");
        if fs::is_dir(&rules_dir) {
            // List all files in rules directory (typically .mdc files)
            let files = fs::list_files(&rules_dir, "*");
            for file in files {
                let file_path = fs::join_path(&rules_dir, &file);
                if !results.contains(&file_path) {
                    results.push(file_path);
                }
            }
        }
    }

    // Recurse into subdirectories
    for subdir in fs::list_dirs(dir) {
        if should_skip_dir(&subdir) {
            continue;
        }
        let subpath = fs::join_path(dir, &subdir);
        find_cursor_rules_recursive(&subpath, max_depth, depth + 1, results);
    }
}

fn get_cursor_json(user: &str) -> Option<String> {
    let home = fs::get_user_home(user);
    let rules = find_cursor_rules(user);

    if rules.is_empty() {
        return None;
    }

    let mut json = format!("{{\"user\":\"{}\",\"rules_files\":[", escape_json(user));
    let mut first = true;
    for rule in &rules {
        if !first {
            json.push(',');
        }
        json.push_str(&format!("\"{}\"", escape_json(rule)));
        first = false;
    }
    json.push_str("]}");

    Some(json)
}

// ============================================================================
// Gemini CLI Enumeration
// ============================================================================

fn enumerate_gemini(user: &str) -> Option<usize> {
    let home = fs::get_user_home(user);
    let gemini_path = fs::join_path(&home, ".gemini");

    if fs::is_dir(&gemini_path) {
        beacon::output_line("[*] Gemini CLI Configurations:");
        beacon::output_line("------------------------------");
        beacon::output_line(&format!("  User: {}", user));
        beacon::output_line(&format!("  Config Dir: {}", gemini_path));
        beacon::output_line("");
        return Some(1);
    }

    None
}

fn get_gemini_json(user: &str) -> Option<String> {
    let home = fs::get_user_home(user);
    let gemini_path = fs::join_path(&home, ".gemini");

    if !fs::is_dir(&gemini_path) {
        return None;
    }

    Some(format!(
        "{{\"user\":\"{}\",\"config_dir\":\"{}\"}}",
        escape_json(user),
        escape_json(&gemini_path)
    ))
}

// ============================================================================
// AGENTS.md Enumeration
// ============================================================================

/// Find all AGENTS.md files for a user (deep search including .cargo)
fn find_all_agents_md(user: &str) -> Vec<String> {
    let mut results = Vec::new();
    let home = fs::get_user_home(user);

    // Search from home directory (matching EXE behavior)
    find_files_recursive(&home, "AGENTS.md", 5, 0, &mut results);

    // Also search for AGENTS.override.md
    find_files_recursive(&home, "AGENTS.override.md", 5, 0, &mut results);

    // Search in .cargo/registry (important for finding AGENTS.md in crates)
    let cargo_registry = fs::join_path(&home, ".cargo\\registry\\src");
    if fs::is_dir(&cargo_registry) {
        // Search inside registry source directories
        for index_dir in fs::list_dirs(&cargo_registry) {
            let index_path = fs::join_path(&cargo_registry, &index_dir);
            if fs::is_dir(&index_path) {
                // Search crate directories
                for crate_dir in fs::list_dirs(&index_path) {
                    let crate_path = fs::join_path(&index_path, &crate_dir);
                    let agents_path = fs::join_path(&crate_path, "AGENTS.md");
                    if fs::exists(&agents_path) {
                        if !results.contains(&agents_path) {
                            results.push(agents_path);
                        }
                    }
                }
            }
        }
    }

    // Also search for test mock configs
    let tests_dir = fs::join_path(&home, "source\\repos");
    if fs::is_dir(&tests_dir) {
        find_agents_in_tests(&tests_dir, 4, 0, &mut results);
    }

    results
}

/// Search for AGENTS.md in test directories
fn find_agents_in_tests(dir: &str, max_depth: usize, depth: usize, results: &mut Vec<String>) {
    if depth > max_depth || !fs::is_dir(dir) {
        return;
    }

    // Check for tests/mock_configs/AGENTS.md pattern
    let tests_dir = fs::join_path(dir, "tests");
    if fs::is_dir(&tests_dir) {
        let mock_configs = fs::join_path(&tests_dir, "mock_configs");
        if fs::is_dir(&mock_configs) {
            let agents_path = fs::join_path(&mock_configs, "AGENTS.md");
            if fs::exists(&agents_path) {
                if !results.contains(&agents_path) {
                    results.push(agents_path);
                }
            }
        }
    }

    // Also check direct AGENTS.md
    let agents_path = fs::join_path(dir, "AGENTS.md");
    if fs::exists(&agents_path) {
        if !results.contains(&agents_path) {
            results.push(agents_path);
        }
    }

    // Recurse
    for subdir in fs::list_dirs(dir) {
        if should_skip_dir(&subdir) {
            continue;
        }
        let subpath = fs::join_path(dir, &subdir);
        find_agents_in_tests(&subpath, max_depth, depth + 1, results);
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}
