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
  linux*)   PLATFORM="unknown-linux-gnu" ;;
  darwin*)  PLATFORM="apple-darwin" ;;
  msys*|cygwin*|mingw*) PLATFORM="pc-windows-msvc" ;;
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
# This gets the full tag name (e.g. chisel-v0.1.1)
LATEST_TAG=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "❌ Could not find latest release. Please check https://github.com/$REPO/releases"
    exit 1
fi

echo "🚀 Downloading Chisel $LATEST_TAG..."

# 3. Download and Extract
# GitHub assets for cargo-dist usually follow the format: {crate}-{target}.tar.gz
DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_TAG/chisel-$TARGET.tar.gz"
TEMP_DIR=$(mktemp -d)

curl -L "$DOWNLOAD_URL" -o "$TEMP_DIR/chisel.tar.gz"
tar -xzf "$TEMP_DIR/chisel.tar.gz" -C "$TEMP_DIR"

# 4. Install
INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

mv "$TEMP_DIR/chisel" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/chisel"

echo "✨ Chisel $LATEST_RELEASE installed successfully to $INSTALL_DIR/chisel"
echo "👉 Run 'chisel --help' to get started."

# Cleanup
rm -rf "$TEMP_DIR"
