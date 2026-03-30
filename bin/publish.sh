#!/bin/bash
# Publish Inky to all package registries.
# Run after the GitHub release workflow has completed.
# Each step prompts for confirmation and can be skipped.
#
# Usage: bin/publish.sh

set -e

cd "$(dirname "$0")/.."

VERSION=$(grep '^version' crates/inky-core/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
TAG="v${VERSION}"

echo "==> Publishing Inky $TAG"
echo ""

# Verify GitHub release exists
if ! gh release view "$TAG" > /dev/null 2>&1; then
    echo "Error: GitHub release $TAG not found."
    echo "Run bin/release.sh $VERSION first."
    exit 1
fi

confirm() {
    local prompt="$1"
    read -r -p "$prompt [y/N] " response
    [[ "$response" =~ ^[Yy]$ ]]
}

PUBLISHED=()
SKIPPED=()

# --- crates.io ---
echo "--- crates.io ---"
echo "  Packages: inky-core, inky-cli"
echo "  Note: CI may have already published these."
if confirm "  Publish to crates.io?"; then
    echo "  Publishing inky-core..."
    cargo publish -p inky-core
    echo "  Publishing inky-cli..."
    cargo publish -p inky-cli
    PUBLISHED+=("crates.io")
else
    SKIPPED+=("crates.io")
fi
echo ""

# --- npm ---
echo "--- npm ---"
echo "  Package: inky-wasm@$VERSION"
if confirm "  Publish to npm?"; then
    TMPDIR=$(mktemp -d)
    echo "  Downloading WASM artifact from GitHub release..."
    gh release download "$TAG" --pattern "inky-wasm.tar.gz" --dir "$TMPDIR"
    echo "  Extracting to bindings/node/..."
    tar -xzf "$TMPDIR/inky-wasm.tar.gz" -C bindings/node/
    rm -rf "$TMPDIR"
    echo "  Publishing..."
    cd bindings/node
    npm publish
    cd ../..
    PUBLISHED+=("npm")
else
    SKIPPED+=("npm")
fi
echo ""

# --- PyPI ---
echo "--- PyPI ---"
echo "  Package: inky-email@$VERSION"
echo "  Requires: python -m build, twine"
if confirm "  Publish to PyPI?"; then
    cd bindings/python
    rm -rf dist/
    python -m build
    twine upload dist/*
    cd ../..
    PUBLISHED+=("PyPI")
else
    SKIPPED+=("PyPI")
fi
echo ""

# --- RubyGems ---
echo "--- RubyGems ---"
echo "  Package: inky-email@$VERSION"
if confirm "  Publish to RubyGems?"; then
    cd bindings/ruby
    gem build inky-email.gemspec
    gem push inky-email-"${VERSION}".gem
    rm -f inky-email-"${VERSION}".gem
    cd ../..
    PUBLISHED+=("RubyGems")
else
    SKIPPED+=("RubyGems")
fi
echo ""

# --- Packagist ---
echo "--- Packagist ---"
echo "  Package: foundation/inky"
echo "  Packagist auto-publishes from GitHub tags."
echo "  Verify at: https://packagist.org/packages/foundation/inky"
echo ""

# --- Summary ---
echo "==> Done!"
if [ ${#PUBLISHED[@]} -gt 0 ]; then
    echo "  Published: ${PUBLISHED[*]}"
fi
if [ ${#SKIPPED[@]} -gt 0 ]; then
    echo "  Skipped:   ${SKIPPED[*]}"
fi
