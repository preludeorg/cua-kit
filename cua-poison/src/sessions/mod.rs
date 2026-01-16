//! Session enumeration for Claude Code

pub mod claude;

use crate::SessionInfo;

/// List available Claude Code sessions
pub fn list_sessions(user: Option<&str>) -> Vec<SessionInfo> {
    claude::enumerate_sessions(user)
}

/// Get the latest/most recent Claude Code session
pub fn get_latest_session(user: Option<&str>) -> Option<SessionInfo> {
    list_sessions(user).into_iter().next()
}
