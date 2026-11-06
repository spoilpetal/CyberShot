#!/usr/bin/env bash
set -euo pipefail
BIN_DIR="${CYBERSHOT_BIN_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/cybershot"
DEFAULT_SCREENSHOTS="${XDG_PICTURES_DIR:-$HOME/Pictures}/CyberShot"

ask_yes_no() {
  local prompt="$1" reply
  read -r -p "$prompt [y/N] " reply
  [[ "$reply" =~ ^[Yy]$ ]]
}

cat <<'CHOICE'
CyberShot uninstall options:
  1) Remove only the 'cybershot' command
  2) Remove only the 'csb' command
  3) Remove both commands (keep settings and screenshots)
  4) Full uninstall (commands, settings, and optionally screenshots)
  5) Cancel
CHOICE
read -r -p "Choose [1-5]: " choice
if [[ "$choice" == "4" ]]; then
  ask_yes_no "Continue with full uninstall?" || { echo "Cancelled; nothing was removed."; exit 0; }
fi

if [[ "$choice" =~ ^[1-4]$ ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  if [[ -f "$SCRIPT_DIR/scripts/manage-shortcut.sh" ]]; then
    if "$SCRIPT_DIR/scripts/manage-shortcut.sh" status | grep -q "Managed Hyprland"; then
      echo "A CyberShot-managed Hyprland shortcut exists."
      if ask_yes_no "Remove only the CyberShot-managed shortcut entry?"; then
        "$SCRIPT_DIR/scripts/manage-shortcut.sh" remove
        if [[ -x "$BIN_DIR/csb" ]]; then "$BIN_DIR/csb" keybind disable >/dev/null 2>&1 || true;
        elif [[ -x "$BIN_DIR/cybershot" ]]; then "$BIN_DIR/cybershot" keybind disable >/dev/null 2>&1 || true; fi
        command -v hyprctl >/dev/null 2>&1 && hyprctl reload >/dev/null 2>&1 || true
      else
        echo "Shortcut entry preserved. It may stop working if its command is removed."
      fi
    fi
  fi
fi

remove_command() {
  local name="$1" path="$BIN_DIR/$1"
  if [[ -e "$path" || -L "$path" ]]; then
    rm -f -- "$path"
    echo "Removed $name ($path)."
    if [[ "$name" == "cybershot" && -L "$BIN_DIR/csb" && "$(readlink "$BIN_DIR/csb")" == "$BIN_DIR/cybershot" ]]; then
      rm -f -- "$BIN_DIR/csb"
      echo "Also removed csb because it was a symlink to the removed cybershot command."
    fi
  else
    echo "$name was not installed in $BIN_DIR."
  fi
}

case "$choice" in
  1) remove_command cybershot ;;
  2) remove_command csb ;;
  3) remove_command cybershot; remove_command csb
     echo "Commands removed. Settings and screenshots were preserved." ;;
  4)
    echo "Full uninstall removes both commands and CyberShot settings."
    remove_command cybershot
    remove_command csb
    if [[ -d "$CONFIG_DIR" ]] && ask_yes_no "Delete CyberShot settings at $CONFIG_DIR?"; then
      rm -rf -- "$CONFIG_DIR"
      echo "Settings removed."
    else
      echo "Settings preserved."
    fi
    if [[ -d "$DEFAULT_SCREENSHOTS" ]] && ask_yes_no "Delete screenshots in $DEFAULT_SCREENSHOTS? This cannot be undone."; then
      rm -rf -- "$DEFAULT_SCREENSHOTS"
      echo "Screenshots removed."
    else
      echo "Screenshots preserved."
    fi
    echo "Full uninstall choices completed."
    ;;
  *) echo "Uninstall cancelled; nothing was removed." ;;
esac
