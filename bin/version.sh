#!/bin/bash
# Bump version across all crates and bindings.
# Usage: bin/version.sh <version>
# Example: bin/version.sh 2.0.0-beta.2

set -e

cd "$(dirname "$0")/.."

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: bin/version.sh <version>"
    echo "Example: bin/version.sh 2.0.0-beta.2"
    exit 1
fi

# Detect current version from inky-core
CURRENT=$(grep '^version' crates/inky-core/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ "$CURRENT" = "$VERSION" ]; then
    echo "Version is already $VERSION"
    exit 0
fi

echo "==> Bumping version: $CURRENT → $VERSION"

# Rust crates — package version (line 3 in each Cargo.toml)
for toml in crates/inky-core/Cargo.toml crates/inky-cli/Cargo.toml crates/inky-ffi/Cargo.toml crates/inky-wasm/Cargo.toml; do
    sed -i '' "s/^version = \"$CURRENT\"/version = \"$VERSION\"/" "$toml"
    echo "  updated $toml (package version)"
done

# Rust crates — inky-core dependency version
for toml in crates/inky-cli/Cargo.toml crates/inky-ffi/Cargo.toml crates/inky-wasm/Cargo.toml; do
    sed -i '' "s/inky-core = { version = \"$CURRENT\"/inky-core = { version = \"$VERSION\"/" "$toml"
    echo "  updated $toml (inky-core dependency)"
done

# Node.js (WASM package)
sed -i '' "s/\"version\": \"$CURRENT\"/\"version\": \"$VERSION\"/" bindings/node/package.json
echo "  updated bindings/node/package.json"

# Python
# Python may have a different version if it wasn't synced — use a broader match
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" bindings/python/pyproject.toml
echo "  updated bindings/python/pyproject.toml"

# Ruby
sed -i '' "s/s\.version     = \".*\"/s.version     = \"$VERSION\"/" bindings/ruby/inky-email.gemspec
echo "  updated bindings/ruby/inky-email.gemspec"

echo ""
echo "==> Verifying workspace..."
cargo check --workspace 2>&1 | tail -1

echo ""
echo "==> Version bumped to $VERSION"
echo "    Review changes with: git diff"
echo "    Then commit: git add -A && git commit -m 'Bump version to $VERSION'"
