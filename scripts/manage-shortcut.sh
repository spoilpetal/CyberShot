#!/usr/bin/env bash
# Manage only the CyberShot-owned Hyprland shortcut block.
set -euo pipefail

ACTION="${1:-status}"
COMMAND_PATH="${2:-}"
SHORTCUT="${3:-SUPER SHIFT, C}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
HYPR_DIR="$CONFIG_HOME/hypr"
LUA_FILE="$HYPR_DIR/hyprland.lua"
CONF_FILE="$HYPR_DIR/hyprland.conf"
remove_block() {
  local file="$1" begin_marker end_marker
  [[ -f "$file" ]] || return 0
  if [[ "$file" == *.lua ]]; then
    begin_marker="-- BEGIN CYBERSHOT MANAGED SHORTCUT"
    end_marker="-- END CYBERSHOT MANAGED SHORTCUT"
  else
    begin_marker="# BEGIN CYBERSHOT MANAGED SHORTCUT"
    end_marker="# END CYBERSHOT MANAGED SHORTCUT"
  fi
  if grep -Fq -- "$begin_marker" "$file" || grep -Fq -- "$end_marker" "$file"; then
    if ! grep -Fq -- "$begin_marker" "$file" || ! grep -Fq -- "$end_marker" "$file"; then
      echo "Incomplete CyberShot marker pair in $file; refusing to edit it." >&2
      return 1
    fi
    local tmp
    tmp="$(mktemp)"
    awk -v begin="$begin_marker" -v end="$end_marker" '
      $0 == begin { skip=1; next }
      $0 == end { skip=0; next }
      !skip { print }
    ' "$file" > "$tmp"
    cat "$tmp" > "$file"
    rm -f "$tmp"
    echo "Removed CyberShot shortcut block from $file"
  fi
}

backup_file() {
  local file="$1"
  [[ -f "$file" ]] || return 0
  local backup="${file}.cybershot-backup-$(date +%Y%m%d-%H%M%S)-$$"
  cp -p -- "$file" "$backup"
  echo "Backup created: $backup"
}

file_has_pattern_outside_managed_block() {
  local file="$1" pattern="$2" begin_marker end_marker
  [[ -f "$file" ]] || return 1
  if [[ "$file" == *.lua ]]; then
    begin_marker="-- BEGIN CYBERSHOT MANAGED SHORTCUT"
    end_marker="-- END CYBERSHOT MANAGED SHORTCUT"
  else
    begin_marker="# BEGIN CYBERSHOT MANAGED SHORTCUT"
    end_marker="# END CYBERSHOT MANAGED SHORTCUT"
  fi
  awk -v begin="$begin_marker" -v end="$end_marker" '
    $0 == begin { skip=1; next }
    $0 == end { skip=0; next }
    !skip { print }
  ' "$file" | grep -Ei -- "$pattern" >/dev/null
}

case "$ACTION" in
  remove)
    if grep -Fq -- "-- BEGIN CYBERSHOT MANAGED SHORTCUT" "$LUA_FILE" 2>/dev/null; then backup_file "$LUA_FILE"; fi
    if grep -Fq "# BEGIN CYBERSHOT MANAGED SHORTCUT" "$CONF_FILE" 2>/dev/null; then backup_file "$CONF_FILE"; fi
    remove_block "$LUA_FILE"
    remove_block "$CONF_FILE"
    ;;
  status)
    if grep -Fq -- "-- BEGIN CYBERSHOT MANAGED SHORTCUT" "$LUA_FILE" 2>/dev/null; then
      echo "Managed Hyprland Lua shortcut: $LUA_FILE"
    elif grep -Fq "# BEGIN CYBERSHOT MANAGED SHORTCUT" "$CONF_FILE" 2>/dev/null; then
      echo "Managed Hyprland config shortcut: $CONF_FILE"
    else
      echo "No CyberShot-managed Hyprland shortcut found."
    fi
    ;;
  install)
    [[ -n "$COMMAND_PATH" ]] || { echo "Missing absolute CyberShot executable path" >&2; exit 2; }
    [[ -x "$COMMAND_PATH" ]] || { echo "Executable not found: $COMMAND_PATH" >&2; exit 2; }
    [[ -d "$HYPR_DIR" ]] || { echo "Hyprland config directory not found: $HYPR_DIR" >&2; exit 3; }

    # Normalize the common user-facing form SUPER+SHIFT+C to Hyprland's config form.
    SHORTCUT="${SHORTCUT//+/ }"
    SHORTCUT="$(printf '%s' "$SHORTCUT" | xargs)"
    if [[ "$SHORTCUT" =~ ^([[:alnum:]_]+)[[:space:]]+([[:alnum:]_]+)[[:space:]]*,?[[:space:]]*([[:alnum:]_]+)$ ]]; then
      MOD1="${BASH_REMATCH[1]}"; MOD2="${BASH_REMATCH[2]}"; KEY="${BASH_REMATCH[3]}"
    else
      echo "Use a shortcut like SUPER SHIFT C or SUPER+SHIFT+C." >&2; exit 2
    fi
    CONF_SHORTCUT="$MOD1 $MOD2, $KEY"
    LUA_SHORTCUT="$MOD1 + $MOD2 + $KEY"

    if [[ -f "$LUA_FILE" ]]; then
      if file_has_pattern_outside_managed_block "$LUA_FILE" "hl\.bind.*${MOD2}.*${KEY}"; then
        echo "A possible conflicting Lua binding for $LUA_SHORTCUT was found in $LUA_FILE. No changes made." >&2
        exit 4
      fi
      backup_file "$LUA_FILE"
      backup_file "$CONF_FILE"
      remove_block "$LUA_FILE"
      remove_block "$CONF_FILE"
      LUA_CMD="$(printf '%s' "$COMMAND_PATH" | sed 's/\\/\\\\/g; s/"/\\"/g') hotkey"
      {
        printf '\n%s\n' "-- BEGIN CYBERSHOT MANAGED SHORTCUT"
        printf 'hl.bind("%s", hl.dsp.exec_cmd("%s"))\n' "$LUA_SHORTCUT" "$LUA_CMD"
        printf '%s\n' "-- END CYBERSHOT MANAGED SHORTCUT"
      } >> "$LUA_FILE"
      echo "Installed $LUA_SHORTCUT in $LUA_FILE"
    elif [[ -f "$CONF_FILE" ]]; then
      if file_has_pattern_outside_managed_block "$CONF_FILE" "^[[:space:]]*bind[^#]*${CONF_SHORTCUT//,/,[[:space:]]*}([^[:alnum:]_]|$)"; then
        echo "The exact shortcut $CONF_SHORTCUT is already present in $CONF_FILE. No changes made." >&2
        exit 4
      fi
      backup_file "$LUA_FILE"
      backup_file "$CONF_FILE"
      remove_block "$LUA_FILE"
      remove_block "$CONF_FILE"
      {
        printf '\n%s\n' "# BEGIN CYBERSHOT MANAGED SHORTCUT"
        printf 'bind = %s, exec, "%s" hotkey\n' "$CONF_SHORTCUT" "$COMMAND_PATH"
        printf '%s\n' "# END CYBERSHOT MANAGED SHORTCUT"
      } >> "$CONF_FILE"
      echo "Installed $CONF_SHORTCUT in $CONF_FILE"
    else
      echo "No hyprland.lua or hyprland.conf found in $HYPR_DIR." >&2
      exit 3
    fi
    ;;
  *) echo "Usage: $0 {install|remove|status} [absolute-binary] [SUPER SHIFT C]" >&2; exit 2 ;;
esac
