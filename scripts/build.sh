#!/usr/bin/env bash
# FlowLocal — Linux Build Script
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "[FlowLocal Build] Building production bundle..."
cd "$ROOT/apps/desktop/src"
cargo tauri build
echo "[FlowLocal Build] Done. Bundle: $ROOT/apps/desktop/src-tauri/target/release/bundle/"
