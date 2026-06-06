#!/usr/bin/env bash
# One-shot: backup → install → Hypr hooks → enable daemon → smoke test
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "==> 1/5 Backup current system state"
bash "$ROOT/scripts/backup-system-state.sh"

echo "==> 2/5 Install sidebar binaries + CodexBar CLI"
bash "$ROOT/scripts/install.sh"

echo "==> 3/5 Install Hyprland autostart + keybinds"
bash "$ROOT/scripts/install-hypr-hooks.sh" install

echo "==> 4/5 Enable and start daemon"
systemctl --user enable --now codexbar-sidebard.service

echo "==> 5/5 Smoke test"
bash "$ROOT/scripts/e2e-smoke.sh"
"$HOME/.local/bin/codexbar-sidebar" doctor

echo
echo "Setup complete."
echo "  Toggle panel now:  codexbar-sidebar toggle"
echo "  Or keybind:        CTRL+SUPER+U"
echo "  Rollback:          $ROOT/scripts/rollback-system-state.sh"
echo "  Docs:              $ROOT/docs/SYSTEM-INSTALL.md"
