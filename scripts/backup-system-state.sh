#!/usr/bin/env bash
# Backup every user file that codexbar-linux-sidebar install/autostart may touch.
set -euo pipefail

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
BACKUP_ROOT="${BACKUP_ROOT:-$HOME/.local/share/codexbar-sidebar-backups/$STAMP}"
mkdir -p "$BACKUP_ROOT"

copy_if_exists() {
  local src="$1"
  local dest="$2"
  if [[ -e "$src" ]]; then
    mkdir -p "$(dirname "$dest")"
    cp -a "$src" "$dest"
    printf '  saved %s\n' "$src"
  else
    printf '  (absent) %s\n' "$src"
  fi
}

{
  echo "CodexBar Linux Sidebar backup"
  echo "Timestamp (UTC): $STAMP"
  echo "Host: $(uname -n 2>/dev/null || echo unknown)"
  echo "User: $(id -un)"
  echo
  echo "Files captured:"
} >"$BACKUP_ROOT/MANIFEST.txt"

backup_one() {
  local src="$1"
  local rel="$2"
  copy_if_exists "$src" "$BACKUP_ROOT/$rel" | tee -a "$BACKUP_ROOT/MANIFEST.txt"
}

echo "Creating backup at $BACKUP_ROOT"

# Hyprland custom overrides (only files we may edit)
backup_one "$HOME/.config/hypr/custom/execs.lua" "config/hypr/custom/execs.lua"
backup_one "$HOME/.config/hypr/custom/keybinds.lua" "config/hypr/custom/keybinds.lua"
backup_one "$HOME/.config/hypr/custom/rules.lua" "config/hypr/custom/rules.lua"

# Existing codexbar/sidebar installs (if any)
backup_one "$HOME/.local/bin/codexbar-sidebar" "local/bin/codexbar-sidebar"
backup_one "$HOME/.local/bin/codexbar-sidebard" "local/bin/codexbar-sidebard"
backup_one "$HOME/.local/bin/codexbar-sidebarctl" "local/bin/codexbar-sidebarctl"
backup_one "$HOME/.local/bin/codexbar" "local/bin/codexbar"
backup_one "$HOME/.local/share/codexbar-cli" "local/share/codexbar-cli"
backup_one "$HOME/.local/share/codexbar-linux-sidebar" "local/share/codexbar-linux-sidebar"
backup_one "$HOME/.config/codexbar-linux-sidebar" "config/codexbar-linux-sidebar"
backup_one "$HOME/.config/systemd/user/codexbar-sidebard.service" "config/systemd/user/codexbar-sidebard.service"
backup_one "$HOME/.local/state/codexbar-sidebar" "local/state/codexbar-sidebar"

# systemd enablement state
if systemctl --user is-enabled codexbar-sidebard.service >/dev/null 2>&1; then
  systemctl --user is-enabled codexbar-sidebard.service >"$BACKUP_ROOT/systemd-codexbar-sidebard-enabled.txt"
  echo "  saved systemd enable state" | tee -a "$BACKUP_ROOT/MANIFEST.txt"
else
  echo "disabled" >"$BACKUP_ROOT/systemd-codexbar-sidebard-enabled.txt"
  echo "  (systemd unit not enabled)" | tee -a "$BACKUP_ROOT/MANIFEST.txt"
fi

# Running processes snapshot
{
  echo
  echo "Process snapshot:"
  pgrep -af 'codexbar-sidebard|codexbar-sidebar|qs --path .*codexbar-linux-sidebar' || true
} >>"$BACKUP_ROOT/MANIFEST.txt"

ln -sfn "$BACKUP_ROOT" "$HOME/.local/share/codexbar-sidebar-backups/latest"
printf '\nBackup complete: %s\n' "$BACKUP_ROOT"
printf 'Latest symlink: %s\n' "$HOME/.local/share/codexbar-sidebar-backups/latest"
