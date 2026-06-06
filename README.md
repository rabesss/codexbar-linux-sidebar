# CodexBar Linux Sidebar

A standalone [Quickshell](https://quickshell.outfoxxed.me/) left panel that shows CodexBar provider usage, reset timers, credits, cost, and status on Linux.

```
codexbar CLI  →  codexbar-sidebard  →  state.json  →  Quickshell sidebar
```

This repository ships **only** the Rust daemon, control CLI, and Quickshell UI. It does **not** vendor the CodexBar macOS app or SwiftUI sources. The CodexBar CLI is installed as an upstream **prebuilt Linux binary** from [steipete/CodexBar releases](https://github.com/steipete/CodexBar/releases).

## Quick install

```bash
git clone https://github.com/rabesss/codexbar-linux-sidebar.git
cd codexbar-linux-sidebar
./scripts/install.sh
```

Requirements:

- Rust toolchain (`cargo`)
- Quickshell (for the UI panel)
- `curl`, `sha256sum` (for CodexBar CLI fetch)

## Usage

```bash
# Start background daemon (systemd --user if available)
codexbar-sidebarctl start

# One-shot refresh / mock dev data
codexbar-sidebard --once --mock

# Status summary
codexbar-sidebarctl status

# Launch sidebar UI
quickshell --path ~/.local/share/codexbar-linux-sidebar/quickshell
```

State file: `$XDG_RUNTIME_DIR/codexbar-sidebar/state.json`

## Architecture

| Component | Role |
|-----------|------|
| `codexbar` | Upstream CLI: usage + status + cost JSON |
| `codexbar-sidebard` | Polls CodexBar, normalizes provider cards, atomic-writes state |
| `codexbar-sidebarctl` | start / stop / status / refresh |
| `quickshell/` | FileView.watchChanges UI with 5s timer fallback |

Collectors (each poll cycle):

```bash
codexbar --provider all --format json --json-only --status
codexbar cost --format json --json-only   # Claude + Codex only
```

## Bundling policy

We intentionally **do not** ship CodexBar Swift/macOS sources in this repo.

- `scripts/fetch-codexbar-cli.sh` downloads the official `CodexBarCLI-v*-linux-*.tar.gz` release asset
- Installs to `~/.local/share/codexbar-cli/` with a `VERSION` file
- Symlinks `~/.local/bin/codexbar`

To pin a version:

```bash
CODEXBAR_VERSION=0.32.4 ./scripts/fetch-codexbar-cli.sh
```

## Configuration

`~/.config/codexbar-linux-sidebar/config.toml` — see `examples/config.example.toml`.

## Development

```bash
cargo test
cargo test -- --ignored          # requires codexbar on PATH
./scripts/e2e-smoke.sh
```

## Packaging (Arch)

See `packaging/PKGBUILD` for an Arch Linux package that builds the sidebar and optionally fetches the CodexBar CLI at install time.

## License

MIT

## Upstream

- [CodexBar](https://github.com/steipete/CodexBar) — provider usage CLI (installed separately)
- [Quickshell](https://quickshell.outfoxxed.me/) — Wayland shell for the panel UI
