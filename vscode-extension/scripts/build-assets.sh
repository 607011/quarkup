#!/usr/bin/env bash
# Builds the Quarkup wasm compiler (reusing web/build.sh) and copies the
# resulting bindings, plus the vendored KaTeX assets, into media/ so the
# extension's preview webview can load them without any network access.
set -euo pipefail

cd "$(dirname "$0")/../.."

./web/build.sh

rm -rf vscode-extension/media/pkg vscode-extension/media/vendor
mkdir -p vscode-extension/media
cp -R web/pkg vscode-extension/media/pkg
cp -R web/vendor vscode-extension/media/vendor

echo "Assets copied into vscode-extension/media/."
