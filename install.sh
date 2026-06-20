#!/usr/bin/env bash
set -euo pipefail

REPO="digitalninjanv/fedora-system-monitor"
BIN="fedora-monitor"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect platform
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
  aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
  armv7l)  TARGET="armv7-unknown-linux-gnueabihf" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

# Get latest release tag
echo "Fetching latest release..."
TAG=$(curl -sSfL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4)
if [ -z "$TAG" ]; then
  echo "Failed to fetch latest release tag"
  exit 1
fi
echo "Latest release: $TAG"

# Download archive
ARCHIVE="$BIN-$TAG-$TARGET.tar.gz"
URL="https://github.com/$REPO/releases/download/$TAG/$ARCHIVE"
echo "Downloading $URL ..."
curl -sSfL "$URL" -o "/tmp/$ARCHIVE"

# Extract
echo "Extracting..."
tar -xzf "/tmp/$ARCHIVE" -C /tmp

# Install
mkdir -p "$INSTALL_DIR"
install -m 755 "/tmp/$BIN" "$INSTALL_DIR/$BIN"
rm -f "/tmp/$ARCHIVE" "/tmp/$BIN"

echo "Installed $BIN $TAG to $INSTALL_DIR/$BIN"
echo "Make sure $INSTALL_DIR is in your PATH."
