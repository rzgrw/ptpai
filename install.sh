#!/usr/bin/env bash
set -e

# ==============================================================================
# PTPAI (⚡) Universal Installer for macOS and Ubuntu Linux
# ==============================================================================

BOLD='\033[1m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BOLD}${BLUE}================================================================${NC}"
echo -e "${BOLD}${BLUE}          PTPAI (⚡) — Universal Installer (macOS & Ubuntu)     ${NC}"
echo -e "${BOLD}${BLUE}================================================================${NC}"

OS="$(uname -s)"
ARCH="$(uname -m)"

echo -e "${CYAN}• Detecting Operating System...${NC} $OS ($ARCH)"

case "$OS" in
    Darwin)
        echo -e "${GREEN}✓ Platform: macOS (Apple Darwin)${NC}"
        if [ "$ARCH" = "arm64" ]; then
            CHIP=$(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo "Apple Silicon")
            MEM_BYTES=$(sysctl -n hw.memsize 2>/dev/null || echo "0")
            MEM_GB=$((MEM_BYTES / 1024 / 1024 / 1024))
            echo -e "  -> Silicon: ${BOLD}${CHIP}${NC}"
            echo -e "  -> Unified Memory: ${BOLD}${MEM_GB} GB UMA${NC} (Zero-copy Metal acceleration active)"
        fi
        ;;
    Linux)
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            echo -e "${GREEN}✓ Platform: Linux ($NAME $VERSION_ID)${NC}"
        else
            echo -e "${GREEN}✓ Platform: Linux${NC}"
        fi

        # Check for NVIDIA CUDA GPU
        if command -v nvidia-smi &> /dev/null; then
            GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader | head -n 1)
            GPU_MEM=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits | head -n 1)
            echo -e "  -> NVIDIA GPU Detected: ${BOLD}${GPU_NAME}${NC} (${GPU_MEM} MB VRAM)"
            echo -e "  -> CUDA Acceleration: Active"
        fi
        ;;
    *)
        echo -e "${YELLOW}Warning: Untested operating system: $OS. Attempting standard build...${NC}"
        ;;
esac

# Check for Rust / Cargo toolchain
if ! command -v cargo &> /dev/null; then
    echo -e "\n${YELLOW}• Rust compiler (cargo) not found. Installing Rust toolchain via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo -e "\n${CYAN}• Compiling optimized production binary with native vector kernels...${NC}"
cargo build --release

INSTALL_DIR="/usr/local/bin"
BINARY_NAME="ptpai"

echo -e "\n${CYAN}• Installing binary to ${INSTALL_DIR}/${BINARY_NAME}...${NC}"

if [ -w "$INSTALL_DIR" ]; then
    cp target/release/ptpai "${INSTALL_DIR}/${BINARY_NAME}"
else
    echo -e "  (Requesting sudo privileges to install to ${INSTALL_DIR})"
    sudo cp target/release/ptpai "${INSTALL_DIR}/${BINARY_NAME}"
fi

chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo -e "\n${GREEN}${BOLD}✓ Installation Successful!${NC}"
echo -e "${BOLD}================================================================${NC}"
echo -e "Quickstart commands:"
echo -e "  ${CYAN}ptpai tracker --port 8080${NC}   Launch Swarm Tracker & qBittorrent WebUI"
echo -e "  ${CYAN}ptpai seed${NC}                  Auto-detect hardware & start seeding compute"
echo -e "  ${CYAN}ptpai benchmark${NC}             Measure local memory bandwidth & SIMD TFLOPS"
echo -e "  ${CYAN}ptpai run --prompt \"...\"${NC}    Request distributed inference from the swarm"
echo -e "${BOLD}================================================================${NC}"
