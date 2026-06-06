# Patches for end-4 Illogical Impulse (qs -c ii)

Apply these to integrate CodexBar into the **existing left sidebar** (SUPER+A), not a standalone panel.

## Files modified on your system

| File | Change |
|------|--------|
| `~/.config/quickshell/ii/modules/ii/sidebarLeft/CodexBar.qml` | **New** — Usage tab content |
| `~/.config/quickshell/ii/modules/ii/sidebarLeft/SidebarLeftContent.qml` | Adds "Usage" tab |
| `~/.config/quickshell/ii/modules/ii/bar/LeftSidebarButton.qml` | Shows bar button when CodexBar enabled |
| `~/.config/hypr/custom/execs.lua` | Daemon autostart only (`codexbar-sidebarctl start`) |
| `~/.config/hypr/custom/keybinds.lua` | `CTRL+SUPER+SHIFT+U` refresh |

## NOT used (standalone panel removed)

- No second `qs --path …/codexbar-linux-sidebar` process
- No `hl.layerrule` in custom/rules.lua (breaks end-4 Hyprland Lua API)

## Usage

1. `codexbar-sidebarctl start` (or autostart on login)
2. `SUPER + A` → open end-4 left sidebar
3. First tab: **Usage** — provider cards from `$XDG_RUNTIME_DIR/codexbar-sidebar/state.json`
4. `CTRL + SUPER + SHIFT + U` — refresh data

## Rollback ii integration

```bash
cp ~/.local/share/codexbar-sidebar-backups/pre-ii-integration/* \
   ~/.config/quickshell/ii/modules/ii/sidebarLeft/
cp ~/.local/share/codexbar-sidebar-backups/pre-ii-integration/LeftSidebarButton.qml \
   ~/.config/quickshell/ii/modules/ii/bar/
rm ~/.config/quickshell/ii/modules/ii/sidebarLeft/CodexBar.qml
scripts/install-hypr-hooks.sh remove
hyprctl reload
# then CTRL+SUPER+R to reload qs -c ii
```

## Install on another machine

```bash
./scripts/install.sh
./scripts/install-ii-integration.sh   # copies CodexBar.qml + Hypr hooks; patch SidebarLeft* manually or from backup
```

Copy `quickshell-integration/ii/CodexBar.qml` into your ii sidebarLeft folder and merge the small changes in `SidebarLeftContent.qml` / `LeftSidebarButton.qml` (see git history or `pre-ii-integration` backup diff).
