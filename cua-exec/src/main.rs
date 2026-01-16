//! CUA-Exec Standalone Executable
//!
//! Entry point for running as a standalone Windows executable.

use cua_exec_lib::{ClaudeResult, ExecutionMode, Tool, run_claude};
use std::env;

#[allow(clippy::too_many_lines)]
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut prompt: Option<String> = None;
    let mut session_id: Option<String> = None;
    let mut mode = ExecutionMode::Wrapper;
    let mut api_key: Option<String> = None;
    let mut json_output = false;
    let mut tool = Tool::Claude; // Default to Claude for backward compatibility

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" | "--prompt" => {
                if i + 1 < args.len() {
                    prompt = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-s" | "--session" => {
                if i + 1 < args.len() {
                    session_id = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-a" | "--api" => {
                mode = ExecutionMode::Api;
            }
            "-k" | "--api-key" => {
                if i + 1 < args.len() {
                    api_key = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "-t" | "--tool" => {
                if i + 1 < args.len() {
                    tool = match args[i + 1].to_lowercase().as_str() {
                        "claude" => Tool::Claude,
                        "codex" => Tool::Codex,
                        "gemini" => Tool::Gemini,
                        "cursor" => Tool::Cursor,
                        unknown => {
                            eprintln!(
                                "Unknown tool: {unknown}. Use 'claude', 'codex', 'gemini', or 'cursor'"
                            );
                            print_help();
                            return;
                        }
                    };
                    i += 1;
                }
            }
            "-j" | "--json" => {
                json_output = true;
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            arg if !arg.starts_with('-') => {
                // Treat as prompt if not already set
                if prompt.is_none() {
                    prompt = Some(args[i..].join(" "));
                    break;
                }
            }
            _ => {
                eprintln!("Unknown option: {}", args[i]);
                print_help();
                return;
            }
        }
        i += 1;
    }

    let Some(prompt_text) = prompt else {
        eprintln!("Error: No prompt provided");
        print_help();
        return;
    };

    let result = run_claude(
        &prompt_text,
        session_id.as_deref(),
        mode,
        api_key.as_deref(),
        tool,
    );

    // Output result
    if json_output {
        match serde_json::to_string(&result) {
            Ok(json) => println!("{json}"),
            Err(e) => {
                let error_result = ClaudeResult::error(format!("JSON serialization failed: {e}"));
                if let Ok(json) = serde_json::to_string(&error_result) {
                    println!("{json}");
                } else {
                    eprintln!("Fatal error: Could not serialize result");
                }
            }
        }
    } else {
        // Plaintext output - just the result
        if result.is_error {
            eprintln!("Error: {}", result.result);
        } else {
            println!("{}", result.result);
        }
    }
}

fn print_help() {
    println!("CUA-Exec - Computer Use Agent Execution Tool (Claude, Codex, Gemini, Cursor)");
    println!();
    println!("Usage: cua-exec.exe [OPTIONS] [prompt]");
    println!();
    println!("Options:");
    println!("  -p, --prompt <text>   Prompt to send to AI tool");
    println!("  -s, --session <id>    Resume existing session by ID");
    println!(
        "  -t, --tool <name>     AI tool: 'claude', 'codex', 'gemini', or 'cursor' (default: claude)"
    );
    println!("  -j, --json            Output results in JSON format");
    println!("  -a, --api             Use direct API mode instead of CLI wrapper");
    println!("  -k, --api-key <key>   API key for direct API mode");
    println!("  -h, --help            Show this help message");
    println!();
    println!("Modes:");
    println!("  Wrapper (default)     Executes AI CLI tool");
    println!("  API (-a flag)         Direct HTTP calls to Claude API (Claude only)");
    println!();
    println!("Examples:");
    println!("  cua-exec.exe \"what is 2+2?\"");
    println!("  cua-exec.exe -p \"explain buffer overflows\"");
    println!("  cua-exec.exe -t codex \"what is a buffer overflow?\"");
    println!("  cua-exec.exe -t gemini \"what is a buffer overflow?\"");
    println!("  cua-exec.exe -t cursor \"explain this code\"");
    println!("  cua-exec.exe -j -p \"what is 2+2?\"");
    println!("  cua-exec.exe -s abc123 -p \"what was my last question?\"");
    println!("  cua-exec.exe -t codex -s abc123 \"explain more\"");
    println!("  cua-exec.exe -a -k sk-ant-xxx -p \"hello world\"");
    println!();
    println!("Environment Variables:");
    println!("  ANTHROPIC_API_KEY     API key for direct API mode");
    println!("  CURSOR_API_KEY        API key for Cursor CLI");
}
