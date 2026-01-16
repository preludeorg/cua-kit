//! CUA-Enum: Computer Use Agent Enumeration
//!
//! A security tool for enumerating AI/LLM agent configurations on Windows systems.
//! Supports both BOF (Beacon Object File) and standalone EXE modes.

// For BOF builds, use no_std with alloc
#![cfg_attr(feature = "bof", no_std)]
#![cfg_attr(feature = "bof", no_main)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::doc_markdown)]

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
pub mod agents;
#[cfg(not(feature = "bof"))]
pub mod beacon;
#[cfg(not(feature = "bof"))]
pub mod enumeration;
#[cfg(not(feature = "bof"))]
pub mod output;
#[cfg(not(feature = "bof"))]
pub mod platform;

#[cfg(not(feature = "bof"))]
use output::OutputFormat;

/// Main enumeration logic - called by both BOF and EXE entry points
#[cfg(not(feature = "bof"))]
pub fn run_enumeration(json_output: bool, target_user: Option<&str>) {
    let format = if json_output {
        OutputFormat::Json
    } else {
        OutputFormat::Plain
    };

    let mut results = enumeration::EnumerationResults::new();

    // Determine which users to enumerate
    let users = if let Some(user) = target_user {
        vec![user.to_string()]
    } else {
        enumeration::get_all_users()
    };

    for user in &users {
        // Claude Code enumeration
        if let Some(claude_results) = agents::claude::enumerate_claude_code(user) {
            results.claude_code.push(claude_results);
        }

        // OpenAI Codex CLI enumeration
        if let Some(codex_results) = agents::codex::enumerate_codex(user) {
            results.codex_cli.push(codex_results);
        }

        // Cursor enumeration
        if let Some(cursor_results) = agents::cursor::enumerate_cursor(user) {
            results.cursor.push(cursor_results);
        }

        // Gemini CLI enumeration
        if let Some(gemini_results) = agents::gemini::enumerate_gemini(user) {
            results.gemini_cli.push(gemini_results);
        }

        // AGENTS.md enumeration
        let agents_md = agents::agents_md::find_agents_md_files(user);
        results.agents_md.extend(agents_md);
    }

    // Output results
    output::print_results(&results, format);
}
