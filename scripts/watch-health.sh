#!/usr/bin/env bash
# Health check for codexbar-sidebar stack. Exit 0 = healthy.
set -euo pipefail

SIDEBAR="${HOME}/.local/bin/codexbar-sidebar"
SIDEBARCTL="${HOME}/.local/bin/codexbar-sidebarctl"
RUNTIME="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
STATE_JSON="$RUNTIME/codexbar-sidebar/state.json"
FAIL=0

check() {
  if "$@"; then
    printf '[ok] %s\n' "$*"
  else
    printf '[FAIL] %s\n' "$*"
    FAIL=1
  fi
}

command -v codexbar >/dev/null && printf '[ok] codexbar on PATH\n' || { printf '[FAIL] codexbar missing\n'; FAIL=1; }
command -v codexbar-sidebard >/dev/null && printf '[ok] codexbar-sidebard on PATH\n' || { printf '[FAIL] codexbar-sidebard missing\n'; FAIL=1; }

if systemctl --user is-active codexbar-sidebard.service >/dev/null 2>&1; then
  printf '[ok] systemd codexbar-sidebard active\n'
else
  printf '[FAIL] systemd codexbar-sidebard not active\n'
  FAIL=1
fi

if [[ -f "$STATE_JSON" ]]; then
  age=$(( $(date +%s) - $(stat -c %Y "$STATE_JSON") ))
  printf '[ok] state.json present (age %ss)\n' "$age"
  if (( age > 300 )); then
    printf '[warn] state.json older than 5 minutes\n'
  fi
else
  printf '[FAIL] missing %s\n' "$STATE_JSON"
  FAIL=1
fi

if pgrep -f 'qs --path .*codexbar-linux-sidebar' >/dev/null; then
  printf '[ok] quickshell panel process running\n'
else
  printf '[warn] quickshell panel not running (toggle with codexbar-sidebar toggle)\n'
fi

if [[ -x "$SIDEBAR" ]]; then
  "$SIDEBAR" doctor >/dev/null 2>&1 && printf '[ok] codexbar-sidebar doctor\n' || { printf '[warn] codexbar-sidebar doctor reported issues\n'; FAIL=1; }
fi

exit "$FAIL"
