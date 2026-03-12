#!/bin/bash
# Run all Inky test suites.
# Usage: bin/runtests.sh

cd "$(dirname "$0")/.."

failed=0

echo "=== Rust Unit & Fixture Tests ==="
if cargo test -p inky-core 2>&1; then
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

if [ $failed -eq 0 ]; then
  echo "=== All test suites passed ==="
else
  echo "=== Some test suites failed ==="
  exit 1
fi
