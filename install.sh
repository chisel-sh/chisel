#!/bin/bash
set -e

# Chisel Installation Script
# https://chisel.build

REPO="chisel-sh/chisel"
BINARY_NAME="chisel"

# 1. Detect Architecture and OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux*)   PLATFORM="unknown-linux-gnu"; EXT="tar.gz" ;;
  darwin*)  PLATFORM="apple-darwin"; EXT="tar.gz" ;;
  msys*|cygwin*|mingw*) PLATFORM="pc-windows-msvc"; EXT="zip" ;;
  *)        echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  x86_64)  ARCH_NAME="x86_64" ;;
  arm64|aarch64) ARCH_NAME="aarch64" ;;
  *)       echo "❌ Unsupported Architecture: $ARCH"; exit 1 ;;
esac

TARGET="${ARCH_NAME}-${PLATFORM}"

# 2. Get latest release version
echo "🔍 Finding latest release for $TARGET..."
# This gets the latest tag matching 'chisel-v*'
LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases" | grep '"tag_name": "chisel-v' | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "❌ Could not find latest release. Please check https://github.com/$REPO/releases"
    exit 1
fi

echo "🚀 Downloading Chisel $LATEST_TAG..."

# 3. Download and Extract
# GitHub assets for cargo-dist usually follow the format: {crate}-{target}.{ext}
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/chisel-$TARGET.$EXT"
TEMP_DIR=$(mktemp -d)

if [ "$EXT" = "tar.gz" ]; then
    curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_DIR/chisel.tar.gz"
    tar -xzf "$TEMP_DIR/chisel.tar.gz" -C "$TEMP_DIR"
else
    curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_DIR/chisel.zip"
    unzip -q "$TEMP_DIR/chisel.zip" -d "$TEMP_DIR"
fi

# 4. Install
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# cargo-dist often puts the binary in a subdirectory, let's find it
BIN_PATH=$(find "$TEMP_DIR" -name "$BINARY_NAME" -type f | head -n 1)
if [ -z "$BIN_PATH" ]; then
    # Check for .exe on Windows
    BIN_PATH=$(find "$TEMP_DIR" -name "${BINARY_NAME}.exe" -type f | head -n 1)
fi

if [ -z "$BIN_PATH" ]; then
    echo "❌ Could not find binary in release archive"
    exit 1
fi

mv "$BIN_PATH" "$INSTALL_DIR/$BINARY_NAME"
chmod +x "$INSTALL_DIR/$BINARY_NAME"

echo "✨ Chisel $LATEST_TAG installed successfully to $INSTALL_DIR/$BINARY_NAME"
echo "👉 Run 'chisel --help' to get started."

# Cleanup
rm -rf "$TEMP_DIR"
