# FlowLocal — Windows Build Script
# Produces: apps/desktop/src-tauri/target/release/bundle/

$ErrorActionPreference = "Stop"
$ROOT = Split-Path -Parent $PSScriptRoot

Write-Host "`n[FlowLocal Build] Building production bundle..." -ForegroundColor Cyan

# Build Tauri (compiles Rust + bundles React)
Set-Location (Join-Path $ROOT "apps\desktop\src")
cargo tauri build

$bundleDir = Join-Path $ROOT "apps\desktop\src-tauri\target\release\bundle"
Write-Host "`n[FlowLocal Build] Bundle output: $bundleDir" -ForegroundColor Green
Write-Host "[FlowLocal Build] Done." -ForegroundColor Green
