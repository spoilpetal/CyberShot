<h1 align="center">CyberShot</h1>

<p align="center">
  <strong>A terminal-first screenshot tool built with Rust</strong><br>
  Capture · History · File Manager · Clipboard
</p>

<p align="center">
  <a href="https://github.com/zaynkhan100110/CyberShot">GitHub</a> ·
  <a href="https://github.com/zaynkhan100110/CyberShot/issues">Issues</a> ·
  <a href="https://github.com/zaynkhan100110/CyberShot/releases">Releases</a>
</p>

CyberShot is a standalone, keyboard-driven screenshot utility. It has two application TUIs: a combined capture/settings interface and a screenshot History. The `files` command opens a separate file-manager interface.

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



## ScreenShot (using <a href="https://github.com/zaynkhan100110/CyberShot">CyberShot</a>)


## System File Manager

![File Manager](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberShot/CyberShot-0.2.0/file_manager.png)

- [About History](#general)

### Tui History   

![History](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberShot/CyberShot-0.2.0/tui-history.png)

### Workplace Show

![Workplace](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberShot/CyberShot-0.2.0/workspace_show.png)
 
### Workplace window show

![Window](https://raw.githubusercontent.com/spoilpetal/Picture_Vault/main/CyberShot/CyberShot-0.2.0/window_show.png)



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
| `csb keybind enable [--shortcut "..."]` | Enable the externally assigned shortcut trigger |
| `csb keybind disable` | Disable the shortcut trigger without removing the OS shortcut |
| `csb keybind status` | Show shortcut-trigger state |
| `csb hotkey` | Run region capture when the shortcut trigger is enabled |

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

## Keyboard shortcut toggle (all supported platforms)

CyberShot provides one cross-platform command for a desktop/OS keyboard shortcut: `csb hotkey`. The operating system or desktop environment registers the physical key combination; CyberShot does not install a background hotkey daemon or silently edit system shortcut settings.

First enable the trigger:

```sh
csb keybind enable --shortcut "Super+Shift+S"
csb keybind status
```

Use your preferred shortcut label in place of the example. Then create a custom shortcut in your desktop/OS settings and set its command to `csb hotkey` (or the full path to the executable if `csb` is not on the PATH). The shortcut launches the existing native region selector. To temporarily stop it working without deleting the OS shortcut, run:

```sh
csb keybind disable
```

Turn it back on with `csb keybind enable`. The optional `--shortcut` value is a display label only; it does not register or validate a physical key combination.

### Linux

- **Hyprland (traditional hyprland.conf):** add a bind such as `bind = SUPER SHIFT, S, exec, csb hotkey`, then reload the configuration. If your Hyprland setup uses the Lua config provider, add the equivalent command binding using the binding API for your installed config; the traditional line above is not Lua syntax.
- **GNOME:** Settings → Keyboard → View and Customize Shortcuts → Custom Shortcuts. Add a shortcut with command `csb hotkey`, then assign the desired key combination.
- **KDE Plasma:** System Settings → Keyboard → Shortcuts → Add New → Command or Script. Set the command to `csb hotkey` and assign a key combination.
- **Other desktop environments:** use the desktop's custom-shortcut settings and launch `csb hotkey`.

### Windows

Create a Windows shortcut whose target runs `csb hotkey` (or the full path to `cybershot.exe` followed by `hotkey`). Open the shortcut's Properties and set its **Shortcut key**. Windows may require the shortcut to be on the Desktop or in the Start Menu for the shortcut key to work. Enable the CyberShot trigger with `csb keybind enable` first.

### macOS

Create an Automator Quick Action that receives no input and runs the shell command `csb hotkey`. Save it, then open System Settings → Keyboard → Keyboard Shortcuts → Services and assign a shortcut to that Quick Action. Enable the CyberShot trigger with `csb keybind enable` first. Ensure the command is available in the shell PATH used by the Quick Action; otherwise use the full executable path.

**Important:** the CLI toggle controls whether `csb hotkey` performs a capture. It does not create, remove, or globally register the physical key combination. That registration remains under your control in the platform's own shortcut settings.

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
- Global shortcut registration is intentionally managed by the user in the operating system/desktop. Configure it to launch `csb hotkey`, then use `csb keybind enable` / `disable` to control the trigger.
- The file-manager copy buffer is internal to the active TUI session.

## General
- Why history don't show any logs ? <br>
 Ans.The ``History`` only records when you capture using cli
- Where does my captured saved ? <br>
 Ans.The screenshot saved inside the same directory from where you give commands in cli or capture using Tui a folder named ```CyberShot```
- Why the `File Manager` isn't behaving like [CyberClip](https://github.com/spoilpetal/CyberClip) ? </br>
 Ans. The `File Manager` inside [CyberShot](https://github.com/zaynkhan100110/CyberShot) is specifically designed for preview image files in full screen. But this have some basic functionality 


## Contributing and bug reports

Please include your OS, desktop/session (Hyprland, X11, GNOME Wayland, KDE Wayland, etc.), CyberShot version, command used, and the complete error output when opening an issue:

<https://github.com/zaynkhan100110/CyberShot/issues>

## License

MIT. See [LICENSE](LICENSE).
