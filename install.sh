#!/bin/sh
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m' # No Color

REPO="kalebhenrique/neo-mangal"
BIN_DIR="${HOME}/.local/bin"

printf "${CYAN}${BOLD}⚡ neo-mangal Installer & Updater${NC}\n"

# 1. Detect OS
OS_TYPE="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$OS_TYPE" in
    darwin*)
        OS="apple-darwin"
        ;;
    linux*)
        OS="unknown-linux-gnu"
        ;;
    *)
        printf "${RED}Unsupported Operating System: %s${NC}\n" "$OS_TYPE"
        exit 1
        ;;
esac

# 2. Detect Architecture
ARCH_TYPE="$(uname -m)"
case "$ARCH_TYPE" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    arm64|aarch64)
        ARCH="aarch64"
        ;;
    *)
        printf "${RED}Unsupported Architecture: %s${NC}\n" "$ARCH_TYPE"
        exit 1
        ;;
esac

TARGET="${ARCH}-${OS}"
printf "Detected platform: ${BOLD}%s${NC}\n" "$TARGET"

# 3. Create destination directory
mkdir -p "$BIN_DIR"

# 4. Fetch latest release from GitHub
TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'neo-mangal')"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

printf "Checking latest release from GitHub (%s)...\n" "$REPO"
LATEST_TAG="$(curl -sSL -H "Accept: application/vnd.github.v3+json" "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)"

DOWNLOAD_SUCCESS=false

if [ -n "$LATEST_TAG" ]; then
    ARCHIVE_NAME="neo-mangal-${LATEST_TAG}-${TARGET}.tar.gz"
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARCHIVE_NAME}"

    printf "Downloading %s (${BOLD}%s${NC})...\n" "$ARCHIVE_NAME" "$LATEST_TAG"
    if curl -f -sSL "$DOWNLOAD_URL" -o "${TMP_DIR}/${ARCHIVE_NAME}" 2>/dev/null; then
        tar -xzf "${TMP_DIR}/${ARCHIVE_NAME}" -C "$TMP_DIR"
        if [ -f "${TMP_DIR}/neo-mangal" ]; then
            mv "${TMP_DIR}/neo-mangal" "${BIN_DIR}/neo-mangal"
            DOWNLOAD_SUCCESS=true
        fi
    fi
fi

# Fallback: If no binary release asset found yet, build with cargo if available
if [ "$DOWNLOAD_SUCCESS" = false ]; then
    if command -v cargo >/dev/null 2>&1; then
        printf "${YELLOW}Pre-compiled binary for ${TARGET} not found on GitHub Releases yet.${NC}\n"
        printf "Building and installing via cargo from repository source...\n"
        cargo install --git "https://github.com/${REPO}.git" --root "${HOME}/.local" --force
        DOWNLOAD_SUCCESS=true
    else
        printf "${RED}Error: Could not download pre-built binary and 'cargo' was not found on your system.${NC}\n"
        printf "Please install Rust (https://rustup.rs) or check https://github.com/%s/releases\n" "$REPO"
        exit 1
    fi
fi

chmod +x "${BIN_DIR}/neo-mangal"

# Create symlinks for nmangal and neomangal
ln -sf "${BIN_DIR}/neo-mangal" "${BIN_DIR}/nmangal"
ln -sf "${BIN_DIR}/neo-mangal" "${BIN_DIR}/neomangal"

printf "\n${GREEN}${BOLD}✓ neo-mangal installed successfully in %s!${NC}\n" "$BIN_DIR"
printf "Created shortcuts: ${BOLD}nmangal${NC}, ${BOLD}neomangal${NC}, ${BOLD}neo-mangal${NC}\n\n"

# 5. Check if ~/.local/bin is in PATH
case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
        printf "${YELLOW}Note: %s is not currently in your PATH.${NC}\n" "$BIN_DIR"
        printf "Add it to your shell configuration (e.g. ~/.zshrc or ~/.bashrc):\n"
        printf "  export PATH=\"\$HOME/.local/bin:\$PATH\"\n\n"
        ;;
esac

printf "Run ${CYAN}${BOLD}nmangal${NC} to start!\n"
