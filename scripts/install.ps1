# FlowLocal — Windows Installation Script
# Run as: .\scripts\install.ps1
# Requires PowerShell 7+ and internet connection for initial setup

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$ROOT = Split-Path -Parent $PSScriptRoot
$DATA_DIR = Join-Path $ROOT "data"
$SERVICES_DIR = Join-Path $ROOT "services"
$VENV_DIR = Join-Path $ROOT ".venvs"

Write-Host "`n====================================" -ForegroundColor Cyan
Write-Host " FlowLocal — Installation" -ForegroundColor Cyan
Write-Host "====================================`n" -ForegroundColor Cyan

# ──────────────────────────────────────────────────────────
# 1. Check prerequisites
# ──────────────────────────────────────────────────────────
function Test-Command($name) {
    return $null -ne (Get-Command $name -ErrorAction SilentlyContinue)
}

Write-Host "[1/7] Checking prerequisites..." -ForegroundColor Yellow

if (-not (Test-Command "rustup")) {
    Write-Host "  ✗ Rust not found. Install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}
Write-Host "  ✓ Rust: $(rustc --version)" -ForegroundColor Green

if (-not (Test-Command "node")) {
    Write-Host "  ✗ Node.js not found. Install from https://nodejs.org/" -ForegroundColor Red
    exit 1
}
Write-Host "  ✓ Node: $(node --version)" -ForegroundColor Green

if (-not (Test-Command "python")) {
    Write-Host "  ✗ Python 3.11+ not found. Install from https://python.org/" -ForegroundColor Red
    exit 1
}
$pyVersion = python --version 2>&1
Write-Host "  ✓ Python: $pyVersion" -ForegroundColor Green

if (-not (Test-Command "ollama")) {
    Write-Host "  ⚠ Ollama not found. Run .\scripts\setup-ollama.ps1 after install." -ForegroundColor Yellow
} else {
    Write-Host "  ✓ Ollama: $(ollama --version)" -ForegroundColor Green
}

# ──────────────────────────────────────────────────────────
# 2. Create data directories
# ──────────────────────────────────────────────────────────
Write-Host "`n[2/7] Creating data directories..." -ForegroundColor Yellow

@("$DATA_DIR\models", "$DATA_DIR\chroma", "$DATA_DIR\db") | ForEach-Object {
    if (-not (Test-Path $_)) {
        New-Item -ItemType Directory -Path $_ -Force | Out-Null
        Write-Host "  ✓ Created: $_" -ForegroundColor Green
    }
}

# ──────────────────────────────────────────────────────────
# 3. Install Rust target + Tauri CLI
# ──────────────────────────────────────────────────────────
Write-Host "`n[3/7] Setting up Rust toolchain..." -ForegroundColor Yellow

rustup update stable 2>&1 | Out-Null
Write-Host "  ✓ Rust toolchain updated" -ForegroundColor Green

if (-not (Test-Command "cargo-tauri")) {
    Write-Host "  Installing Tauri CLI..." -ForegroundColor Gray
    cargo install tauri-cli --version "^2" 2>&1 | Out-Null
}
Write-Host "  ✓ Tauri CLI ready" -ForegroundColor Green

# ──────────────────────────────────────────────────────────
# 4. Install Node dependencies
# ──────────────────────────────────────────────────────────
Write-Host "`n[4/7] Installing Node dependencies..." -ForegroundColor Yellow

Set-Location (Join-Path $ROOT "apps\desktop\src")
npm install --prefer-offline 2>&1 | Out-Null
Write-Host "  ✓ Node modules installed" -ForegroundColor Green
Set-Location $ROOT

# ──────────────────────────────────────────────────────────
# 5. Create Python virtual environments
# ──────────────────────────────────────────────────────────
Write-Host "`n[5/7] Setting up Python environments..." -ForegroundColor Yellow

if (-not (Test-Path $VENV_DIR)) {
    New-Item -ItemType Directory -Path $VENV_DIR -Force | Out-Null
}

@("whisper", "llm", "rag") | ForEach-Object {
    $svc = $_
    $venv = Join-Path $VENV_DIR $svc
    $svcDir = Join-Path $SERVICES_DIR $svc

    Write-Host "  Setting up $svc service..." -ForegroundColor Gray

    if (-not (Test-Path $venv)) {
        python -m venv $venv 2>&1 | Out-Null
    }

    # Install shared lib first
    & "$venv\Scripts\pip" install -e (Join-Path $SERVICES_DIR "shared") --quiet

    # Install service
    & "$venv\Scripts\pip" install -e $svcDir --quiet

    Write-Host "  ✓ $svc venv ready" -ForegroundColor Green
}

# ──────────────────────────────────────────────────────────
# 6. Copy .env.example → .env
# ──────────────────────────────────────────────────────────
Write-Host "`n[6/7] Configuring environment..." -ForegroundColor Yellow

if (-not (Test-Path (Join-Path $ROOT ".env"))) {
    Copy-Item (Join-Path $ROOT ".env.example") (Join-Path $ROOT ".env")
    Write-Host "  ✓ .env created from .env.example" -ForegroundColor Green
} else {
    Write-Host "  ✓ .env already exists — skipping" -ForegroundColor Gray
}

# ──────────────────────────────────────────────────────────
# 7. Done
# ──────────────────────────────────────────────────────────
Write-Host "`n====================================" -ForegroundColor Cyan
Write-Host " Installation complete!" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor White
Write-Host "  1. Run: .\scripts\setup-ollama.ps1    (download AI models)" -ForegroundColor Gray
Write-Host "  2. Run: .\scripts\dev.ps1              (start development)" -ForegroundColor Gray
Write-Host "  3. Run: .\scripts\build.ps1            (build production)" -ForegroundColor Gray
Write-Host ""
