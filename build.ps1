# CUA-Kit Build Script
# Usage: .\build.ps1 [-Release] [-Tool <name>] [-Exe|-Bof] [-Test] [-Clean]

param(
    [ValidateSet("all", "enum", "exec", "poison")]
    [string]$Tool = "all",
    [switch]$Release,
    [switch]$Exe,
    [switch]$Bof,
    [switch]$Test,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot
$Profile = if ($Release) { "release" } else { "debug" }
$OutDir = Join-Path (Join-Path $Root "bin") $Profile

# Tool definitions
$Tools = @{
    enum   = "cua-enum"
    exec   = "cua-exec"
    poison = "cua-poison"
}

function Build-Exe($name) {
    Write-Host "[*] Building $name EXE..." -ForegroundColor Cyan
    $flags = if ($Release) { "--release" } else { "" }

    Invoke-Expression "cargo build -q $flags -p $name --bin $name --target x86_64-pc-windows-msvc"
    if ($LASTEXITCODE -ne 0) { throw "$name EXE build failed" }

    $src = Join-Path $Root "target/x86_64-pc-windows-msvc/$Profile/$name.exe"
    Copy-Item $src $OutDir -Force

    $size = [math]::Round((Get-Item "$OutDir/$name.exe").Length / 1KB, 2)
    Write-Host "[+] $name.exe ($size KB)" -ForegroundColor Green
}

function Build-Bof($name) {
    # BOFs always build as release - debug BOFs don't work with COFFLoader
    Write-Host "[*] Building $name BOF (release)..." -ForegroundColor Cyan

    Invoke-Expression "cargo rustc -q --release -p $name --features bof --lib --crate-type=staticlib --target x86_64-pc-windows-msvc -- --emit=obj -C panic=abort -C opt-level=z"
    if ($LASTEXITCODE -ne 0) { throw "$name BOF build failed" }

    $objName = $name -replace "-", "_"
    $obj = Get-ChildItem "target/x86_64-pc-windows-msvc/release/deps/$objName*.o" | Select-Object -First 1
    if (-not $obj) { throw "Object file not found for $name" }

    $dest = "$OutDir/$name.x64.o"
    Copy-Item $obj.FullName $dest -Force

    $size = [math]::Round((Get-Item $dest).Length / 1KB, 2)
    Write-Host "[+] $name.x64.o ($size KB)" -ForegroundColor Green

    if ((Get-Item $dest).Length -gt 1MB) {
        Write-Host "[!] WARNING: BOF exceeds 1MB limit" -ForegroundColor Yellow
    }
}

function Build-Tool($key) {
    $name = $Tools[$key]
    if ($Exe -or (-not $Exe -and -not $Bof)) { Build-Exe $name }
    if ($Bof -or (-not $Exe -and -not $Bof)) { Build-Bof $name }
}

# Main
Write-Host "`n=== CUA-Kit Build ===" -ForegroundColor Cyan

if ($Clean) {
    Write-Host "[*] Cleaning..." -ForegroundColor Cyan
    cargo clean -q
    Remove-Item -Recurse -Force (Join-Path $Root "bin") -ErrorAction SilentlyContinue
    Write-Host "[+] Done" -ForegroundColor Green
    exit 0
}

if ($Test) {
    Write-Host "[*] Running tests..." -ForegroundColor Cyan
    cargo test --workspace
    exit $LASTEXITCODE
}

# Ensure output directory exists
New-Item -ItemType Directory -Path $OutDir -Force | Out-Null

# Build
if ($Tool -eq "all") {
    foreach ($key in $Tools.Keys) { Build-Tool $key }
} else {
    Build-Tool $Tool
}

Write-Host "`n[+] Build complete: $OutDir" -ForegroundColor Green
Get-ChildItem $OutDir | ForEach-Object {
    $size = [math]::Round($_.Length / 1KB, 2)
    Write-Host "    $($_.Name) ($size KB)"
}
