#!/usr/bin/env bash
# ccboard installer - Downloads and installs latest release binary
# Usage: curl -sSL https://raw.githubusercontent.com/FlorianBruniaux/ccboard/main/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Config
REPO="FlorianBruniaux/ccboard"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)
        OS_TYPE="linux"
        ;;
    Darwin*)
        OS_TYPE="macos"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        OS_TYPE="windows"
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_TYPE="x86_64"
        ;;
    aarch64|arm64)
        ARCH_TYPE="aarch64"
        ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}ccboard installer${NC}"
echo "OS: $OS_TYPE"
echo "Architecture: $ARCH_TYPE"
echo "Install directory: $INSTALL_DIR"
echo ""

# Get latest release version
echo -e "${YELLOW}Fetching latest release...${NC}"
LATEST_VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')

if [ -z "$LATEST_VERSION" ]; then
    echo -e "${RED}Failed to fetch latest version${NC}"
    exit 1
fi

echo "Latest version: v$LATEST_VERSION"

# Construct download URL
if [ "$OS_TYPE" = "windows" ]; then
    ASSET_NAME="ccboard-windows-${ARCH_TYPE}.exe.zip"
    BINARY_NAME="ccboard-windows-${ARCH_TYPE}.exe"
else
    ASSET_NAME="ccboard-${OS_TYPE}-${ARCH_TYPE}.tar.gz"
    BINARY_NAME="ccboard"
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/v$LATEST_VERSION/$ASSET_NAME"

echo "Downloading: $DOWNLOAD_URL"

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

cd "$TMP_DIR"

# Download
if ! curl -sL "$DOWNLOAD_URL" -o "$ASSET_NAME"; then
    echo -e "${RED}Failed to download $ASSET_NAME${NC}"
    echo "URL: $DOWNLOAD_URL"
    exit 1
fi

# Extract
echo -e "${YELLOW}Extracting...${NC}"
if [ "$OS_TYPE" = "windows" ]; then
    unzip -q "$ASSET_NAME"
else
    tar xzf "$ASSET_NAME"
fi

# Create install directory if needed
mkdir -p "$INSTALL_DIR"

# Install binary
echo -e "${YELLOW}Installing to $INSTALL_DIR...${NC}"
if [ "$OS_TYPE" = "windows" ]; then
    mv "$BINARY_NAME" "$INSTALL_DIR/ccboard.exe"
    INSTALLED_PATH="$INSTALL_DIR/ccboard.exe"
else
    mv ccboard "$INSTALL_DIR/ccboard"
    chmod +x "$INSTALL_DIR/ccboard"
    INSTALLED_PATH="$INSTALL_DIR/ccboard"
fi

echo -e "${GREEN}✓ ccboard v$LATEST_VERSION installed successfully!${NC}"
echo ""
echo "Binary location: $INSTALLED_PATH"
echo ""

# Check if install dir is in PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo -e "${YELLOW}⚠ $INSTALL_DIR is not in your PATH${NC}"
    echo ""
    echo "Add it to your PATH by adding this line to your shell config:"
    echo ""
    if [ -n "$BASH_VERSION" ]; then
        echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.bashrc"
        echo "  source ~/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        echo "  echo 'export PATH=\"$INSTALL_DIR:\$PATH\"' >> ~/.zshrc"
        echo "  source ~/.zshrc"
    else
        echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
    echo ""
fi

# Verify installation
echo "Verifying installation..."
if command -v ccboard &> /dev/null; then
    VERSION_OUTPUT=$(ccboard --version 2>&1 || true)
    echo -e "${GREEN}✓ ccboard is ready to use${NC}"
    echo "$VERSION_OUTPUT"
else
    echo -e "${YELLOW}⚠ ccboard command not found in PATH${NC}"
    echo "You may need to add $INSTALL_DIR to your PATH (see above)"
fi

echo ""
echo "Usage:"
echo "  ccboard              # Launch TUI"
echo "  ccboard web          # Launch web interface"
echo "  ccboard stats        # Print stats"
echo "  ccboard --help       # Show help"
