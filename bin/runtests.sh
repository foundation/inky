#!/bin/bash
# Run all Inky test suites.
# Usage: bin/runtests.sh

cd "$(dirname "$0")/.."

failed=0

echo "=== Formatting Check ==="
if cargo fmt --all -- --check 2>&1; then
  echo "  OK"
else
  echo "  Run 'cargo fmt --all' to fix"
  failed=1
fi
echo ""

echo "=== Clippy ==="
if cargo clippy --workspace -- -D warnings 2>&1; then
  echo "  OK"
else
  failed=1
fi
echo ""

echo "=== Rust Unit & Fixture Tests ==="
if cargo test --workspace 2>&1; then
  echo ""
else
  failed=1
fi

echo "=== Building Release CLI ==="
if ! cargo build -p inky-cli --release 2>&1; then
  echo "Build failed — skipping remaining tests"
  exit 1
fi
echo ""

echo "=== Template Comparison Tests ==="
if node tests/compare.js; then
  echo ""
else
  failed=1
fi

echo "=== CLI Integration Tests ==="
if bash tests/test-cli.sh; then
  echo ""
else
  failed=1
fi

if command -v wasm-pack &> /dev/null; then
  echo "=== WASM Build & Node Tests ==="
  if wasm-pack build crates/inky-wasm --target nodejs --out-dir ../../bindings/node 2>&1; then
    if node bindings/node/test.js; then
      echo ""
    else
      failed=1
    fi
  else
    failed=1
  fi
else
  echo "=== Skipping WASM tests (wasm-pack not installed) ==="
  echo ""
fi

if [ $failed -eq 0 ]; then
  echo "=== All test suites passed ==="
else
  echo "=== Some test suites failed ==="
  exit 1
fi
