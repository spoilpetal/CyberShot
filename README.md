# CyberShot

<p align="center">
  <strong>A terminal-first screenshot tool built with Rust</strong><br>
  Capture · History · File Manager · Clipboard
</p>

<p align="center">
  <a href="https://github.com/zaynkhan100110/CyberShot">GitHub</a> ·
  <a href="https://github.com/zaynkhan100110/CyberShot/issues">Issues</a> ·
  <a href="https://github.com/zaynkhan100110/CyberShot/releases">Releases</a>
</p>

CyberShot is a standalone, keyboard-driven screenshot utility. It has two application TUIs: a combined capture/settings interface and a screenshot History. The `files` command opens a separate file-manager interface. It does not require CyberClip or another Cyber* application.

## Features

- Full-screen and interactive region capture.
- Hyprland workspace and window discovery/capture.
- X11 workspace and window discovery through `wmctrl`.
- Current desktop and visible-window discovery on Windows and macOS.
- Screenshot History with open, copy-to-image-clipboard, rename, and delete actions.
- Two-pane file manager with navigation, search, multi-select, copy/paste, rename, delete confirmation, and native file-manager handoff.
- Native image clipboard integration for Linux Wayland/X11, Windows, and macOS.
- Configurable screenshot directory.
- No dependency on CyberClip.

## Commands

| Command | Description |
|---|---|
| `csb` or `csb tui` | Open capture and settings TUI |
| `csb history` | Browse, open, copy, rename, or delete screenshots |
| `csb files` | Open the two-pane file manager |
| `csb capture full` | Capture all available displays / the desktop |
| `csb capture region` | Select a screen region |
| `csb capture work` | Capture the selected/current desktop |
| `csb capture wind <number>` | Capture a listed window |
| `csb capture wind <number> --noh` | Capture the window with the top 32 px excluded |
| `csb show work` | List available workspaces/desktops |
| `csb sl work <number>` | Select a workspace where switching is supported |
| `csb show wind` | List windows in the selected/current workspace |
| `csb location` | Print the screenshot directory |
| `csb select` | Start the platform region selector |

`csb capture wind` uses the order shown by `csb show wind`. `--noh` excludes a fixed 32-pixel strip at the top of the selected window; this is intended for title bars and may need adjustment for unusual desktop themes or scaling.

## Keyboard controls

### Capture TUI

- `↑` / `↓`: navigate
- `Enter`: choose workspace/window or capture
- `s`: open Settings
- `d`: edit the screenshot directory in Settings
- `Backspace`: return from window list to workspace list
- `Esc`: go back / exit

### History

- `↑` / `↓`: select screenshot
- `Enter` / `o`: open in the default image viewer
- `c`: copy image data to the system clipboard
- `r`: rename
- `d`: request deletion (confirm when prompted)
- `Esc`: exit

### File manager

- `↑` / `↓`: navigate
- `Enter`: open directory or file
- `Backspace`: parent directory
- `/`: search by filename
- `Space`: mark/unmark an item
- `c`: copy marked items (or the selected item)
- `Ctrl+V`: paste into the current directory
- `r`: rename
- `d`: delete (confirmation required)
- `f`: open the current directory in the native file manager
- `q` / `Esc`: exit

The file manager's built-in copy/paste selection is scoped to the running TUI session; it is not a persistent cross-application file clipboard.

## Platform support

| Platform / session | Full capture | Region capture | Window discovery/capture | Workspace discovery/switching |
|---|---|---|---|---|
| Linux + Hyprland (Wayland) | `grim` | `slurp` + `grim` | Yes | Yes |
| Linux + X11 | ImageMagick `import` | `slop` + ImageMagick | `wmctrl` | `wmctrl` |
| Linux + other Wayland compositors | `grim` | `slurp` + `grim` | Compositor-specific; not provided by this generic backend | Compositor-specific; not provided by this generic backend |
| macOS | `screencapture` | `screencapture -i` | AppleScript / System Events | Current desktop only; native Space switching is not implemented |
| Windows | PowerShell / .NET | PowerShell / WinForms | Win32 window enumeration | Current desktop only; native virtual-desktop switching is not implemented |

**macOS permission:** window listing/capture may require granting Accessibility permission to the terminal application (or the CyberShot executable) in System Settings → Privacy & Security → Accessibility. Screen Recording permission is also required for screen capture.

**Windows:** full-screen and region capture use the virtual-screen bounds, including multiple monitors. Window enumeration uses Win32 and may omit windows that Windows does not expose as visible top-level windows.

**Linux:** the generic X11 backend requires `wmctrl`; Wayland workspace/window control is compositor-specific. Hyprland is fully integrated. Other Wayland compositors still support full-screen and region capture when `grim` and `slurp` are available.

## Dependencies

### Arch Linux / Hyprland

```sh
sudo pacman -S --needed rustup grim slurp gtk4
# For X11 sessions, also install:
sudo pacman -S --needed imagemagick slop xclip wmctrl
```

On Debian/Ubuntu, the equivalent packages are `cargo rustc pkg-config libgtk-4-dev grim slurp imagemagick slop xclip wmctrl` as needed for the session.

### macOS

Install Rust with [rustup](https://rustup.rs/). CyberShot uses the built-in `screencapture`, `osascript`, and System Events facilities.

### Windows

Install Rust with [rustup](https://rustup.rs/). Capture, window enumeration, clipboard, and file-manager integration use Windows PowerShell/.NET and Win32 facilities.

## Build from source

```sh
git clone https://github.com/zaynkhan100110/CyberShot.git
cd CyberShot
cargo build --release
```

The binary is `target/release/cybershot` on Unix-like systems and `target/release/cybershot.exe` on Windows.

### Install on Linux

```sh
chmod +x install.sh
./install.sh
```

The installer checks for existing `cybershot`/`csb` commands and known desktop shortcut or shell-alias references before installing. It asks which command name(s) to install: `cybershot`, `csb`, or both. Existing files are never replaced without confirmation. Normally it builds the release binary; set `CYBERSHOT_SKIP_BUILD=1` to reuse an already-built `target/release/cybershot` without starting another build. On Linux, GTK4 development files are required to build the frozen-frame region selector. Ensure `~/.local/bin` is on your `PATH`.

The uninstaller is interactive. You can remove either command name independently, remove both while keeping data, or choose a full uninstall. A full uninstall asks separately before deleting settings and the default screenshots folder; screenshots and settings are preserved unless you confirm their deletion.

## Configuration and screenshots

- Default screenshot folder: the platform Pictures directory under `CyberShot` (for example `~/Pictures/CyberShot`).
- Change the folder in the Capture TUI: press `s`, then `d`.
- Configuration file: the platform configuration directory under `cybershot/config.toml`.

## GitHub Actions

- **CI** runs `cargo check --locked --all-targets` and `cargo test --locked` on Ubuntu, Windows, and macOS. The repository does not currently include a dedicated formatting-check step.
- **Release** builds native binaries when a tag matching `v*` is pushed and attaches the artifacts to a GitHub Release.

A green CI run verifies compilation on the corresponding GitHub-hosted runners. It does not replace interactive testing on each desktop session, particularly Wayland compositor permissions and macOS Accessibility/Screen Recording permissions.

## Current scope

- Native workspace switching is implemented for Hyprland and X11. On Windows and macOS, CyberShot reports the current desktop and captures it; switching between virtual desktops/Spaces is not implemented.
- Generic non-Hyprland Wayland window discovery is not available because Wayland does not provide a universal window-enumeration protocol. Full-screen and region capture remain available through `grim`/`slurp`.
- Global shortcut registration is not bundled. Configure a shortcut in your desktop environment to launch `csb capture region` or `csb`.
- The file-manager copy buffer is internal to the active TUI session.

## Contributing and bug reports

Please include your OS, desktop/session (Hyprland, X11, GNOME Wayland, KDE Wayland, etc.), CyberShot version, command used, and the complete error output when opening an issue:

<https://github.com/zaynkhan100110/CyberShot/issues>

## License

MIT. See [LICENSE](LICENSE).
