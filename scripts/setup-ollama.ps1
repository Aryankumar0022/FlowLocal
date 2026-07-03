# FlowLocal — Ollama Setup Script (Windows)
# Downloads required models for FlowLocal

$ErrorActionPreference = "Stop"

Write-Host "`n[FlowLocal] Setting up Ollama models..." -ForegroundColor Cyan

$models = @(
    "qwen3:4b",
    "nomic-embed-text"
)

foreach ($model in $models) {
    Write-Host "  Pulling $model..." -ForegroundColor Yellow
    ollama pull $model
    Write-Host "  ✓ $model ready" -ForegroundColor Green
}

Write-Host "`n[FlowLocal] All models ready!" -ForegroundColor Cyan
Write-Host "  Run 'ollama list' to verify." -ForegroundColor Gray
