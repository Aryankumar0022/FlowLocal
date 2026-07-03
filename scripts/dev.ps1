# FlowLocal — Windows Dev Script
# Runs all services concurrently in development mode
# Usage: .\scripts\dev.ps1

$ErrorActionPreference = "Stop"
$ROOT = Split-Path -Parent $PSScriptRoot
$VENV_DIR = Join-Path $ROOT ".venvs"
$SERVICES_DIR = Join-Path $ROOT "services"

Write-Host "`n[FlowLocal Dev] Starting all services..." -ForegroundColor Cyan

# ──────────────────────────────────────────────────────────
# Start Python services as background jobs
# ──────────────────────────────────────────────────────────
$whisperJob = Start-Job -Name "whisper" -ScriptBlock {
    param($root, $venv, $svc)
    & "$venv\Scripts\python" -m flowlocal_whisper.main
} -ArgumentList $ROOT, (Join-Path $VENV_DIR "whisper"), (Join-Path $SERVICES_DIR "whisper")

Write-Host "  ✓ Whisper service starting (PID: $($whisperJob.Id))" -ForegroundColor Green

$llmJob = Start-Job -Name "llm" -ScriptBlock {
    param($root, $venv, $svc)
    & "$venv\Scripts\python" -m flowlocal_llm.main
} -ArgumentList $ROOT, (Join-Path $VENV_DIR "llm"), (Join-Path $SERVICES_DIR "llm")

Write-Host "  ✓ LLM service starting (PID: $($llmJob.Id))" -ForegroundColor Green

$ragJob = Start-Job -Name "rag" -ScriptBlock {
    param($root, $venv, $svc)
    & "$venv\Scripts\python" -m flowlocal_rag.main
} -ArgumentList $ROOT, (Join-Path $VENV_DIR "rag"), (Join-Path $SERVICES_DIR "rag")

Write-Host "  ✓ RAG service starting (PID: $($ragJob.Id))" -ForegroundColor Green

# Wait for services to be ready
Start-Sleep -Seconds 3

# ──────────────────────────────────────────────────────────
# Start Tauri dev server (foreground — blocks until Ctrl+C)
# ──────────────────────────────────────────────────────────
Write-Host "`n  Starting Tauri dev app..." -ForegroundColor Cyan

Set-Location (Join-Path $ROOT "apps\desktop\src")

try {
    cargo tauri dev
} finally {
    Write-Host "`n[FlowLocal Dev] Shutting down services..." -ForegroundColor Yellow
    Stop-Job -Job $whisperJob, $llmJob, $ragJob -ErrorAction SilentlyContinue
    Remove-Job -Job $whisperJob, $llmJob, $ragJob -Force -ErrorAction SilentlyContinue
    Write-Host "[FlowLocal Dev] Done." -ForegroundColor Green
}
