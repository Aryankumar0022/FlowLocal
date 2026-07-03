# ============================================================
# setup_dev.ps1 — FlowLocal Development Bootstrap Script
# 
# Usage:
#   .\setup_dev.ps1
#
# What it does:
# 1. Checks prerequisites (Node, Rust, uv, Ollama)
# 2. Installs frontend dependencies (npm install)
# 3. Creates a Python virtual environment via uv
# 4. Installs all Python backend dependencies
# 5. Pulls required Ollama models
# ============================================================

$ErrorActionPreference = "Stop"

Write-Host "🚀 FlowLocal Development Bootstrap" -ForegroundColor Cyan
Write-Host "=================================="

# 1. Check Prereqs
Write-Host "`n[1/5] Checking prerequisites..." -ForegroundColor Yellow

$prereqs = @(
    @{ Name = "node"; Command = "node --version"; ErrorMsg = "Node.js not found. Please install it." }
    @{ Name = "npm"; Command = "npm --version"; ErrorMsg = "npm not found." }
    @{ Name = "cargo"; Command = "cargo --version"; ErrorMsg = "Rust/cargo not found. Please install Rust." }
    @{ Name = "uv"; Command = "uv --version"; ErrorMsg = "uv not found. Please install it: pip install uv" }
    @{ Name = "ollama"; Command = "ollama --version"; ErrorMsg = "Ollama not found. Please install it." }
)

foreach ($p in $prereqs) {
    try {
        Invoke-Expression $p.Command | Out-Null
        Write-Host "  $($p.Name) ... OK" -ForegroundColor Green
    } catch {
        Write-Host "  ❌ $($p.ErrorMsg)" -ForegroundColor Red
        exit 1
    }
}

# 2. Frontend Dependencies
Write-Host "`n[2/5] Installing frontend dependencies..." -ForegroundColor Yellow
Set-Location "apps/desktop"
npm install
Set-Location "../.."

# 3. Python Environment
Write-Host "`n[3/5] Setting up Python environment..." -ForegroundColor Yellow
Set-Location "services"
if (-not (Test-Path ".venv")) {
    Write-Host "  Creating virtual environment with uv..."
    uv venv
} else {
    Write-Host "  .venv already exists."
}

# 4. Python Dependencies
Write-Host "`n[4/5] Installing Python dependencies..." -ForegroundColor Yellow
$deps = @(
    "faster-whisper",
    "ollama",
    "chromadb",
    "jinja2",
    "scipy",
    "numpy<2.0.0",
    "typing_extensions"
)
Write-Host "  Running uv pip install..."
uv pip install $deps
Set-Location ".."

# 5. Ollama Models
Write-Host "`n[5/5] Pulling Ollama models..." -ForegroundColor Yellow
Write-Host "  Pulling cleanup model (qwen2.5:3b)..."
ollama pull qwen2.5:3b
Write-Host "  Pulling embedding model (nomic-embed-text)..."
ollama pull nomic-embed-text

Write-Host "`n✅ Bootstrap complete!" -ForegroundColor Green
Write-Host "`nTo start developing:" -ForegroundColor Cyan
Write-Host "  cd apps/desktop"
Write-Host "  npm run tauri dev"
