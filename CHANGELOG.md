## Unreleased

- Reworked the capture TUI into a workspace-first flow: choose a workspace, then capture the whole workspace or select one of its windows, then choose full-window or content-only mode.
- Added `capture workspace` and `capture window` CLI workflows, explicit workspace selection for `show wind`, and `--mode full|content`.
- Added Hyprland compositor-border-aware window bounds. Content mode captures the application surface; full mode expands to include the configured compositor border.
- Expanded README command and TUI documentation. Documented that application-drawn browser/toolbars remain part of the client surface and should be cropped with region capture.

# Changelog

## Unreleased

- Kept full-screen and region quick-capture actions visible in the TUI even when workspace discovery is available (including Hyprland).
- Routed Wayland `capture region` through CyberShot's frozen-frame selector instead of requiring external `slurp`.
- Added detailed `cybershot help` / `--help` command documentation, including nested command help and examples.
- Made shortcut status and enable messages reflect the invoked command name (`cybershot` or `csb`).
- Added optional, conflict-aware Hyprland shortcut integration for Lua and traditional configs, with a removable managed block.
- Made `csb hotkey` use the same Wayland frozen-frame selector as `csb select`.
- Added native source-build installers/uninstallers for macOS and Windows.
- Expanded platform shortcut and TTY documentation.
- Added formatting checks to CI.


## 0.2.0

- Added `csb keybind enable`, `disable`, and `status` commands.
- Added `csb hotkey` as a shared region-capture entry point for user-configured OS/desktop shortcuts.
- Added cross-platform shortcut setup guidance for Hyprland, GNOME, KDE Plasma, Windows, and macOS.
- Preserved compatibility with older configuration files by defaulting newly introduced configuration fields.
- Physical global shortcut registration remains user-managed; CyberShot does not install a background hotkey daemon.
