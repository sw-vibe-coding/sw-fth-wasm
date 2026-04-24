#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v rustup >/dev/null 2>&1; then
  echo "error: rustup not found" >&2
  exit 1
fi

if ! rustup target list --installed | grep -q '^wasm32-unknown-unknown$'; then
  echo "Installing wasm32-unknown-unknown target..."
  rustup target add wasm32-unknown-unknown
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "error: wasm-bindgen CLI not found" >&2
  echo "install with: cargo install wasm-bindgen-cli" >&2
  exit 1
fi

echo "Building Rust library for wasm32-unknown-unknown..."
cargo build --target wasm32-unknown-unknown --release

echo "Generating browser bindings with wasm-bindgen..."
mkdir -p web/pkg
wasm-bindgen \
  --target web \
  --out-dir web/pkg \
  target/wasm32-unknown-unknown/release/proto_forth_wasm.wasm

echo "Done. Serve the web directory with a static file server, for example:"
echo "  cd web && python3 -m http.server 8000"
