# CUA-Exec

Computer Use Agent Execution BOF - Execute Claude commands from Cobalt Strike Beacon or standalone.

## Features

- **Wrapper Mode**: Execute `claude` CLI tool with output capture
- **API Mode**: Direct Claude API access (requires API key)
- **Session Management**: Maintain conversation context across prompts
- **Dual Build**: Standalone EXE and BOF (Beacon Object File)

## Building

```powershell
# Build everything (release)
.\build.ps1 -All -Release

# Build only EXE
.\build.ps1 -Exe -Release

# Build only BOF
.\build.ps1 -Bof -Release

# Debug build
.\build.ps1 -All

# Run tests
.\build.ps1 -Test

# Clean
.\build.ps1 -Clean
```

## Usage

### Standalone EXE

```powershell
# Basic prompt (wrapper mode) - outputs plaintext
.\cua-exec.exe "what is 2+2?"
.\cua-exec.exe -p "explain buffer overflows"

# JSON output
.\cua-exec.exe -j "what is 2+2?"

# Resume session
.\cua-exec.exe -s <session_id> -p "what was my last question?"

# API mode (requires ANTHROPIC_API_KEY env var or -k flag)
.\cua-exec.exe -a -p "hello world"
.\cua-exec.exe -a -k sk-ant-xxx -p "hello world"
```

### Cobalt Strike

First, run `setup.ps1` to download the JSON library and configure the Aggressor script:

```powershell
.\setup.ps1
```

Then load `claude.cna` in Cobalt Strike's Script Manager.

```
beacon> claude "what processes are running?"
beacon> claude "what files are in C:\Users?"
beacon> claude_session
beacon> claude_reset
```

## Options

| Option | Description |
|--------|-------------|
| `-p, --prompt <text>` | Prompt to send to Claude |
| `-s, --session <id>` | Resume existing session by ID |
| `-j, --json` | Output results in JSON format |
| `-a, --api` | Use direct API mode instead of CLI wrapper |
| `-k, --api-key <key>` | API key for direct API mode |
| `-h, --help` | Show help message |

## Modes

### Wrapper Mode (Default)

Executes `claude` with the following flags:
- `--dangerously-skip-permissions` - Bypass permission checks
- `--add-dir "C:\\"` - Add C:\ to allowed directories
- `--output-format json` - Return structured JSON

**Requirements**: `claude` must be in PATH on target system.

### API Mode

Direct HTTP calls to Claude API.

**Requirements**: `ANTHROPIC_API_KEY` environment variable or `-k` flag.

## Output Format

By default, outputs plaintext (just the result). Use `-j` or `--json` for structured output:

```
# Default (plaintext)
.\cua-exec.exe "what is 2+2?"
The answer is 4

# JSON output
.\cua-exec.exe -j "what is 2+2?"
{"session_id":"abc123...","result":"The answer is 4","is_error":false}
```

## Session Management

- First prompt creates a new session
- Session ID is returned in output
- Pass session ID with `-s` to maintain context
- Use `claude_reset` in Beacon to clear session

## Requirements

- **Build**: Rust 1.70+, Windows target `x86_64-pc-windows-msvc`
- **Wrapper Mode**: Claude Code CLI (`claude`) in PATH
- **API Mode**: Anthropic API key
- **BOF Execution**: Cobalt Strike 4.9+ or COFFLoader

## Security Notes

This tool is intended for authorized security testing only. The `--dangerously-skip-permissions` flag bypasses Claude's safety checks.
