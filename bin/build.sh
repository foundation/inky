#!/bin/bash
# Build all Inky artifacts.
# Usage: bin/build.sh [--release]

cd "$(dirname "$0")/.."

PROFILE="--release"
if [ "$1" = "--debug" ]; then
  PROFILE=""
fi

failed=0

echo "=== Building Rust CLI ==="
if ! cargo build -p inky-cli $PROFILE 2>&1; then
  echo "Rust build failed"
  failed=1
fi

echo ""
echo "=== Compiling SCSS ==="
mkdir -p build
if sass scss/foundation-emails.scss build/foundation-emails.css 2>&1; then
  echo "  -> build/foundation-emails.css"
else
  echo "SCSS build failed"
  failed=1
fi

if [ $failed -eq 0 ]; then
  echo ""
  echo "=== Build complete ==="
else
  echo ""
  echo "=== Build failed ==="
  exit 1
fi
