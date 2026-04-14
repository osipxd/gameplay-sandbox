#!/bin/sh
set -eu

TARGET=wasm32-unknown-unknown
PROFILE="${1:-release}"
OUT_DIR="web"
GAME_DIR="$OUT_DIR/game"
OUT_NAME="gameplay_sandbox"
WASM_PATH="target/$TARGET/$PROFILE/gameplay-sandbox.wasm"

case "$PROFILE" in
  debug|release) ;;
  *)
    echo "usage: $0 [debug|release]" >&2
    exit 1
    ;;
esac

if ! rustup target list --installed | grep -qx "$TARGET"; then
  echo "missing Rust target: rustup target add $TARGET" >&2
  exit 1
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "missing wasm-bindgen-cli: cargo install wasm-bindgen-cli" >&2
  exit 1
fi

if [ "$PROFILE" = "release" ]; then
  cargo build --release --target "$TARGET"
else
  cargo build --target "$TARGET"
fi

mkdir -p "$GAME_DIR" "$OUT_DIR/assets"
wasm-bindgen --target web --no-typescript --out-dir "$GAME_DIR" --out-name "$OUT_NAME" "$WASM_PATH"
cp -R assets/. "$OUT_DIR/assets/"

echo "web build ready in $OUT_DIR/"
