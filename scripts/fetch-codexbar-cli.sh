#!/usr/bin/env bash
# Fetch the official CodexBar Linux CLI release binary (no macOS app / Swift source vendored).
set -euo pipefail

VERSION="${CODEXBAR_VERSION:-0.32.4}"
REPO="${CODEXBAR_REPO:-steipete/CodexBar}"
PREFIX="${PREFIX:-$HOME/.local}"
SHARE_DIR="$PREFIX/share/codexbar-cli"
BIN_DIR="$PREFIX/bin"

arch="$(uname -m)"
case "$arch" in
  x86_64) ASSET_ARCH="x86_64" ;;
  aarch64|arm64) ASSET_ARCH="aarch64" ;;
  *)
    echo "Unsupported architecture: $arch" >&2
    exit 1
    ;;
esac

TARBALL="CodexBarCLI-v${VERSION}-linux-${ASSET_ARCH}.tar.gz"
CHECKSUM="${TARBALL}.sha256"
URL="https://github.com/${REPO}/releases/download/v${VERSION}/${TARBALL}"
CHECKSUM_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${CHECKSUM}"

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

echo "Downloading CodexBar CLI v${VERSION} (${ASSET_ARCH})..."
curl -fsSL "$URL" -o "$WORKDIR/$TARBALL"
curl -fsSL "$CHECKSUM_URL" -o "$WORKDIR/$CHECKSUM"

(
  cd "$WORKDIR"
  expected="$(awk '{print $1}' "$CHECKSUM")"
  actual="$(sha256sum "$TARBALL" | awk '{print $1}')"
  if [[ "$expected" != "$actual" ]]; then
    echo "Checksum mismatch for $TARBALL" >&2
    exit 1
  fi
  tar -xzf "$TARBALL"
)

mkdir -p "$SHARE_DIR" "$BIN_DIR"
install -m 755 "$WORKDIR/codexbar" "$SHARE_DIR/codexbar"
if [[ -f "$WORKDIR/VERSION" ]]; then
  install -m 644 "$WORKDIR/VERSION" "$SHARE_DIR/VERSION"
else
  echo "$VERSION" > "$SHARE_DIR/VERSION"
fi
ln -sf "$SHARE_DIR/codexbar" "$BIN_DIR/codexbar"

echo "Installed codexbar to $BIN_DIR/codexbar (-> $SHARE_DIR/codexbar)"
"$BIN_DIR/codexbar" -V 2>&1 || true
echo "VERSION file: $(cat "$SHARE_DIR/VERSION")"
