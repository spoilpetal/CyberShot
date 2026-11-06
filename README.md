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
| `cybershot` / `csb` or `cybershot tui` | Open the workspace-first capture TUI |
| `cybershot --help` / `cybershot help` | Show the full command guide |
| `cybershot help <command>` | Show detailed help for a command |
| `csb show work` | List available workspaces and their IDs |
| `csb show wind --workspace <ID>` | List windows in the specified workspace |
| `csb sl work <NUMBER>` | Save a workspace using its position in `show work` |
| `csb capture workspace <NUMBER>` | Capture the workspace at the displayed position in `show work` |
| `csb capture window <NUMBER> --workspace <ID> --mode full` | Capture a window including its compositor border |
| `csb capture window <NUMBER> --workspace <ID> --mode content` | Capture the window surface without the compositor border |
| `csb capture window <NUMBER> --workspace <ID> --mode page` | Interactively crop only the webpage viewport, excluding browser tabs and address bar |
| `csb capture full` | Capture the entire desktop / all displays |
| `csb capture region` | Select and capture a screen region |
| `csb history` | Browse, open, copy, rename, or delete screenshots |
| `csb files` | Open the two-pane file manager |
| `csb location` | Print the screenshot directory |
| `csb keybind enable [--shortcut "..."]` | Enable the externally assigned shortcut trigger |
| `csb keybind disable` | Disable the shortcut trigger without removing the OS shortcut |
| `csb keybind status` | Show shortcut-trigger state |
| `csb hotkey` | Run region capture when the shortcut trigger is enabled |

When using `capture window`, window numbering matches `show wind --workspace ID`. If `--workspace` is omitted, CyberShot uses the workspace saved by `sl work NUMBER`. `--mode full` is the default. On Hyprland, `--mode content` trims the configured compositor border. Application-drawn browser controls (tabs, address bar, toolbar) are part of the application surface and cannot be identified reliably from generic window geometry. Use `--mode page` to open a frozen-frame selector and draw a crop around just the webpage viewport. This is an explicit interactive crop, not automatic browser chrome detection.

### Command help

```sh
cybershot --help
cybershot help
cybershot help capture
cybershot help keybind enable
cybershot capture --help
```

Each command documents its purpose, arguments/options, and examples. Screen-level `capture full` and `capture region` remain available in the CLI, but the main TUI follows the workspace/window workflow described below.

## Keyboard controls

### Capture TUI

The capture interface follows a three-step workflow:
1. **Choose a workspace.** The first screen lists detected workspaces.
2. **Choose the capture target.** The next screen starts with **Capture entire workspace**, followed by the windows in that workspace.
3. **Choose the window mode.** For an individual window, choose **Full window** (include compositor borders), **Window content** (trim compositor borders), or **Page area** (draw a crop around the webpage viewport, excluding browser tabs and address bar).

Controls:
- `↑` / `↓`: navigate
- `Enter`: choose a workspace, capture target, or window mode
- `Backspace` / `Left`: return to the previous step
- `s`: open Settings from the workspace screen
- `d`: edit the screenshot directory in Settings
- `Esc`: go back / exit

**Content-only limitation:** compositor borders can be trimmed using the desktop's reported border width. Client-side decorations (for example, browser tabs, toolbar, or an application-drawn title bar) are rendered as part of the application surface and are not reliably distinguishable from the content area. Choose **Page area** to interactively draw the exact webpage viewport crop on the frozen desktop image.

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

The Linux installer checks for existing `cybershot`/`csb` commands and known desktop shortcut or shell-alias references before installing. It asks which command name(s) to install: `cybershot`, `csb`, or both. Existing files are never replaced without confirmation. Normally it builds the release binary; set `CYBERSHOT_SKIP_BUILD=1` to reuse an already-built `target/release/cybershot`. On Linux, GTK4 development files are required to build the frozen-frame region selector. Ensure `~/.local/bin` is on your `PATH`.

On macOS, run `./install-macos.sh`; on Windows, run `./install.ps1` from PowerShell. These build the native binary and install `cybershot`, `csb`, or both. The matching uninstallers are `uninstall-macos.sh` and `uninstall.ps1`. They preserve settings and screenshots. The Linux `uninstall.sh` also removes only the CyberShot-managed Hyprland shortcut block if you approve it; it creates another backup and never deletes unrelated bindings.

## Keyboard shortcuts

`csb hotkey` is the common entry point for a desktop shortcut. Enable its trigger first:

```sh
csb keybind enable --shortcut "Super+Shift+C"
csb keybind status
```

The optional `--shortcut` value is a label saved in CyberShot; it does not itself register a physical key combination. `csb keybind disable` safely disables the trigger without removing the operating-system shortcut.

### Linux / Hyprland

During installation, if a Hyprland config is detected, the installer offers to add `SUPER SHIFT, C` (or the shortcut you enter). It supports both `hyprland.lua` and traditional `hyprland.conf`, creates a timestamped backup, adds a clearly marked CyberShot-owned block, checks for a possible existing binding, and asks before making changes. It never rewrites the whole config. If the chosen key is already bound, installation stops without changing that binding; choose another shortcut and rerun the installer. The uninstaller can remove only the marked CyberShot block.

On GNOME, KDE Plasma, Sway, i3, XFCE, and other desktops, use the desktop's custom-shortcut settings and set the command to the absolute path to `csb hotkey` (for example `~/.local/bin/csb hotkey`). Then run `csb keybind enable`. CyberShot does not silently edit those desktop-specific settings.

### Windows

Create a Windows shortcut targeting the installed `cybershot.exe` or `csb.exe` with the argument `hotkey`. Set its Shortcut key in the shortcut's Properties, then run `csb.exe keybind enable`.

### macOS

Create an Automator Quick Action that runs the installed executable with the `hotkey` argument, then assign it in System Settings → Keyboard → Keyboard Shortcuts. Run `csb keybind enable`. Ensure the Quick Action uses the full executable path.

### TTY / headless sessions

The CLI and terminal interfaces remain usable without a desktop shortcut. A pure TTY has no graphical display on which to show an interactive region-selection overlay, so global graphical region shortcuts require a graphical session.

**Capture behavior:** on Linux Wayland, `csb hotkey` launches the same frozen-frame selection overlay as `csb select`; other supported capture backends use their native region selector.

## Configuration and screenshots

- Default screenshot folder: the platform Pictures directory under `CyberShot` (for example `~/Pictures/CyberShot`).
- Change the folder in the Capture TUI: press `s`, then `d`.
- Configuration file: the platform configuration directory under `cybershot/config.toml`.

## GitHub Actions

- **CI** runs formatting, `cargo check --locked --all-targets`, and `cargo test --locked` on Ubuntu, Windows, and macOS.
- **Release** builds native binaries when a tag matching `v*` is pushed and attaches the artifacts to a GitHub Release.

A green CI run verifies compilation on the corresponding GitHub-hosted runners. It does not replace interactive testing on each desktop session, particularly Wayland compositor permissions and macOS Accessibility/Screen Recording permissions.

## Current scope

- Native workspace switching is implemented for Hyprland and X11. On Windows and macOS, CyberShot reports the current desktop and captures it; switching between virtual desktops/Spaces is not implemented.
- Generic non-Hyprland Wayland window discovery is not available because Wayland does not provide a universal window-enumeration protocol. Full-screen and region capture remain available through `grim`/`slurp`.
- Automated global shortcut installation currently targets Hyprland only. Other desktop environments and OSes use their native shortcut settings as documented above.
- The file-manager copy buffer is internal to the active TUI session.

## Contributing and bug reports

Please include your OS, desktop/session (Hyprland, X11, GNOME Wayland, KDE Wayland, etc.), CyberShot version, command used, and the complete error output when opening an issue:

<https://github.com/zaynkhan100110/CyberShot/issues>

## License

MIT. See [LICENSE](LICENSE).
