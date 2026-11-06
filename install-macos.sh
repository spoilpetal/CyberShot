#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${CYBERSHOT_BIN_DIR:-$HOME/.local/bin}"
mkdir -p "$BIN_DIR"
command -v cargo >/dev/null 2>&1 || { echo "Install Rust first: https://rustup.rs/" >&2; exit 1; }
cargo build --locked --release --manifest-path "$ROOT/Cargo.toml"
BIN="$ROOT/target/release/cybershot"
[[ -x "$BIN" ]] || { echo "Build did not produce $BIN" >&2; exit 1; }
read -r -p "Install command names: 1) cybershot  2) csb  3) both [3]: " choice
choice="${choice:-3}"
case "$choice" in
  1) targets=("$BIN_DIR/cybershot") ;;
  2) targets=("$BIN_DIR/csb") ;;
  3) targets=("$BIN_DIR/cybershot" "$BIN_DIR/csb") ;;
  *) echo "Cancelled."; exit 0 ;;
esac
conflicts=()
for target in "${targets[@]}"; do [[ -e "$target" || -L "$target" ]] && conflicts+=("$target"); done
if ((${#conflicts[@]})); then
  printf 'Existing command files found:\n'; printf '  %s\n' "${conflicts[@]}"
  read -r -p 'Replace these files? [y/N] ' confirm
  [[ "$confirm" =~ ^[Yy]$ ]] || { echo 'Cancelled; existing files preserved.'; exit 0; }
fi
case "$choice" in
  1) install -m755 "$BIN" "$BIN_DIR/cybershot" ;;
  2) install -m755 "$BIN" "$BIN_DIR/csb" ;;
  3) install -m755 "$BIN" "$BIN_DIR/cybershot"; ln -sfn "$BIN_DIR/cybershot" "$BIN_DIR/csb" ;;
esac
case ":$PATH:" in *":$BIN_DIR:"*) ;; *) echo "Add to PATH: export PATH=\"$BIN_DIR:\$PATH\"" ;; esac
echo "Installed. Configure a macOS Keyboard Shortcut/Automator Quick Action to run: $BIN_DIR/csb hotkey"
echo "Enable the trigger with: $BIN_DIR/csb keybind enable"
