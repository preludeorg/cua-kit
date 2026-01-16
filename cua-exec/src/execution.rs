//! Claude execution via CLI wrapper
//!
//! This module wraps the claude CLI tool to execute prompts.

use crate::{ClaudeResult, Tool};
use std::process::{Command, Stdio};

/// Create a command for the given tool name with platform-appropriate shell wrapping
fn create_tool_command(tool_name: &str) -> Command {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("cmd.exe");
        cmd.args(["/c", tool_name]);
        cmd
    }
    #[cfg(not(windows))]
    {
        // On Unix-like systems (macOS, Linux), run tool directly
        Command::new(tool_name)
    }
}

/// Get the root directory path for filesystem access flags
fn get_root_dir() -> &'static str {
    #[cfg(windows)]
    {
        "C:\\"
    }
    #[cfg(not(windows))]
    {
        "/"
    }
}

/// Configure the command to hide the window (Windows-only)
fn configure_hidden_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        // No-op on non-Windows - processes don't create visible windows by default
        let _ = cmd;
    }
}

/// Execute via CLI wrapper with specified tool
pub fn execute_tool_wrapper(prompt: &str, session_id: Option<&str>, tool: Tool) -> ClaudeResult {
    match tool {
        Tool::Claude => execute_claude_wrapper(prompt, session_id),
        Tool::Codex => execute_codex_wrapper(prompt, session_id),
        Tool::Gemini => execute_gemini_wrapper(prompt, session_id),
        Tool::Cursor => execute_cursor_wrapper(prompt, session_id),
    }
}

/// Execute Claude via the CLI wrapper
pub fn execute_claude_wrapper(prompt: &str, session_id: Option<&str>) -> ClaudeResult {
    // Build command with platform-appropriate wrapping
    let mut cmd = create_tool_command("claude");
    cmd.args(["--dangerously-skip-permissions"]);
    cmd.args(["--add-dir", get_root_dir()]);
    cmd.args(["--output-format", "json"]);

    // Add session resume if provided
    if let Some(sid) = session_id
        && !sid.is_empty()
    {
        cmd.args(["-r", sid]);
    }

    // Add prompt
    cmd.args(["-p", prompt]);

    // Configure process
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::null());

    // Hide window on Windows
    configure_hidden_window(&mut cmd);

    // Execute
    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            return ClaudeResult::error(format!("Failed to spawn process: {}", e));
        }
    };

    // Wait for completion with timeout
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => {
            return ClaudeResult::error(format!("Failed to wait for process: {}", e));
        }
    };

    // Get output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let error_msg = if stderr.is_empty() {
            format!("Process exited with status: {}", output.status)
        } else {
            stderr.to_string()
        };
        return ClaudeResult::error(error_msg);
    }

    // Parse JSON output from claude
    parse_claude_output(&stdout)
}

/// Execute Codex via the CLI wrapper
pub fn execute_codex_wrapper(prompt: &str, session_id: Option<&str>) -> ClaudeResult {
    let mut cmd = create_tool_command("codex");
    cmd.arg("exec");

    // Codex uses "resume SESSION_ID" as a subcommand
    if let Some(sid) = session_id
        && !sid.is_empty()
    {
        cmd.args(["resume", sid]);
    }

    // Prompt is positional (no -p flag)
    cmd.arg(prompt);

    // Codex flags
    cmd.args(["--yolo", "--json", "--skip-git-repo-check"]);

    // Configure process
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::null());

    // Hide window on Windows
    configure_hidden_window(&mut cmd);

    // Execute
    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            return ClaudeResult::error(format!("Failed to spawn codex: {}", e));
        }
    };

    // Wait for completion with timeout
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => {
            return ClaudeResult::error(format!("Failed to wait for codex: {}", e));
        }
    };

    // Get output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let error_msg = if stderr.is_empty() {
            format!("Codex exited with status: {}", output.status)
        } else {
            stderr.to_string()
        };
        return ClaudeResult::error(error_msg);
    }

    // Parse Codex output (newline-delimited JSON)
    parse_codex_output(&stdout)
}

/// Execute Gemini via the CLI wrapper
pub fn execute_gemini_wrapper(prompt: &str, session_id: Option<&str>) -> ClaudeResult {
    let mut cmd = create_tool_command("gemini");

    // Add session resume if provided
    if let Some(sid) = session_id
        && !sid.is_empty()
    {
        cmd.args(["--resume", sid]);
    }

    // Prompt flag
    cmd.args(["-p", prompt]);

    // Gemini flags
    cmd.args(["--yolo"]); // Auto-approve actions
    cmd.args(["--output-format", "json"]);
    cmd.args(["--include-directories", get_root_dir()]); // Like Claude's --add-dir

    // Configure process (same pattern as others)
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::null());

    configure_hidden_window(&mut cmd);

    // Execute
    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => return ClaudeResult::error(format!("Failed to spawn gemini: {}", e)),
    };

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => return ClaudeResult::error(format!("Failed to wait for gemini: {}", e)),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let error_msg = if stderr.is_empty() {
            format!("Gemini exited with status: {}", output.status)
        } else {
            stderr.to_string()
        };
        return ClaudeResult::error(error_msg);
    }

    // Parse Gemini JSON output
    parse_gemini_output(&stdout)
}

/// Execute Cursor via the CLI wrapper
pub fn execute_cursor_wrapper(prompt: &str, session_id: Option<&str>) -> ClaudeResult {
    let mut cmd = create_tool_command("agent"); // Note: command is "agent", not "cursor"

    // Add session resume if provided (Cursor uses --resume="SESSION_ID" format)
    if let Some(sid) = session_id
        && !sid.is_empty()
    {
        let resume_arg = format!("--resume={}", sid);
        cmd.arg(&resume_arg);
    }

    // Prompt with print mode
    cmd.args(["-p", prompt]);

    // Cursor flags
    cmd.args(["--force"]); // Like yolo mode
    cmd.args(["--output-format", "json"]);

    // Configure process (same pattern as others)
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::null());

    configure_hidden_window(&mut cmd);

    // Execute
    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => return ClaudeResult::error(format!("Failed to spawn cursor: {}", e)),
    };

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => return ClaudeResult::error(format!("Failed to wait for cursor: {}", e)),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        let error_msg = if stderr.is_empty() {
            format!("Cursor exited with status: {}", output.status)
        } else {
            stderr.to_string()
        };
        return ClaudeResult::error(error_msg);
    }

    // Parse Cursor JSON output
    parse_cursor_output(&stdout)
}

/// Parse Claude CLI JSON output and extract `session_id` and result
fn parse_claude_output(output: &str) -> ClaudeResult {
    // Claude outputs JSON with session_id and result fields
    // Try to parse as JSON
    match serde_json::from_str::<serde_json::Value>(output.trim()) {
        Ok(json) => {
            let session_id = json
                .get("session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let result = json
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or(output)
                .to_string();

            let is_error = json
                .get("is_error")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            ClaudeResult::new(session_id, result, is_error)
        }
        Err(_) => {
            // If not valid JSON, return raw output
            ClaudeResult::new(String::new(), output.to_string(), false)
        }
    }
}

/// Parse Codex CLI newline-delimited JSON output
fn parse_codex_output(output: &str) -> ClaudeResult {
    let mut session_id = String::new();
    let mut result_text = String::new();
    let mut is_error = false;

    // Process each line as a JSON event
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            // Extract session_id if present
            if let Some(sid) = json.get("session_id").and_then(|v| v.as_str()) {
                session_id = sid.to_string();
            }

            // Extract thread_id as session_id if present
            if let Some(tid) = json.get("thread_id").and_then(|v| v.as_str())
                && session_id.is_empty()
            {
                session_id = tid.to_string();
            }

            // Check for item.completed with agent_message type
            if json.get("type").and_then(|v| v.as_str()) == Some("item.completed")
                && let Some(item) = json.get("item")
            {
                // Check if this is an agent_message
                if item.get("type").and_then(|v| v.as_str()) == Some("agent_message")
                    && let Some(text) = item.get("text").and_then(|v| v.as_str())
                {
                    result_text.push_str(text);
                }
            }

            // Also try content/text fields for compatibility
            if let Some(text) = json.get("content").and_then(|v| v.as_str()) {
                result_text.push_str(text);
            } else if let Some(text) = json.get("text").and_then(|v| v.as_str()) {
                result_text.push_str(text);
            }

            // Check for error events
            if let Some(err) = json.get("error").and_then(serde_json::Value::as_bool)
                && err
            {
                is_error = true;
            }
        } else {
            // Non-JSON line, append as plain text
            result_text.push_str(line);
            result_text.push('\n');
        }
    }

    // Fallback to raw output if nothing parsed
    if result_text.is_empty() {
        result_text = output.to_string();
    }

    ClaudeResult::new(session_id, result_text, is_error)
}

/// Parse Gemini CLI JSON output
fn parse_gemini_output(output: &str) -> ClaudeResult {
    // Gemini outputs JSON with "response" field for the answer
    match serde_json::from_str::<serde_json::Value>(output.trim()) {
        Ok(json) => {
            // Extract session ID (try multiple field names)
            let session_id = json
                .get("session_id")
                .and_then(|v| v.as_str())
                .or_else(|| json.get("chat_id").and_then(|v| v.as_str()))
                .or_else(|| json.get("thread_id").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();

            // Try to extract response from various possible fields
            let result = json
                .get("response")
                .and_then(|v| v.as_str())
                .or_else(|| json.get("result").and_then(|v| v.as_str()))
                .or_else(|| json.get("text").and_then(|v| v.as_str()))
                .unwrap_or(output)
                .to_string();

            let is_error = json
                .get("error")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            ClaudeResult::new(session_id, result, is_error)
        }
        Err(_) => {
            // If not valid JSON, return raw output
            ClaudeResult::new(String::new(), output.to_string(), false)
        }
    }
}

/// Parse Cursor CLI JSON output
fn parse_cursor_output(output: &str) -> ClaudeResult {
    // Cursor outputs JSON with response data
    match serde_json::from_str::<serde_json::Value>(output.trim()) {
        Ok(json) => {
            // Extract session ID (try multiple field names - Cursor uses "chat_id")
            let session_id = json
                .get("chat_id")
                .and_then(|v| v.as_str())
                .or_else(|| json.get("session_id").and_then(|v| v.as_str()))
                .or_else(|| json.get("thread_id").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();

            // Try to extract response from various possible fields
            let result = json
                .get("response")
                .and_then(|v| v.as_str())
                .or_else(|| json.get("result").and_then(|v| v.as_str()))
                .or_else(|| json.get("text").and_then(|v| v.as_str()))
                .or_else(|| json.get("output").and_then(|v| v.as_str()))
                .unwrap_or(output)
                .to_string();

            let is_error = json
                .get("error")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);

            ClaudeResult::new(session_id, result, is_error)
        }
        Err(_) => {
            // If not valid JSON, return raw output
            ClaudeResult::new(String::new(), output.to_string(), false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_claude_output_json() {
        let output = r#"{"session_id": "abc123", "result": "Hello!", "is_error": false}"#;
        let result = parse_claude_output(output);
        assert_eq!(result.session_id, "abc123");
        assert_eq!(result.result, "Hello!");
        assert!(!result.is_error);
    }

    #[test]
    fn test_parse_claude_output_plain() {
        let output = "Plain text output";
        let result = parse_claude_output(output);
        assert!(result.session_id.is_empty());
        assert_eq!(result.result, "Plain text output");
        assert!(!result.is_error);
    }

    #[test]
    fn test_parse_codex_output_ndjson() {
        let output = r#"{"session_id": "codex123", "text": "Hello"}
{"text": " World"}
{"status": "complete"}"#;
        let result = parse_codex_output(output);
        assert_eq!(result.session_id, "codex123");
        assert_eq!(result.result, "Hello World");
        assert!(!result.is_error);
    }

    #[test]
    fn test_parse_gemini_output_json() {
        let output =
            r#"{"session_id": "gemini-uuid-123", "response": "Hello World!", "error": false}"#;
        let result = parse_gemini_output(output);
        assert_eq!(result.session_id, "gemini-uuid-123");
        assert_eq!(result.result, "Hello World!");
        assert!(!result.is_error);
    }

    #[test]
    fn test_parse_gemini_output_no_session() {
        let output = r#"{"response": "Hello!", "error": false}"#;
        let result = parse_gemini_output(output);
        assert!(result.session_id.is_empty()); // No session in output
        assert_eq!(result.result, "Hello!");
        assert!(!result.is_error);
    }

    #[test]
    fn test_parse_cursor_output_json() {
        let output = r#"{"chat_id": "cursor-chat-456", "output": "Test result", "error": false}"#;
        let result = parse_cursor_output(output);
        assert_eq!(result.session_id, "cursor-chat-456");
        assert_eq!(result.result, "Test result");
        assert!(!result.is_error);
    }

    #[test]
    fn test_parse_cursor_output_no_session() {
        let output = r#"{"output": "Test result", "error": false}"#;
        let result = parse_cursor_output(output);
        assert!(result.session_id.is_empty()); // No session in output
        assert_eq!(result.result, "Test result");
        assert!(!result.is_error);
    }
}
