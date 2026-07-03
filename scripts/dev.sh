#!/usr/bin/env bash
# FlowLocal — Linux Dev Script
# Usage: ./scripts/dev.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="$ROOT/.venvs"
SERVICES_DIR="$ROOT/services"

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${CYAN}[FlowLocal Dev] Starting all services...${NC}"

# Track PIDs for cleanup
PIDS=()

cleanup() {
    echo -e "\n${YELLOW}[FlowLocal Dev] Shutting down services...${NC}"
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
    done
    wait 2>/dev/null || true
    echo -e "${GREEN}[FlowLocal Dev] Done.${NC}"
}
trap cleanup EXIT INT TERM

# ──────────────────────────────────────────────────────────
# Start Python services
# ──────────────────────────────────────────────────────────
echo -e "${GREEN}  Starting Whisper service...${NC}"
"$VENV_DIR/whisper/bin/python" -m flowlocal_whisper.main &
PIDS+=($!)

echo -e "${GREEN}  Starting LLM service...${NC}"
"$VENV_DIR/llm/bin/python" -m flowlocal_llm.main &
PIDS+=($!)

echo -e "${GREEN}  Starting RAG service...${NC}"
"$VENV_DIR/rag/bin/python" -m flowlocal_rag.main &
PIDS+=($!)

# Give services time to initialize
sleep 3

# ──────────────────────────────────────────────────────────
# Start Tauri dev
# ──────────────────────────────────────────────────────────
echo -e "\n${CYAN}  Starting Tauri dev app...${NC}"
cd "$ROOT/apps/desktop/src"
cargo tauri dev
