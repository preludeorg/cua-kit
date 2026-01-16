# Building CUA-Kit

This guide covers how to compile the CUA-Kit tools from source.

## Platform Support

| Platform | Standalone EXE | BOF |
|----------|----------------|-----|
| Windows (x64) | Yes | Yes |
| macOS (Intel/ARM) | Yes | No |

---

## Windows

### Prerequisites

1. **Rust Toolchain** (1.70 or later)
   ```powershell
   # Install via rustup (https://rustup.rs)
   rustup default stable
   rustup target add x86_64-pc-windows-msvc
   ```

2. **Visual Studio Build Tools** (for MSVC linker)
   - Install "Desktop development with C++" workload
   - Or install via `winget install Microsoft.VisualStudio.2022.BuildTools`

3. **PowerShell** (included with Windows)

### Building All Tools

From the repository root:

```powershell
# Build everything (release mode)
.\build.ps1 -Release

# Build everything (debug mode)
.\build.ps1

# Clean all build artifacts
.\build.ps1 -Clean

# Run tests
.\build.ps1 -Test
```

### Building Individual Tools

```powershell
# Build specific tool (release)
.\build.ps1 -Tool enum -Release
.\build.ps1 -Tool exec -Release
.\build.ps1 -Tool poison -Release

# Build only EXE
.\build.ps1 -Tool enum -Exe -Release

# Build only BOF
.\build.ps1 -Tool enum -Bof -Release
```

### Build Outputs (Windows)

After a successful build, all artifacts are placed in `bin/release/` (or `bin/debug/`):

| File | Description |
|------|-------------|
| `cua-enum.exe` | Standalone enumeration executable |
| `cua-enum.x64.o` | BOF object file (x64) |
| `cua-exec.exe` | Standalone execution executable |
| `cua-exec.x64.o` | BOF object file (x64) |
| `cua-poison.exe` | Standalone session poisoning executable |
| `cua-poison.x64.o` | BOF object file (x64) |

---

## macOS

### Prerequisites

1. **Rust Toolchain** (1.70 or later)
   ```bash
   # Install via rustup (https://rustup.rs)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   rustup default stable
   ```

2. **For universal binaries on macOS** (optional):
   ```bash
   rustup target add x86_64-apple-darwin
   rustup target add aarch64-apple-darwin
   ```

### Building All Tools

From the repository root:

```bash
# Build everything (release mode)
./build.sh -release

# Build everything (debug mode)
./build.sh

# Clean all build artifacts
./build.sh -clean

# Run tests
./build.sh -test
```

### Building Individual Tools

```bash
# Build specific tool (release)
./build.sh -tool enum -release
./build.sh -tool exec -release
./build.sh -tool poison -release
```

### Universal Binaries (macOS only)

Build fat binaries that run natively on both Intel and Apple Silicon:

```bash
# Build all tools as universal binaries
./build.sh -release -universal

# Build specific tool as universal binary
./build.sh -tool enum -release -universal
```

### Build Outputs (macOS)

After a successful build, all artifacts are placed in `bin/release/` (or `bin/debug/`):

| File | Description |
|------|-------------|
| `cua-enum` | Standalone enumeration executable |
| `cua-exec` | Standalone execution executable |
| `cua-poison` | Standalone session poisoning executable |

Note: BOF files are not built on macOS (Windows-only feature).

## Build Modes

### EXE Mode (Default)

Compiles to a standard Windows executable using the full Rust standard library.

```powershell
.\build.ps1 -Exe -Release
```

Features:
- Full `std` library support
- JSON serialization with serde
- Windows API via `windows` crate
- Can run standalone on any Windows system

### BOF Mode

Compiles to a COFF object file for Beacon Object File execution.

```powershell
.\build.ps1 -Bof -Release
```

Features:
- `no_std` environment (no standard library)
- Custom heap allocator using Windows HeapAlloc
- COFFLoader-compatible symbol naming (`__imp_LIBRARY$Function`)
- Beacon API integration for output

## Understanding the Dual-Mode Architecture

All tools use Rust's feature flags to compile different code paths:

```toml
# Cargo.toml
[features]
default = ["exe"]
exe = []      # Standard executable with std
bof = []      # BOF with no_std
```

The source code uses conditional compilation:

```rust
// lib.rs
#![cfg_attr(feature = "bof", no_std)]

#[cfg(feature = "bof")]
mod bof;  // BOF-specific implementation

#[cfg(not(feature = "bof"))]
mod execution;  // EXE-specific implementation
```

## Troubleshooting

### "cargo not found"

Ensure Rust is installed and in your PATH:
```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
```

### "linker not found"

Install Visual Studio Build Tools with C++ support.

### BOF size exceeds 1MB

The build script warns if the BOF exceeds 1MB. To reduce size:
- Ensure release mode: `-Release` flag
- The `Cargo.toml` is already configured for size optimization:
  ```toml
  [profile.release]
  opt-level = "z"    # Optimize for size
  lto = true         # Link-time optimization
  strip = true       # Remove symbols
  ```

### Object file not found after BOF build

The BOF build looks for `.o` files in `target/x86_64-pc-windows-msvc/release/deps/`. If not found:
1. Ensure the MSVC target is installed: `rustup target add x86_64-pc-windows-msvc`
2. Check for compilation errors in the output

## Manual Build Commands

If you prefer not to use the build scripts:

### EXE Build
```powershell
cargo build --release -p cua-enum --bin cua-enum
cargo build --release -p cua-exec --bin cua-exec
cargo build --release -p cua-poison --bin cua-poison
```

### BOF Build
```powershell
cargo rustc --release -p cua-enum --features bof --lib --crate-type=staticlib -- --emit=obj -C panic=abort -C opt-level=z
```

Then copy the `.o` file from `target/x86_64-pc-windows-msvc/release/deps/` to `bin/release/`.
