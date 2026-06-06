#!/usr/bin/env bash
# Install CodexBar into end-4 Illogical Impulse left sidebar (qs -c ii).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
II_SIDEBAR="$HOME/.config/quickshell/ii/modules/ii/sidebarLeft"
BACKUP="$HOME/.local/share/codexbar-sidebar-backups/pre-ii-integration-$(date -u +%Y%m%dT%H%M%SZ)"

mkdir -p "$BACKUP"
for f in SidebarLeftContent.qml; do
  [[ -f "$II_SIDEBAR/$f" ]] && cp -a "$II_SIDEBAR/$f" "$BACKUP/"
done
[[ -f "$HOME/.config/quickshell/ii/modules/ii/bar/LeftSidebarButton.qml" ]] && \
  cp -a "$HOME/.config/quickshell/ii/modules/ii/bar/LeftSidebarButton.qml" "$BACKUP/"

install -Dm644 "$ROOT/quickshell-integration/ii/CodexBar.qml" "$II_SIDEBAR/CodexBar.qml"
bash "$ROOT/scripts/install-hypr-hooks.sh" install-ii

echo "Installed CodexBar tab into ii left sidebar."
echo "Backup: $BACKUP"
echo "Reload shell: CTRL+SUPER+R (or log out/in)"
echo "Open sidebar: SUPER+A → Usage tab"
