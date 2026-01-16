# CUA-Kit Documentation

## Contents

- [Building](building.md) - How to compile the tools
- [Usage](usage.md) - How to run the standalone executables
- [Cobalt Strike Integration](cobalt-strike.md) - BOF usage and Aggressor Scripts

## Platform Support

| Platform | Standalone EXE | BOF |
|----------|----------------|-----|
| Windows (x64) | Yes | Yes |
| macOS (Intel/ARM) | Yes | No |

## Quick Reference

| Tool | Purpose | Windows Outputs | macOS Outputs |
|------|---------|-----------------|---------------|
| **cua-enum** | Enumerate AI agent configurations | `cua-enum.exe`, `cua-enum.x64.o` | `cua-enum` |
| **cua-exec** | Execute computer use agent commands (Claude, Codex, Gemini, Cursor) | `cua-exec.exe`, `cua-exec.x64.o` | `cua-exec` |
| **cua-poison** | Poison Claude Code sessions | `cua-poison.exe`, `cua-poison.x64.o` | `cua-poison` |

## Requirements

- **Build (Windows)**: Rust 1.70+, `x86_64-pc-windows-msvc` target
- **Build (macOS)**: Rust 1.70+
- **Runtime (cua-exec)**: One or more CLI tools in PATH (`claude`, `codex`, `gemini`, or `cursor agent`)
- **Runtime (cua-poison)**: No external dependencies (file access only)
- **BOF Execution**: Cobalt Strike 4.9+ or COFFLoader (Windows only)
