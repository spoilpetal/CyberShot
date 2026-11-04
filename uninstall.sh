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

remove_command() {
  local name="$1" path="$BIN_DIR/$1"
  if [[ -e "$path" || -L "$path" ]]; then
    rm -f -- "$path"
    echo "Removed $name ($path)."
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
    ask_yes_no "Continue with full uninstall?" || { echo "Cancelled; nothing was removed."; exit 0; }
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
