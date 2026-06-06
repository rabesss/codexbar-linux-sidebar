#!/usr/bin/env bash
# Inject or remove Hyprland hooks for codexbar-linux-sidebar.
set -euo pipefail

MODE="${1:-install-ii}"
EXECS_FILE="$HOME/.config/hypr/custom/execs.lua"
KEYBINDS_FILE="$HOME/.config/hypr/custom/keybinds.lua"
RULES_FILE="$HOME/.config/hypr/custom/rules.lua"
MARK_BEGIN='-- BEGIN codexbar-linux-sidebar (managed)'
MARK_END='-- END codexbar-linux-sidebar (managed)'

remove_block() {
  local file="$1"
  [[ -f "$file" ]] || return 0
  python3 - "$file" "$MARK_BEGIN" "$MARK_END" <<'PY'
import sys
from pathlib import Path
path, begin, end = sys.argv[1:4]
text = Path(path).read_text()
start = text.find(begin)
if start == -1:
    sys.exit(0)
stop = text.find(end, start)
if stop == -1:
    raise SystemExit(f'missing end marker in {path}')
stop = text.find('\n', stop)
stop = len(text) if stop == -1 else stop + 1
Path(path).write_text(text[:start] + text[stop:])
PY
}

install_ii_hooks() {
  remove_block "$EXECS_FILE"
  remove_block "$KEYBINDS_FILE"
  remove_block "$RULES_FILE"

  {
    [[ -f "$EXECS_FILE" ]] && cat "$EXECS_FILE"
    [[ -s "$EXECS_FILE" ]] && printf '\n'
    echo "$MARK_BEGIN"
    cat <<'LUA'
hl.on("hyprland.start", function()
    hl.exec_cmd("~/.local/bin/codexbar-sidebarctl start")
end)
LUA
    echo "$MARK_END"
  } >"${EXECS_FILE}.tmp" && mv "${EXECS_FILE}.tmp" "$EXECS_FILE"

  {
    [[ -f "$KEYBINDS_FILE" ]] && cat "$KEYBINDS_FILE"
    [[ -s "$KEYBINDS_FILE" ]] && printf '\n'
    echo "$MARK_BEGIN"
    cat <<'LUA'
hl.bind("CTRL + SUPER + SHIFT + U", hl.dsp.exec_cmd("~/.local/bin/codexbar-sidebarctl refresh"), {
    description = "CodexBar: Refresh provider usage data"
})
LUA
    echo "$MARK_END"
  } >"${KEYBINDS_FILE}.tmp" && mv "${KEYBINDS_FILE}.tmp" "$KEYBINDS_FILE"

  echo "Installed ii integration Hypr hooks (daemon autostart + refresh keybind)"
}

case "$MODE" in
  install-ii)
    install_ii_hooks
    ;;
  install)
    echo "Standalone panel install is deprecated. Use: $0 install-ii" >&2
    exit 2
    ;;
  remove)
    remove_block "$EXECS_FILE"
    remove_block "$KEYBINDS_FILE"
    remove_block "$RULES_FILE"
    echo "Removed Hyprland hooks"
    ;;
  *)
    echo "usage: $0 [install-ii|remove]" >&2
    exit 2
    ;;
esac
