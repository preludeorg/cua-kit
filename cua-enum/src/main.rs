//! CUA-Enum Standalone Executable
//!
//! Entry point for running as a standalone Windows executable.

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut json_output = false;
    let mut target_user: Option<&str> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-j" | "--json" => json_output = true,
            "-u" | "--user" => {
                if i + 1 < args.len() {
                    target_user = Some(&args[i + 1]);
                    i += 1;
                }
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    cua_enum_lib::run_enumeration(json_output, target_user);
}

fn print_help() {
    println!("CUA-Enum - Computer Use Agent Enumeration Tool");
    println!();
    println!("Usage: cua-enum.exe [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -j, --json     Output results in JSON format");
    println!("  -u, --user     Target specific user (default: all users)");
    println!("  -h, --help     Show this help message");
    println!();
    println!("Supported Agents:");
    println!("  - Claude Code (Anthropic)");
    println!("  - Codex CLI (OpenAI)");
    println!("  - Cursor IDE");
    println!("  - Gemini CLI (Google)");
    println!("  - AGENTS.md files");
}
