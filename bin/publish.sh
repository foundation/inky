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

# Pre-flight: check credentials for all registries
echo "==> Checking credentials..."
PREFLIGHT_OK=true

# crates.io: cargo checks ~/.cargo/credentials.toml or CARGO_REGISTRY_TOKEN
if cargo login --help > /dev/null 2>&1 && [ -f ~/.cargo/credentials.toml ] || [ -n "$CARGO_REGISTRY_TOKEN" ]; then
    echo "  crates.io: ok"
else
    echo "  crates.io: NOT CONFIGURED — run 'cargo login' or set CARGO_REGISTRY_TOKEN"
    PREFLIGHT_OK=false
fi

# npm: check for auth token
if npm whoami > /dev/null 2>&1; then
    echo "  npm: ok ($(npm whoami))"
else
    echo "  npm: NOT CONFIGURED — run 'npm login'"
    PREFLIGHT_OK=false
fi

# PyPI: check for twine and credentials (~/.pypirc or TWINE_USERNAME/TWINE_PASSWORD)
if ! command -v twine > /dev/null 2>&1; then
    echo "  PyPI: NOT CONFIGURED — twine not installed (brew install twine)"
    PREFLIGHT_OK=false
elif [ -f ~/.pypirc ] || [ -n "$TWINE_USERNAME" ] || [ -n "$TWINE_PASSWORD" ]; then
    echo "  PyPI: ok"
else
    echo "  PyPI: NOT CONFIGURED — create ~/.pypirc or set TWINE_USERNAME/TWINE_PASSWORD"
    PREFLIGHT_OK=false
fi

# RubyGems: check for credentials (~/.gem/credentials or GEM_HOST_API_KEY)
if [ -f ~/.gem/credentials ] || [ -n "$GEM_HOST_API_KEY" ]; then
    echo "  RubyGems: ok"
else
    echo "  RubyGems: NOT CONFIGURED — run 'gem signin' or set GEM_HOST_API_KEY"
    PREFLIGHT_OK=false
fi

echo ""
if [ "$PREFLIGHT_OK" = false ]; then
    echo "Some registries are not configured. Fix the issues above and re-run."
    exit 1
fi

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
    gh release download "$TAG" --pattern "inky-wasm-nodejs.tar.gz" --dir "$TMPDIR"
    echo "  Extracting to bindings/node/..."
    tar -xzf "$TMPDIR/inky-wasm-nodejs.tar.gz" -C bindings/node/
    rm -rf "$TMPDIR"
    echo "  Publishing..."
    cd bindings/node
    # Use --tag beta for prerelease versions, --tag latest for stable
    # --access public is required for first publish of scoped/new packages
    if echo "$VERSION" | grep -qE '(alpha|beta|rc|dev)'; then
        npm publish --access public --tag beta
    else
        npm publish --access public
    fi
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
    pyproject-build
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
    GEM_FILE=$(ls inky-email-*.gem | head -1)
    gem push "$GEM_FILE"
    rm -f "$GEM_FILE"
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
