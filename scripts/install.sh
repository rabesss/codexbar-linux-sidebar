#!/usr/bin/env bash
# Build and install the sidebar daemon, control CLI, systemd unit, and CodexBar CLI dependency.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFIX="${PREFIX:-$HOME/.local}"

echo "==> Building Rust workspace"
cargo build --release --manifest-path "$ROOT/Cargo.toml"

echo "==> Installing sidebar binaries"
install -d "$PREFIX/bin" "$PREFIX/share/codexbar-linux-sidebar/quickshell"
install -m 755 "$ROOT/target/release/codexbar-sidebard" "$PREFIX/bin/codexbar-sidebard"
install -m 755 "$ROOT/target/release/codexbar-sidebarctl" "$PREFIX/bin/codexbar-sidebarctl"
install -m 755 "$ROOT/scripts/codexbar-sidebar" "$PREFIX/bin/codexbar-sidebar"
cp -a "$ROOT/quickshell/." "$PREFIX/share/codexbar-linux-sidebar/quickshell/"

echo "==> Installing config example"
install -d "$HOME/.config/codexbar-linux-sidebar"
if [[ ! -f "$HOME/.config/codexbar-linux-sidebar/config.toml" ]]; then
  install -m 644 "$ROOT/examples/config.example.toml" "$HOME/.config/codexbar-linux-sidebar/config.toml"
fi

echo "==> Installing systemd user unit"
install -d "$HOME/.config/systemd/user"
install -m 644 "$ROOT/systemd/codexbar-sidebard.service" "$HOME/.config/systemd/user/codexbar-sidebard.service"
systemctl --user daemon-reload 2>/dev/null || true

echo "==> Fetching CodexBar CLI (upstream prebuilt Linux binary)"
bash "$ROOT/scripts/fetch-codexbar-cli.sh"

echo
echo "Installed:"
echo "  codexbar-sidebard"
echo "  codexbar-sidebarctl"
echo "  codexbar (upstream CLI, no macOS app bundled)"
echo
echo "Start daemon:  codexbar-sidebarctl start"
echo "Launch UI:     codexbar-sidebar autostart"
echo "Toggle UI:     CTRL+SUPER+U  (after install-hypr-hooks.sh)"
echo "Mock test:     codexbar-sidebard --once --mock"
