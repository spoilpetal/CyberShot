#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${CYBERSHOT_BIN_DIR:-$HOME/.local/bin}"
BIN="$ROOT/target/release/cybershot"

ask_yes_no() {
  local prompt="$1" reply
  read -r -p "$prompt [y/N] " reply
  [[ "$reply" =~ ^[Yy]$ ]]
}

for tool in cargo rustc; do
  command -v "$tool" >/dev/null 2>&1 || { echo "Error: $tool is required (install Rust with rustup)." >&2; exit 1; }
done

if [[ "$(uname -s)" == "Linux" ]]; then
  if ! command -v pkg-config >/dev/null 2>&1 || ! pkg-config --exists gtk4; then
    cat >&2 <<'MSG'
Error: GTK4 development files are required to build CyberShot's region selector.
On Arch Linux: sudo pacman -S --needed gtk4 pkgconf
On Debian/Ubuntu: sudo apt install libgtk-4-dev pkg-config
On Fedora: sudo dnf install gtk4-devel pkgconf-pkg-config
MSG
    exit 1
  fi
  if [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
    command -v grim >/dev/null 2>&1 || echo "Note: install grim for Wayland screenshots."
    command -v slurp >/dev/null 2>&1 || echo "Note: install slurp for Wayland region capture."
  elif [[ -n "${DISPLAY:-}" ]]; then
    command -v import >/dev/null 2>&1 || echo "Note: install ImageMagick for X11 screenshots."
    command -v slop >/dev/null 2>&1 || echo "Note: install slop for X11 region selection."
    command -v xclip >/dev/null 2>&1 || echo "Note: install xclip for X11 clipboard support."
    command -v wmctrl >/dev/null 2>&1 || echo "Note: install wmctrl for X11 workspace/window discovery."
  fi
fi

mkdir -p "$BIN_DIR"

cat <<'CHOICE'
Which command name(s) should be installed?
  1) cybershot only
  2) csb only
  3) both cybershot and csb
  4) cancel
CHOICE
read -r -p "Choose [1-4]: " choice
case "$choice" in
  1) names=(cybershot) ;;
  2) names=(csb) ;;
  3) names=(cybershot csb) ;;
  *) echo "Installation cancelled."; exit 0 ;;
esac

# Check for pre-existing commands before building or replacing anything.
conflicts=()
for name in "${names[@]}"; do
  path="$BIN_DIR/$name"
  if [[ -e "$path" || -L "$path" ]]; then
    if [[ "$path" -ef "$BIN" ]]; then
      echo "Found existing $name at $path (already points to this build)."
    else
      conflicts+=("$name:$path")
    fi
  elif command -v "$name" >/dev/null 2>&1; then
    found="$(command -v "$name")"
    if [[ "$found" != "$path" ]]; then
      echo "Notice: '$name' is already provided by $found (outside $BIN_DIR); it will not be modified."
    fi
  fi
done

# Report existing desktop shortcut/keybinding references without silently editing them.
keybind_hits=()
for file in \
  "$HOME/.config/hypr/hyprland.conf" \
  "$HOME/.config/sway/config" \
  "$HOME/.config/i3/config" \
  "$HOME/.config/sxhkd/sxhkdrc" \
  "$HOME/.config/kglobalshortcutsrc" \
  "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.config/fish/config.fish"; do
  if [[ -f "$file" ]] && grep -nEi 'cybershot|(^|[^[:alnum:]_])csb([^[:alnum:]_]|$)' "$file" >/dev/null 2>&1; then
    keybind_hits+=("$file")
  fi
done
if ((${#keybind_hits[@]})); then
  echo "Existing desktop shortcut configuration mentions CyberShot/csb:"
  for file in "${keybind_hits[@]}"; do
    echo "--- $file"
    grep -nEi 'cybershot|(^|[^[:alnum:]_])csb([^[:alnum:]_]|$)' "$file" || true
  done
  echo "The installer will not alter keybindings. Review these entries if they conflict."
  ask_yes_no "Continue without changing the existing keybindings?" || { echo "Installation cancelled."; exit 1; }
fi

if ((${#conflicts[@]})); then
  echo "An existing command with the same name was found:"
  for item in "${conflicts[@]}"; do echo "  ${item#*:}"; done
  echo "CyberShot will not replace these files unless you explicitly approve."
  ask_yes_no "Allow CyberShot to replace the conflicting file(s) in $BIN_DIR?" || { echo "Installation cancelled; existing commands were left untouched."; exit 1; }
fi

if [[ "${CYBERSHOT_SKIP_BUILD:-0}" == "1" ]]; then
  [[ -x "$BIN" ]] || { echo "Error: CYBERSHOT_SKIP_BUILD=1 was set, but $BIN does not exist. Build CyberShot first." >&2; exit 1; }
  echo "Using the existing release binary at $BIN (no build was run)."
else
  cargo build --release --manifest-path "$ROOT/Cargo.toml"
  [[ -x "$BIN" ]] || { echo "Error: build did not produce $BIN" >&2; exit 1; }
fi

if [[ " ${names[*]} " == *" cybershot "* ]]; then
  install -Dm755 "$BIN" "$BIN_DIR/cybershot"
fi
if [[ " ${names[*]} " == *" csb "* ]]; then
  if [[ " ${names[*]} " == *" cybershot "* ]]; then
    ln -sfn "$BIN_DIR/cybershot" "$BIN_DIR/csb"
  else
    # A standalone copy avoids a dangling csb symlink when cybershot was not selected.
    install -Dm755 "$BIN" "$BIN_DIR/csb"
  fi
fi

echo "CyberShot installation finished. Selected command(s) are in $BIN_DIR."
case ":$PATH:" in *":$BIN_DIR:"*) ;; *) echo "Add this to your shell profile if needed: export PATH=\"$BIN_DIR:\$PATH\"" ;; esac

# Offer a real Hyprland shortcut integration where a supported config exists.
if [[ -f "${XDG_CONFIG_HOME:-$HOME/.config}/hypr/hyprland.lua" || -f "${XDG_CONFIG_HOME:-$HOME/.config}/hypr/hyprland.conf" ]]; then
  echo
  echo "Hyprland configuration detected."
  if ask_yes_no "Install a CyberShot area-selection shortcut (default SUPER+SHIFT+C)?"; then
    read -r -p "Shortcut [SUPER SHIFT C]: " shortcut
    shortcut="${shortcut:-SUPER SHIFT C}"
    executable="$BIN_DIR/cybershot"
    [[ -x "$BIN_DIR/csb" ]] && executable="$BIN_DIR/csb"
    if "$ROOT/scripts/manage-shortcut.sh" install "$executable" "$shortcut"; then
      "$executable" keybind enable --shortcut "$shortcut"
      if command -v hyprctl >/dev/null 2>&1; then
        hyprctl reload >/dev/null 2>&1 || echo "Note: reload Hyprland configuration manually."
      fi
      echo "Shortcut installed. It launches the same region selector as 'csb select'."
    else
      echo "Shortcut was not installed. Existing configuration was left unchanged."
    fi
  else
    echo "Skipped shortcut integration. You can configure your desktop shortcut manually; command: csb hotkey"
  fi
elif [[ "$(uname -s)" == "Linux" ]]; then
  shortcut_executable="$BIN_DIR/cybershot"
  [[ -x "$BIN_DIR/csb" ]] && shortcut_executable="$BIN_DIR/csb"
  echo "No Hyprland config detected. No global shortcut was changed."
  echo "For GNOME/KDE/other desktops, create a custom shortcut running: $shortcut_executable hotkey"
  echo "Enable its trigger with: $shortcut_executable keybind enable"
fi
