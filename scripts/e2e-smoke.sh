#!/usr/bin/env bash
# End-to-end smoke test: mock daemon write + state file validation.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUNTIME="$(mktemp -d)"
trap 'rm -rf "$RUNTIME"' EXIT

export XDG_RUNTIME_DIR="$RUNTIME"

echo "==> Building"
cargo build --manifest-path "$ROOT/Cargo.toml"

echo "==> Mock poll cycle"
"$ROOT/target/debug/codexbar-sidebard" --once --mock

STATE="$RUNTIME/codexbar-sidebar/state.json"
test -f "$STATE"

python3 - <<'PY' "$STATE"
import json, sys
state = json.load(open(sys.argv[1]))
assert state["schema_version"] == 1
assert len(state["providers"]) >= 6
labels = {p["id"]: p.get("usage", {}) and p["usage"].get("primary", {}) and p["usage"]["primary"].get("display_label") for p in state["providers"]}
assert labels.get("codex") == "72% left"
assert any(p["platform_state"] == "unsupported" for p in state["providers"])
print("mock state OK:", [p["id"] for p in state["providers"]])
PY

if command -v codexbar >/dev/null; then
  echo "==> Live poll cycle"
  "$ROOT/target/debug/codexbar-sidebard" --once
  python3 - <<'PY' "$STATE"
import json, sys
state = json.load(open(sys.argv[1]))
assert state["codexbar"]["available"] is True
assert state.get("codexbar", {}).get("version")
print("live state OK:", len(state["providers"]), "providers")
PY
else
  echo "==> Skipping live poll (codexbar not on PATH)"
fi

echo "E2E smoke test passed"
