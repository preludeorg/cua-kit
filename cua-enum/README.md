# CUA-Enum

Computer Use Agent Enumeration Tool - A Beacon Object File (BOF) for enumerating AI/LLM agent configurations on Windows systems.

## Overview

CUA-Enum is a red team tool designed to enumerate local AI/LLM agent settings and configurations. It helps identify security gaps, over-permissioning, and other primitives that could be exploited during post-exploitation activities.

## Supported Agents

### Claude Code (Anthropic)

| File | Location | Description |
|------|----------|-------------|
| `settings.json` | `~/.claude/settings.json` | Global user settings |
| `settings.json` | `.claude/settings.json` | Project-level settings |
| `settings.local.json` | `.claude/settings.local.json` | Local project overrides |
| `.claude.json` | `~/.claude.json` | Preferences, OAuth tokens, MCP servers |
| `.mcp.json` | Project root | MCP server configuration |
| `CLAUDE.md` | Various | Project instructions |
| `agents/` | `~/.claude/agents/` | Subagent configurations |
| `managed-settings.json` | `C:\Program Files\ClaudeCode\` | Enterprise managed settings |
| `managed-mcp.json` | `C:\Program Files\ClaudeCode\` | Enterprise managed MCP |

### OpenAI Codex CLI

| File | Location | Description |
|------|----------|-------------|
| `config.toml` | `~/.codex/config.toml` | Main configuration |
| `AGENTS.md` | `~/.codex/AGENTS.md` | Global instructions |
| `AGENTS.override.md` | `~/.codex/AGENTS.override.md` | Override instructions |
| `SKILL.md` | `~/.codex/skills/**/SKILL.md` | Skill definitions |
| Rules | `~/.codex/rules/` | Execution policy rules |
| `history.jsonl` | `~/.codex/history.jsonl` | Command history |

### Cursor IDE

| File | Location | Description |
|------|----------|-------------|
| `state.vscdb` | `%APPDATA%/Cursor/User/globalStorage/` | SQLite settings database |
| Rules | `.cursor/rules/` | AI behavior rules |
| `environment.json` | `.cursor/environment.json` | Environment configuration |
| `.cursorrules` | Project root | Legacy rules file |

### Gemini CLI (Google)

| File | Location | Description |
|------|----------|-------------|
| `settings.json` | `~/.gemini/settings.json` | Global settings |
| `settings.json` | `.gemini/settings.json` | Project settings |
| Extensions | `~/.gemini/extensions/` | Extension directory |
| Commands | `~/.gemini/commands/` | Custom commands |
| `.env` | Various | Environment variables with API keys |

### AGENTS.md Files

Searches for `AGENTS.md` and `AGENTS.override.md` files across all project directories. These files provide instructions to AI agents and may contain sensitive operational context.

## Building

### Prerequisites

- Rust toolchain (1.70+)
- Windows target: `x86_64-pc-windows-msvc`

### Build Commands

```powershell
# Build standalone EXE
cargo build --release --bin cua-enum

# Build with all features
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy -- -W clippy::all -W clippy::pedantic
```

### Using the Build Script

```powershell
# Build everything
.\build.ps1 -All -Release

# Build only EXE
.\build.ps1 -Exe -Release

# Build only BOF
.\build.ps1 -Bof -Release

# Run tests
.\build.ps1 -Test

# Clean
.\build.ps1 -Clean
```

## Usage

### Standalone EXE

```cmd
# Enumerate all users (default)
cua-enum.exe

# Output as JSON
cua-enum.exe --json

# Target specific user
cua-enum.exe --user jsmith

# Show help
cua-enum.exe --help
```

### Cobalt Strike / C2 Integration

The standalone EXE can be converted to shellcode using tools like [donut](https://github.com/TheWover/donut) for execution via Cobalt Strike's `execute-assembly` or other C2 frameworks:

```bash
# Convert EXE to shellcode
donut -f cua-enum.exe -o cua-enum.bin

# In Cobalt Strike
execute-assembly /path/to/cua-enum.exe
```

**Note:** Full native BOF support would require custom linker scripts and relocation handling. The EXE-to-shellcode approach is recommended for production use.

## Output

### Plaintext (Default)

```
=== CUA-Enum: Computer Use Agent Enumeration ===

[*] Claude Code Configurations:
--------------------------------
  User: jsmith
  Global Settings: C:\Users\jsmith\.claude\settings.json
    Allowed: ["Bash(cargo test:*)","Bash(cargo build:*)"]
    Denied: ["Bash(curl:*)"]
  MCP Config: C:\Users\jsmith\project\.mcp.json
    - filesystem (stdio)
    - github (http)
      [!] Sensitive env var: GITHUB_TOKEN

[*] Summary: 3 agent configurations found
    2 AGENTS.md files found
```

### JSON

Pass `-j` or `--json` flag for structured JSON output suitable for automated processing.

## Security Findings

CUA-Enum specifically highlights:

- **Over-permissioned tools**: Bash commands, file access patterns
- **Sensitive environment variables**: API keys, tokens, secrets in MCP configs
- **Exposed credentials**: Hardcoded values in configuration files
- **AGENTS.md content**: Operational instructions that reveal business logic

## License

For authorized security testing and red team operations only.

## References

- [Claude Code Settings Documentation](https://docs.anthropic.com/en/docs/claude-code/settings)
- [Claude Code MCP Documentation](https://docs.anthropic.com/en/docs/claude-code/mcp)
- [OpenAI Codex CLI Configuration](https://developers.openai.com/codex/config-reference/)
- [Cursor CLI Configuration](https://cursor.com/docs/cli/reference/configuration)
- [Gemini CLI Configuration](https://geminicli.com/docs/get-started/configuration/)
- [Beacon Object File Development](https://www.cobaltstrike.com/blog/simplifying-bof-development)
