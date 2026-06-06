# System install & rollback

This documents **every file** touched when enabling the CodexBar sidebar on your machine alongside end-4 Hyprland (`qs -c ii`).

## What is NOT modified

These stay untouched (safe end-4 setup):

- `~/.config/quickshell/ii/**` — your main Illogical Impulse shell
- `~/.config/illogical-impulse/config.json`
- `~/.config/hypr/hyprland/**` — upstream end-4 defaults (only `custom/` is edited)

The CodexBar panel runs as a **second Quickshell instance** (same pattern as `focus-timer`).

## Files created by install

| Path | Purpose |
|------|---------|
| `~/.local/bin/codexbar-sidebard` | Polling daemon |
| `~/.local/bin/codexbar-sidebarctl` | start/stop/status/refresh |
| `~/.local/bin/codexbar-sidebar` | Panel launcher + toggle |
| `~/.local/share/codexbar-linux-sidebar/quickshell/` | Standalone QML UI |
| `~/.local/share/codexbar-cli/` | Upstream CodexBar CLI binary + VERSION |
| `~/.config/codexbar-linux-sidebar/config.toml` | Daemon config |
| `~/.config/systemd/user/codexbar-sidebard.service` | User systemd unit |
| `~/.local/state/codexbar-sidebar/state` | Panel visible/hidden state |
| `$XDG_RUNTIME_DIR/codexbar-sidebar/state.json` | Live provider data (runtime) |

## Files modified by Hypr hooks

| Path | Change |
|------|--------|
| `~/.config/hypr/custom/execs.lua` | Autostart: `codexbar-sidebar autostart` |
| `~/.config/hypr/custom/keybinds.lua` | `CTRL+SUPER+U` toggle, `CTRL+SUPER+SHIFT+U` refresh |
| `~/.config/hypr/custom/rules.lua` | Layer blur for namespace `codexbar-sidebar` |

Managed blocks are wrapped in:

```lua
-- BEGIN codexbar-linux-sidebar (managed)
...
-- END codexbar-linux-sidebar (managed)
```

## Backup & rollback

```bash
# Before install (automatic in setup-system.sh)
./scripts/backup-system-state.sh

# Full rollback to pre-install state
./scripts/rollback-system-state.sh

# Or only remove Hypr autostart/keybinds
./scripts/install-hypr-hooks.sh remove
```

Backups live at: `~/.local/share/codexbar-sidebar-backups/<timestamp>/`  
Latest symlink: `~/.local/share/codexbar-sidebar-backups/latest`

## Test on your sidebar

```bash
codexbar-sidebar doctor          # binaries + state file
codexbar-sidebarctl status       # daemon summary
codexbar-sidebar toggle          # show/hide left panel
codexbar-sidebar refresh         # force poll

# After Hyprland reload / re-login:
# Panel autostarts with daemon via custom/execs.lua
hyprctl reload
```

## Keybinds

| Binding | Action |
|---------|--------|
| `CTRL + SUPER + U` | Toggle CodexBar usage panel |
| `CTRL + SUPER + SHIFT + U` | Refresh provider data |
| `SUPER + A` | Unchanged — end-4 left sidebar (currently empty tabs) |

Both panels anchor left; hide one with its toggle if they overlap.

## Conflict note

- end-4 left sidebar: ~460px exclusive zone when open
- CodexBar panel: 430px when visible

Use `codexbar-sidebar hide` or `CTRL+SUPER+U` when using the ii left sidebar.
