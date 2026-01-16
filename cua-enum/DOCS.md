# CUA-Enum Technical Documentation

## Architecture

CUA-Enum is designed as a dual-mode tool:
1. **Standalone EXE**: For direct execution on target systems
2. **Beacon Object File (BOF)**: For execution via Cobalt Strike's inline-execute

### Module Structure

```
src/
├── lib.rs          # Core library and enumeration orchestration
├── main.rs         # Standalone EXE entry point
├── beacon.rs       # BOF support and Beacon API bindings
├── output.rs       # Output formatting (plaintext/JSON)
├── enumeration.rs  # Shared enumeration utilities
└── agents/
    ├── mod.rs      # Agent module exports
    ├── claude.rs   # Claude Code enumeration
    ├── codex.rs    # OpenAI Codex CLI enumeration
    ├── cursor.rs   # Cursor IDE enumeration
    ├── gemini.rs   # Gemini CLI enumeration
    └── agents_md.rs # AGENTS.md file discovery
```

## Detection Logic

### User Enumeration

The tool starts by enumerating all user directories:

```
C:\Users\*
```

Excluded system directories:
- `Public`
- `Default`
- `Default User`
- `All Users`

### Recursive File Search

For each user, the tool searches common development directories:
- `source/`
- `repos/`
- `projects/`
- `dev/`
- `code/`
- `workspace/`

Maximum search depth: 4-5 levels (configurable per agent)

### Excluded Directories

To improve performance and avoid false positives, these directories are skipped:
- `node_modules/`
- `.git/`
- `target/`
- `bin/`
- `obj/`
- `__pycache__/`
- `.venv/`, `venv/`
- `dist/`
- `build/`

## Agent-Specific Search Logic

### Claude Code

**Search Paths:**
1. `~/.claude/settings.json` - Global settings
2. `~/.claude.json` - OAuth tokens, MCP servers
3. `~/.claude/agents/` - Subagent definitions
4. `~/.claude/CLAUDE.md` - Global instructions
5. `.claude/settings.json` - Per-project settings
6. `.claude/settings.local.json` - Local overrides
7. `.mcp.json` - MCP server configuration
8. `CLAUDE.md`, `CLAUDE.local.md` - Project instructions
9. `C:\Program Files\ClaudeCode\managed-*.json` - Enterprise configs

**Security-Relevant Fields:**
- `permissions.allow` - Allowed tool patterns
- `permissions.deny` - Denied tool patterns
- `env` - Environment variables
- `mcpServers` - MCP server definitions with potential credentials

### OpenAI Codex CLI

**Search Paths:**
1. `~/.codex/config.toml` - Main configuration
2. `~/.codex/AGENTS.md` - Global instructions
3. `~/.codex/AGENTS.override.md` - Override instructions
4. `~/.codex/skills/**/SKILL.md` - Skill definitions
5. `~/.codex/rules/` - Execution policy rules
6. `~/.codex/history.jsonl` - Command history

**MCP Server Parsing:**
The tool parses TOML sections like:
```toml
[mcp_servers.name]
type = "stdio"
command = "/path/to/server"
env = { "API_KEY" = "value" }
```

### Cursor IDE

**Search Paths:**
1. `%APPDATA%\Cursor\User\globalStorage\state.vscdb` - SQLite database
2. `%LOCALAPPDATA%\Cursor\User\globalStorage\state.vscdb` - Alternative location
3. `%APPDATA%\Cursor\User\settings.json` - User settings
4. `.cursor/rules/` - Project rules directory
5. `.cursor/environment.json` - Environment configuration
6. `.cursorrules` - Legacy rules file

**Note:** The SQLite database path is reported but not parsed (would require SQLite library).

### Gemini CLI

**Search Paths:**
1. `~/.gemini/settings.json` - Global settings
2. `.gemini/settings.json` - Project settings
3. `~/.gemini/extensions/` - Extension directory
4. `~/.gemini/commands/` - Custom commands
5. `.env` files - Environment variables with API keys

**MCP Server Parsing:**
Looks for `mcpServers` or `tools.mcpServers` keys in settings.json.

### AGENTS.md Discovery

Searches for:
- `AGENTS.md`
- `AGENTS.override.md`

Across all user directories with a maximum depth of 5 levels.

## Output Modes

### Plaintext Mode

Default output mode. Provides human-readable output with:
- Section headers for each agent type
- Indented configuration details
- Highlighted sensitive findings (`[!]` prefix)
- Summary statistics

### JSON Mode

Structured output for automated processing:
```json
{
  "claude_code": [...],
  "codex_cli": [...],
  "cursor": [...],
  "gemini_cli": [...],
  "agents_md": [...]
}
```

## BOF Implementation

### Entry Point

The BOF entry point follows Cobalt Strike conventions:

```rust
#[no_mangle]
pub unsafe extern "C" fn go(args: *mut u8, args_len: i32)
```

### Beacon API Integration

Output is sent via Beacon's output API:
- `CALLBACK_OUTPUT_UTF8` (0x20) - Standard output
- `CALLBACK_ERROR` (0x0d) - Error output

### Argument Parsing

Arguments are passed via Beacon's data API:
- First integer: Flags (bit 0 = JSON output)

## Security Considerations

### What CUA-Enum Reveals

1. **Permission Gaps**: Overly permissive tool access patterns
2. **Credential Exposure**: API keys, tokens in environment variables
3. **Operational Intelligence**: AGENTS.md files with business logic
4. **Attack Surface**: MCP server configurations that could be exploited

### Defensive Recommendations

1. Audit AI agent configurations regularly
2. Use environment variable references (`${VAR}`) instead of hardcoded values
3. Restrict file system access in MCP configurations
4. Review AGENTS.md files for sensitive information
5. Implement principle of least privilege in permission settings

## Testing

### Unit Tests

```powershell
cargo test
```

Tests validate:
- Configuration parsing
- File discovery logic
- User enumeration
- MCP server extraction

### Integration Testing

The test suite validates configuration parsing and file discovery logic against mock configurations in the `tests/mock_configs/` directory.

## Build Configuration

### Cargo.toml Features

- `default = ["exe"]` - Standalone executable
- `exe` - EXE-specific code
- `bof` - BOF-specific code with Beacon API bindings

### Release Profile

```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit
panic = "abort"      # Abort on panic
strip = true         # Strip symbols
```

### Size Constraint

The compiled BOF must be under 1MB to comply with BOF loading requirements.
