#!/bin/sh
# CUV (C-Ultra-Velocity) Universal Installer for macOS and Linux
# Usage: curl -LsSf https://raw.githubusercontent.com/ARASKOVA-labs/CUV/main/scripts/install.sh | sh

set -e

# Terminal styling
BOLD="$(tput bold 2>/dev/null || echo '')"
GREEN="$(tput setaf 2 2>/dev/null || echo '')"
CYAN="$(tput setaf 6 2>/dev/null || echo '')"
YELLOW="$(tput setaf 3 2>/dev/null || echo '')"
RESET="$(tput sgr0 2>/dev/null || echo '')"

banner() {
    cat << "EOF"
  ⚡ CUV (C-Ultra-Velocity)
  The "uv" and "bun" for C and C++
EOF
}

detect_target() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Darwin)
            case "$ARCH" in
                arm64|aarch64)
                    TARGET="aarch64-apple-darwin"
                    ;;
                x86_64)
                    TARGET="x86_64-apple-darwin"
                    ;;
                *)
                    echo "Unsupported architecture: $ARCH on macOS"
                    exit 1
                    ;;
            esac
            ;;
        Linux)
            case "$ARCH" in
                x86_64)
                    TARGET="x86_64-unknown-linux-gnu"
                    ;;
                aarch64|arm64)
                    TARGET="aarch64-unknown-linux-gnu"
                    ;;
                *)
                    echo "Unsupported architecture: $ARCH on Linux"
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo "Unsupported operating system: $OS"
            echo "For Windows, install via PowerShell:"
            echo '  powershell -c "irm https://raw.githubusercontent.com/ARASKOVA-labs/CUV/main/scripts/install.ps1 | iex"'
            exit 1
            ;;
    esac
}

main() {
    banner
    detect_target

    INSTALL_DIR="${CUV_INSTALL_DIR:-$HOME/.cuv/bin}"
    REPO="ARASKOVA-labs/CUV"
    TAG="${CUV_VERSION:-latest}"

    if [ "$TAG" = "latest" ]; then
        URL="https://github.com/${REPO}/releases/latest/download/cuv-${TARGET}.tar.gz"
    else
        URL="https://github.com/${REPO}/releases/download/${TAG}/cuv-${TARGET}.tar.gz"
    fi

    echo ""
    echo "${CYAN}Target detected:${RESET} ${BOLD}${TARGET}${RESET}"
    echo "${CYAN}Installing to:${RESET}   ${BOLD}${INSTALL_DIR}/cuv${RESET}"
    echo ""

    mkdir -p "$INSTALL_DIR"
    TMP_DIR="$(mktemp -d)"

    cleanup() {
        rm -rf "$TMP_DIR"
    }
    trap cleanup EXIT

    echo "${YELLOW}Downloading CUV release archive...${RESET}"
    if command -v curl >/dev/null 2>&1; then
        curl -fSL "$URL" -o "$TMP_DIR/cuv.tar.gz"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$TMP_DIR/cuv.tar.gz" "$URL"
    else
        echo "Error: curl or wget is required to install CUV."
        exit 1
    fi

    tar -xzf "$TMP_DIR/cuv.tar.gz" -C "$TMP_DIR"
    cp "$TMP_DIR/cuv" "$INSTALL_DIR/cuv"
    chmod +x "$INSTALL_DIR/cuv"

    echo "${GREEN}${BOLD}✔ Successfully installed CUV!${RESET}"

    # Configure Shell PATH
    SHELL_PROFILE=""
    if [ -n "$ZSH_VERSION" ] || [ "$(basename "$SHELL")" = "zsh" ]; then
        SHELL_PROFILE="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ] || [ "$(basename "$SHELL")" = "bash" ]; then
        if [ -f "$HOME/.bashrc" ]; then
            SHELL_PROFILE="$HOME/.bashrc"
        elif [ -f "$HOME/.bash_profile" ]; then
            SHELL_PROFILE="$HOME/.bash_profile"
        fi
    fi

    if [ -z "$SHELL_PROFILE" ]; then
        SHELL_PROFILE="$HOME/.profile"
    fi

    PATH_ENTRY="export PATH=\"$INSTALL_DIR:\$PATH\""
    if ! grep -q "$INSTALL_DIR" "$SHELL_PROFILE" 2>/dev/null; then
        echo "" >> "$SHELL_PROFILE"
        echo "# CUV (C-Ultra-Velocity) Path" >> "$SHELL_PROFILE"
        echo "$PATH_ENTRY" >> "$SHELL_PROFILE"
        echo "${CYAN}Added CUV to PATH in ${BOLD}${SHELL_PROFILE}${RESET}"
    fi

    echo ""
    echo "${BOLD}To start using CUV, run:${RESET}"
    echo "  ${CYAN}source ${SHELL_PROFILE}${RESET}"
    echo "  ${CYAN}cuv --help${RESET}"
    echo ""
    echo "${GREEN}Welcome to ultra-velocity C/C++ development! ⚡${RESET}"
}

main "$@"
