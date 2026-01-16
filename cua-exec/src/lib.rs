//! CUA-Exec: Computer Use Agent Execution
//!
//! Execute Claude commands on Windows systems via CLI wrapper or direct API.
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
pub mod api;
#[cfg(not(feature = "bof"))]
pub mod execution;

// ClaudeResult is only used in EXE mode
#[cfg(not(feature = "bof"))]
use serde::{Deserialize, Serialize};

/// Result from Claude execution
#[cfg(not(feature = "bof"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeResult {
    pub session_id: String,
    pub result: String,
    pub is_error: bool,
}

#[cfg(not(feature = "bof"))]
impl ClaudeResult {
    pub fn new(session_id: String, result: String, is_error: bool) -> Self {
        Self {
            session_id,
            result,
            is_error,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            session_id: String::new(),
            result: message,
            is_error: true,
        }
    }
}

/// Execution mode for Claude
#[cfg(not(feature = "bof"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Wrap claude CLI tool
    Wrapper,
    /// Direct API calls to Claude
    Api,
}

/// AI tool to execute
#[cfg(not(feature = "bof"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Claude,
    Codex,
    Gemini,
    Cursor,
}

/// Execute Claude with given prompt and optional `session_id`
#[cfg(not(feature = "bof"))]
pub fn run_claude(
    prompt: &str,
    session_id: Option<&str>,
    mode: ExecutionMode,
    api_key: Option<&str>,
    tool: Tool,
) -> ClaudeResult {
    match mode {
        ExecutionMode::Wrapper => execution::execute_tool_wrapper(prompt, session_id, tool),
        ExecutionMode::Api => {
            // Try provided key first, then environment variable
            let key = api_key
                .map(String::from)
                .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok());

            match key {
                Some(k) if !k.is_empty() => api::execute_claude_api(prompt, session_id, &k),
                _ => ClaudeResult::error(
                    "ANTHROPIC_API_KEY not set. Use -k flag or set environment variable."
                        .to_string(),
                ),
            }
        }
    }
}
