//! Claude Code session enumeration from ~/.claude/history.jsonl

use crate::SessionInfo;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Enumerate sessions from Claude Code history file
pub fn enumerate_sessions(user: Option<&str>) -> Vec<SessionInfo> {
    let history_path = get_history_path(user);
    if !history_path.exists() {
        return vec![];
    }

    let mut sessions = Vec::new();
    let mut seen_session_ids = HashSet::new();

    // Parse JSONL file, extract unique sessionId values
    if let Ok(file) = File::open(&history_path) {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                // Look for sessionId field
                let session_id = json
                    .get("sessionId")
                    .or_else(|| json.get("session_id"))
                    .and_then(|v| v.as_str());

                if let Some(sid) = session_id
                    && !seen_session_ids.contains(sid)
                {
                    seen_session_ids.insert(sid.to_string());

                    // Extract timestamp
                    let timestamp = json
                        .get("timestamp")
                        .and_then(serde_json::Value::as_i64)
                        .map(|ts| {
                            // Convert Unix timestamp (milliseconds) to human-readable
                            use std::time::{Duration, UNIX_EPOCH};
                            #[allow(clippy::cast_sign_loss)]
                            let d = UNIX_EPOCH + Duration::from_millis(ts as u64);
                            format!("{d:?}")
                        });

                    // Extract project directory (Claude Code specific)
                    let project = json
                        .get("project")
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    sessions.push(SessionInfo {
                        id: sid.to_string(),
                        timestamp,
                        project,
                    });
                }
            }
        }
    }

    // Return in reverse order (most recent first based on insertion order)
    sessions.reverse();
    sessions
}

fn get_history_path(user: Option<&str>) -> PathBuf {
    let home = crate::platform::get_home_dir(user);
    home.join(".claude").join("history.jsonl")
}

fn get_home_dir(user: Option<&str>) -> PathBuf {
    crate::platform::get_home_dir(user)
}

/// Get the path to a Claude Code session file
/// Session files are stored in ~/.claude/projects/[encoded-path]/[session-uuid].jsonl
pub fn get_session_file_path(session_id: &str, user: Option<&str>) -> Option<PathBuf> {
    let home = get_home_dir(user);
    let projects_dir = home.join(".claude").join("projects");

    if !projects_dir.exists() {
        return None;
    }

    // Search all project directories for the session file
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                // Look for SESSION_ID.jsonl in this project directory
                let session_file = path.join(format!("{}.jsonl", session_id));
                if session_file.exists() {
                    return Some(session_file);
                }
            }
        }
    }

    None
}
