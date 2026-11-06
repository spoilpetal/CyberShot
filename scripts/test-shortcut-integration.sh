#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANAGER="$ROOT/scripts/manage-shortcut.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/config/hypr" "$TMP/bin"
printf '#!/usr/bin/env sh\nexit 0\n' > "$TMP/bin/csb"
chmod +x "$TMP/bin/csb"

# Lua config: preserve user bindings and remove only the marked block.
printf 'local mainMod = "SUPER"\nhl.bind(mainMod .. " + C", hl.dsp.window.close())\n' > "$TMP/config/hypr/hyprland.lua"
XDG_CONFIG_HOME="$TMP/config" "$MANAGER" install "$TMP/bin/csb" 'SUPER+SHIFT+C'
grep -Fq 'hl.bind("SUPER + SHIFT + C"' "$TMP/config/hypr/hyprland.lua"
XDG_CONFIG_HOME="$TMP/config" "$MANAGER" status | grep -q 'Managed Hyprland Lua'
XDG_CONFIG_HOME="$TMP/config" "$MANAGER" remove
grep -Fq 'hl.bind(mainMod .. " + C"' "$TMP/config/hypr/hyprland.lua"
! grep -q 'CYBERSHOT MANAGED' "$TMP/config/hypr/hyprland.lua"

# Re-running is safe; a user conflict must not remove an existing managed block.
XDG_CONFIG_HOME="$TMP/config" "$MANAGER" install "$TMP/bin/csb" 'SUPER+SHIFT+C'
printf 'hl.bind(mainMod .. " + SHIFT + C", hl.dsp.window.move())\n' >> "$TMP/config/hypr/hyprland.lua"
if XDG_CONFIG_HOME="$TMP/config" "$MANAGER" install "$TMP/bin/csb" 'SUPER+SHIFT+C'; then
  echo 'Expected collision against an existing managed install was not detected.' >&2
  exit 1
fi
grep -Fq -- '-- BEGIN CYBERSHOT MANAGED SHORTCUT' "$TMP/config/hypr/hyprland.lua"

# Existing Lua shortcut collision: no managed entry is added.
printf 'hl.bind(mainMod .. " + SHIFT + C", hl.dsp.window.move())\n' > "$TMP/config/hypr/hyprland.lua"
if XDG_CONFIG_HOME="$TMP/config" "$MANAGER" install "$TMP/bin/csb" 'SUPER+SHIFT+C'; then
  echo 'Expected Lua key conflict was not detected.' >&2
  exit 1
fi
! grep -q 'CYBERSHOT MANAGED' "$TMP/config/hypr/hyprland.lua"

# Traditional config: valid comment markers and exact cleanup.
rm -f "$TMP/config/hypr/hyprland.lua"
printf '# user config\n' > "$TMP/config/hypr/hyprland.conf"
XDG_CONFIG_HOME="$TMP/config" "$MANAGER" install "$TMP/bin/csb" 'SUPER SHIFT C'
grep -Fq '# BEGIN CYBERSHOT MANAGED SHORTCUT' "$TMP/config/hypr/hyprland.conf"
XDG_CONFIG_HOME="$TMP/config" "$MANAGER" remove
grep -Fq '# user config' "$TMP/config/hypr/hyprland.conf"
! grep -q 'CYBERSHOT MANAGED' "$TMP/config/hypr/hyprland.conf"

echo 'Shortcut integration tests passed.'
