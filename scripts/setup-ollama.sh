#!/usr/bin/env bash
# FlowLocal — Ollama Setup Script (Linux)
set -euo pipefail

echo "[FlowLocal] Setting up Ollama models..."

for model in "qwen3:4b" "nomic-embed-text"; do
    echo "  Pulling $model..."
    ollama pull "$model"
    echo "  ✓ $model ready"
done

echo "[FlowLocal] All models ready! Run 'ollama list' to verify."
