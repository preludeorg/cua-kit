#!/bin/bash
# CUA-Kit Build Script for macOS/Linux
# Usage: ./build.sh [-release] [-tool <name>] [-exe] [-test] [-clean] [-universal]

set -e

# Defaults
TOOL="all"
RELEASE=false
EXE_ONLY=false
TEST=false
CLEAN=false
UNIVERSAL=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -release|--release)
            RELEASE=true
            shift
            ;;
        -tool|--tool)
            TOOL="$2"
            shift 2
            ;;
        -exe|--exe)
            EXE_ONLY=true
            shift
            ;;
        -test|--test)
            TEST=true
            shift
            ;;
        -clean|--clean)
            CLEAN=true
            shift
            ;;
        -universal|--universal)
            UNIVERSAL=true
            shift
            ;;
        -h|--help)
            echo "Usage: ./build.sh [-release] [-tool <name>] [-exe] [-test] [-clean] [-universal]"
            echo ""
            echo "Options:"
            echo "  -release    Build in release mode (optimized)"
            echo "  -tool NAME  Build specific tool: all, enum, exec, poison"
            echo "  -exe        Build only EXE (no BOF - BOF is Windows-only)"
            echo "  -test       Run tests"
            echo "  -clean      Clean build artifacts"
            echo "  -universal  Build universal binary (macOS only - Intel + ARM)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Determine profile
if [ "$RELEASE" = true ]; then
    PROFILE="release"
    CARGO_FLAGS="--release"
else
    PROFILE="debug"
    CARGO_FLAGS=""
fi

# Script directory
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="$ROOT/bin/$PROFILE"

# Tool name lookup (compatible with bash 3.x)
get_tool_name() {
    case "$1" in
        enum)   echo "cua-enum" ;;
        exec)   echo "cua-exec" ;;
        poison) echo "cua-poison" ;;
        *)      echo "" ;;
    esac
}

# All tool keys
ALL_TOOLS="enum exec poison"

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${CYAN}[*] $1${NC}"
}

success() {
    echo -e "${GREEN}[+] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[!] $1${NC}"
}

error() {
    echo -e "${RED}[-] $1${NC}"
    exit 1
}

# Get native target triple
get_native_target() {
    case "$OS" in
        Darwin)
            case "$ARCH" in
                x86_64) echo "x86_64-apple-darwin" ;;
                arm64)  echo "aarch64-apple-darwin" ;;
                *)      error "Unsupported architecture: $ARCH" ;;
            esac
            ;;
        Linux)
            case "$ARCH" in
                x86_64) echo "x86_64-unknown-linux-gnu" ;;
                aarch64) echo "aarch64-unknown-linux-gnu" ;;
                *)      error "Unsupported architecture: $ARCH" ;;
            esac
            ;;
        *)
            error "Unsupported OS: $OS"
            ;;
    esac
}

build_exe() {
    local name=$1
    info "Building $name EXE..."

    cargo build -q $CARGO_FLAGS -p "$name" --bin "$name"

    # Determine target directory
    local target_dir="$ROOT/target/$PROFILE"

    cp "$target_dir/$name" "$OUT_DIR/"

    local size=$(du -k "$OUT_DIR/$name" | cut -f1)
    success "$name ($size KB)"
}

build_universal() {
    local name=$1

    if [ "$OS" != "Darwin" ]; then
        warn "Universal binaries only supported on macOS"
        build_exe "$name"
        return
    fi

    info "Building $name universal binary (Intel + ARM)..."

    # Build for Intel
    info "  Building for x86_64-apple-darwin..."
    cargo build -q $CARGO_FLAGS -p "$name" --bin "$name" --target x86_64-apple-darwin

    # Build for ARM
    info "  Building for aarch64-apple-darwin..."
    cargo build -q $CARGO_FLAGS -p "$name" --bin "$name" --target aarch64-apple-darwin

    # Combine with lipo
    info "  Creating universal binary..."
    lipo -create \
        "$ROOT/target/x86_64-apple-darwin/$PROFILE/$name" \
        "$ROOT/target/aarch64-apple-darwin/$PROFILE/$name" \
        -output "$OUT_DIR/$name"

    local size=$(du -k "$OUT_DIR/$name" | cut -f1)
    success "$name (universal, $size KB)"
}

build_tool() {
    local key=$1
    local name=$(get_tool_name "$key")

    if [ -z "$name" ]; then
        error "Unknown tool: $key"
    fi

    if [ "$UNIVERSAL" = true ]; then
        build_universal "$name"
    else
        build_exe "$name"
    fi
}

# Main
echo ""
echo -e "${CYAN}=== CUA-Kit Build ($OS/$ARCH) ===${NC}"

if [ "$CLEAN" = true ]; then
    info "Cleaning..."
    cargo clean -q
    rm -rf "$ROOT/bin"
    success "Done"
    exit 0
fi

if [ "$TEST" = true ]; then
    info "Running tests..."
    # Exclude cua-bof-common on non-Windows (it won't compile)
    cargo test --workspace --exclude cua-bof-common
    exit $?
fi

# Ensure output directory exists
mkdir -p "$OUT_DIR"

# Note about BOF
if [ "$OS" != "MINGW"* ] && [ "$OS" != "CYGWIN"* ]; then
    info "Note: BOF builds are Windows-only. Building EXE format."
fi

# Build
if [ "$TOOL" = "all" ]; then
    for key in $ALL_TOOLS; do
        build_tool "$key"
    done
else
    build_tool "$TOOL"
fi

echo ""
success "Build complete: $OUT_DIR"
ls -la "$OUT_DIR" 2>/dev/null | tail -n +2 | while read line; do
    echo "    $line"
done
