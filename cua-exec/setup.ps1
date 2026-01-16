# Setup script for claude BOF
# This script configures the claude.cna file with the correct path to json.jar

$ErrorActionPreference = "Stop"

# Get the directory where this script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Define file paths
$CnaTemplate = Join-Path $ScriptDir "claude.cna"
$JsonJar = Join-Path $ScriptDir "json.jar"

# Check if json.jar exists
if (-not (Test-Path $JsonJar)) {
    Write-Host "[!] json.jar not found. Downloading from Maven Central..." -ForegroundColor Yellow

    $JsonJarUrl = "https://repo1.maven.org/maven2/org/json/json/20250517/json-20250517.jar"

    try {
        Invoke-WebRequest -Uri $JsonJarUrl -OutFile $JsonJar
        Write-Host "[+] Downloaded json.jar successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "[!] Failed to download json.jar: $_" -ForegroundColor Red
        Write-Host "[!] Please manually download from: $JsonJarUrl" -ForegroundColor Red
        exit 1
    }
}

# Read the CNA template
$CnaContent = Get-Content $CnaTemplate -Raw

# Check if placeholder exists
if ($CnaContent -notmatch '\{\{JSON_JAR_PATH\}\}') {
    Write-Host "[!] Placeholder {{JSON_JAR_PATH}} not found in claude.cna" -ForegroundColor Yellow
    Write-Host "[!] The script may already be configured or the template is invalid" -ForegroundColor Yellow
    exit 1
}

# Replace placeholder with actual path (use forward slashes for Aggressor Script)
$JsonJarPath = $JsonJar -replace '\\', '\\'
$CnaContent = $CnaContent -replace '\{\{JSON_JAR_PATH\}\}', $JsonJarPath

# Write back to the CNA file
Set-Content -Path $CnaTemplate -Value $CnaContent -NoNewline

Write-Host "[+] Setup complete!" -ForegroundColor Green
Write-Host "[+] json.jar location: $JsonJar" -ForegroundColor Green
Write-Host "[+] You can now load claude.cna in Cobalt Strike" -ForegroundColor Green
