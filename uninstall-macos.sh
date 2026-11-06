#!/usr/bin/env bash
set -euo pipefail
BIN_DIR="${CYBERSHOT_BIN_DIR:-$HOME/.local/bin}"
read -r -p "Remove cybershot, csb, or both? [both/cybershot/csb/cancel] " choice
choice_lc="$(printf '%s' "$choice" | tr '[:upper:]' '[:lower:]')"
case "$choice_lc" in
  both) rm -f -- "$BIN_DIR/cybershot" "$BIN_DIR/csb" ;;
  cybershot)
    rm -f -- "$BIN_DIR/cybershot"
    if [[ -L "$BIN_DIR/csb" && "$(readlink "$BIN_DIR/csb")" == "$BIN_DIR/cybershot" ]]; then rm -f -- "$BIN_DIR/csb"; fi
    ;;
  csb) rm -f -- "$BIN_DIR/csb" ;;
  *) echo "Cancelled."; exit 0 ;;
esac
echo "Selected command(s) removed. Settings and screenshots were preserved."
