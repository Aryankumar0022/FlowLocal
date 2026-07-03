#!/usr/bin/env bash
# FlowLocal — Linux Installation Script
# Run as: chmod +x scripts/install.sh && ./scripts/install.sh

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA_DIR="$ROOT/data"
SERVICES_DIR="$ROOT/services"
VENV_DIR="$ROOT/.venvs"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "\n${CYAN}====================================${NC}"
echo -e "${CYAN} FlowLocal — Installation${NC}"
echo -e "${CYAN}====================================${NC}\n"

# ──────────────────────────────────────────────────────────
# Helper
# ──────────────────────────────────────────────────────────
check_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# ──────────────────────────────────────────────────────────
# 1. Check prerequisites
# ──────────────────────────────────────────────────────────
echo -e "${YELLOW}[1/7] Checking prerequisites...${NC}"

if ! check_cmd rustup; then
    echo -e "  ${RED}✗ Rust not found. Install from https://rustup.rs/${NC}"
    exit 1
fi
echo -e "  ${GREEN}✓ Rust: $(rustc --version)${NC}"

if ! check_cmd node; then
    echo -e "  ${RED}✗ Node.js not found. Install v20+ from https://nodejs.org/${NC}"
    exit 1
fi
echo -e "  ${GREEN}✓ Node: $(node --version)${NC}"

if ! check_cmd python3; then
    echo -e "  ${RED}✗ Python 3.11+ not found.${NC}"
    exit 1
fi
echo -e "  ${GREEN}✓ Python: $(python3 --version)${NC}"

if ! check_cmd ollama; then
    echo -e "  ${YELLOW}⚠ Ollama not found. Run ./scripts/setup-ollama.sh after install.${NC}"
else
    echo -e "  ${GREEN}✓ Ollama: $(ollama --version)${NC}"
fi

# Check for xdotool (text injection on Linux)
if ! check_cmd xdotool; then
    echo -e "  ${YELLOW}⚠ xdotool not found. Installing...${NC}"
    if check_cmd apt-get; then
        sudo apt-get install -y xdotool >/dev/null 2>&1
    elif check_cmd pacman; then
        sudo pacman -S --noconfirm xdotool >/dev/null 2>&1
    elif check_cmd dnf; then
        sudo dnf install -y xdotool >/dev/null 2>&1
    fi
fi
echo -e "  ${GREEN}✓ xdotool available${NC}"

# ──────────────────────────────────────────────────────────
# 2. Create data directories
# ──────────────────────────────────────────────────────────
echo -e "\n${YELLOW}[2/7] Creating data directories...${NC}"

for dir in "$DATA_DIR/models" "$DATA_DIR/chroma" "$DATA_DIR/db"; do
    mkdir -p "$dir"
    echo -e "  ${GREEN}✓ $dir${NC}"
done

# ──────────────────────────────────────────────────────────
# 3. Rust toolchain
# ──────────────────────────────────────────────────────────
echo -e "\n${YELLOW}[3/7] Setting up Rust toolchain...${NC}"

rustup update stable >/dev/null 2>&1
echo -e "  ${GREEN}✓ Rust updated${NC}"

if ! check_cmd cargo-tauri; then
    cargo install tauri-cli --version "^2" >/dev/null 2>&1
fi
echo -e "  ${GREEN}✓ Tauri CLI ready${NC}"

# ──────────────────────────────────────────────────────────
# 4. Node dependencies
# ──────────────────────────────────────────────────────────
echo -e "\n${YELLOW}[4/7] Installing Node dependencies...${NC}"

cd "$ROOT/apps/desktop/src"
npm install --prefer-offline >/dev/null 2>&1
cd "$ROOT"
echo -e "  ${GREEN}✓ Node modules installed${NC}"

# ──────────────────────────────────────────────────────────
# 5. Python virtual environments
# ──────────────────────────────────────────────────────────
echo -e "\n${YELLOW}[5/7] Setting up Python environments...${NC}"

mkdir -p "$VENV_DIR"

for svc in whisper llm rag; do
    echo -e "  Setting up $svc service..."
    venv="$VENV_DIR/$svc"
    svc_dir="$SERVICES_DIR/$svc"

    if [ ! -d "$venv" ]; then
        python3 -m venv "$venv" >/dev/null 2>&1
    fi

    "$venv/bin/pip" install -e "$SERVICES_DIR/shared" --quiet
    "$venv/bin/pip" install -e "$svc_dir" --quiet
    echo -e "  ${GREEN}✓ $svc venv ready${NC}"
done

# ──────────────────────────────────────────────────────────
# 6. .env setup
# ──────────────────────────────────────────────────────────
echo -e "\n${YELLOW}[6/7] Configuring environment...${NC}"

if [ ! -f "$ROOT/.env" ]; then
    cp "$ROOT/.env.example" "$ROOT/.env"
    echo -e "  ${GREEN}✓ .env created from .env.example${NC}"
else
    echo -e "  .env already exists — skipping"
fi

# ──────────────────────────────────────────────────────────
# 7. Done
# ──────────────────────────────────────────────────────────
echo -e "\n${CYAN}====================================${NC}"
echo -e "${CYAN} Installation complete!${NC}"
echo -e "${CYAN}====================================${NC}\n"
echo "Next steps:"
echo "  1. ./scripts/setup-ollama.sh    # download AI models"
echo "  2. ./scripts/dev.sh             # start development"
echo "  3. ./scripts/build.sh           # build production"
echo ""
