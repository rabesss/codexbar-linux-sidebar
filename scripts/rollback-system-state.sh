#!/usr/bin/env bash
# Restore a backup created by backup-system-state.sh and remove codexbar sidebar autostart hooks.
set -euo pipefail

BACKUP="${1:-$HOME/.local/share/codexbar-sidebar-backups/latest}"
if [[ ! -d "$BACKUP" ]]; then
  echo "Backup not found: $BACKUP" >&2
  exit 1
fi

restore_if_backed_up() {
  local rel="$1"
  local dest="$2"
  if [[ -e "$BACKUP/$rel" ]]; then
    mkdir -p "$(dirname "$dest")"
    cp -a "$BACKUP/$rel" "$dest"
    printf 'restored %s\n' "$dest"
  fi
}

echo "Rolling back from $BACKUP"

systemctl --user stop codexbar-sidebard.service 2>/dev/null || true
systemctl --user disable codexbar-sidebard.service 2>/dev/null || true
pkill -f 'qs --path .*codexbar-linux-sidebar' 2>/dev/null || true

restore_if_backed_up "config/hypr/custom/execs.lua" "$HOME/.config/hypr/custom/execs.lua"
restore_if_backed_up "config/hypr/custom/keybinds.lua" "$HOME/.config/hypr/custom/keybinds.lua"
restore_if_backed_up "config/hypr/custom/rules.lua" "$HOME/.config/hypr/custom/rules.lua"

# Remove install artifacts unless they existed before backup (restored above)
for path in \
  "$HOME/.local/bin/codexbar-sidebar" \
  "$HOME/.local/bin/codexbar-sidebard" \
  "$HOME/.local/bin/codexbar-sidebarctl" \
  "$HOME/.local/share/codexbar-linux-sidebar" \
  "$HOME/.config/codexbar-linux-sidebar" \
  "$HOME/.config/systemd/user/codexbar-sidebard.service" \
  "$HOME/.local/state/codexbar-sidebar"; do
  rel="${path/#$HOME\//}"
  if [[ ! -e "$BACKUP/$rel" && -e "$path" ]]; then
    rm -rf "$path"
    printf 'removed new install artifact %s\n' "$path"
  fi
done

# Restore prior installs if backup had them
restore_if_backed_up "local/bin/codexbar-sidebar" "$HOME/.local/bin/codexbar-sidebar"
restore_if_backed_up "local/bin/codexbar-sidebard" "$HOME/.local/bin/codexbar-sidebard"
restore_if_backed_up "local/bin/codexbar-sidebarctl" "$HOME/.local/bin/codexbar-sidebarctl"
restore_if_backed_up "local/share/codexbar-linux-sidebar" "$HOME/.local/share/codexbar-linux-sidebar"
restore_if_backed_up "config/codexbar-linux-sidebar" "$HOME/.config/codexbar-linux-sidebar"
restore_if_backed_up "config/systemd/user/codexbar-sidebard.service" "$HOME/.config/systemd/user/codexbar-sidebard.service"
restore_if_backed_up "local/state/codexbar-sidebar" "$HOME/.local/state/codexbar-sidebar"

systemctl --user daemon-reload 2>/dev/null || true

if [[ -f "$BACKUP/systemd-codexbar-sidebard-enabled.txt" ]]; then
  state="$(tr -d '\n' <"$BACKUP/systemd-codexbar-sidebard-enabled.txt")"
  if [[ "$state" == "enabled" ]]; then
    systemctl --user enable --now codexbar-sidebard.service || true
  fi
fi

echo "Rollback complete. Reload Hyprland config: hyprctl reload (or log out/in)."
