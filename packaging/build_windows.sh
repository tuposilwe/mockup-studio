#!/usr/bin/env bash
# Cross-compiles the Windows build and packages it as an NSIS installer.
# Run from macOS with the Windows GNU target + mingw-w64 + NSIS installed:
#   rustup target add x86_64-pc-windows-gnu
#   brew install mingw-w64 makensis
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Cross-compiling release binary for x86_64-pc-windows-gnu"
cargo build --release --target x86_64-pc-windows-gnu

echo "==> Building NSIS installer"
cd packaging
makensis installer.nsi

echo "==> Done"
echo "Installer: packaging/MockupStudio-Setup.exe"
