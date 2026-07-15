#!/usr/bin/env bash
# Builds the Quarkup compiler as a WebAssembly module and regenerates the
# JS bindings under web/pkg/, consumed by web/index.html.
set -euo pipefail

cd "$(dirname "$0")/.."

if ! command -v wasm-bindgen >/dev/null 2>&1; then
    echo "error: wasm-bindgen CLI not found." >&2
    echo "  Install it with: cargo install wasm-bindgen-cli --version <version>" >&2
    echo "  <version> must match the wasm-bindgen crate version in Cargo.toml exactly." >&2
    exit 1
fi

# If a Homebrew-installed `cargo`/`rustc` shadows rustup's shims on PATH, it
# won't know about rustup-installed targets like wasm32-unknown-unknown even
# though `rustup target list --installed` reports it. Prefer rustup's own
# toolchain binaries for this build when available.
if command -v rustup >/dev/null 2>&1; then
    RUSTUP_TOOLCHAIN_BIN="$(rustup which rustc 2>/dev/null | xargs dirname 2>/dev/null || true)"
    if [ -n "$RUSTUP_TOOLCHAIN_BIN" ] && [ -x "$RUSTUP_TOOLCHAIN_BIN/rustc" ]; then
        export RUSTC="$RUSTUP_TOOLCHAIN_BIN/rustc"
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
fi

cargo build --release --target wasm32-unknown-unknown --lib

wasm-bindgen --target web --out-dir web/pkg \
    target/wasm32-unknown-unknown/release/quarkup.wasm

echo "Wasm module written to web/pkg/. Serve web/ with a local HTTP server, e.g.:"
echo "  python3 -m http.server -d web 8080"
