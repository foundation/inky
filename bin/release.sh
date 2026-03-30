#!/bin/bash
set -e

# Usage: ./bin/release.sh <version>
# Example: ./bin/release.sh 2.0.0

VERSION=$1

if [ -z "$VERSION" ]; then
    echo "Usage: ./bin/release.sh <version>"
    echo "Example: ./bin/release.sh 2.0.0"
    exit 1
fi

TAG="v${VERSION}"

# Ensure we're on develop and it's clean
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "develop" ]; then
    echo "Error: Must be on the develop branch. Currently on: $BRANCH"
    exit 1
fi

# Check tag doesn't already exist
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Error: Tag $TAG already exists."
    exit 1
fi

# Verify CHANGELOG.md has an entry for this version
if ! grep -q "## $VERSION" CHANGELOG.md 2>/dev/null; then
    echo "Error: No entry for $VERSION found in CHANGELOG.md."
    echo "Add a changelog entry before releasing."
    exit 1
fi

echo "==> Releasing Inky $TAG"

# Bump versions if needed
CORE_VERSION=$(grep '^version' crates/inky-core/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
if [ "$CORE_VERSION" != "$VERSION" ]; then
    echo "==> Bumping versions..."
    bin/version.sh "$VERSION"
    git add -A
    git commit -m "Bump version to $VERSION"
else
    # If versions already match, working directory must be clean
    if [ -n "$(git status --porcelain)" ]; then
        echo "Error: Working directory is not clean. Commit or stash your changes first."
        exit 1
    fi
fi

# Tag the release
echo "==> Tagging $TAG..."
git tag -a "$TAG" -m "$TAG"

# Push the tag (triggers the release workflow)
echo "==> Pushing tag to remote..."
git push origin develop --tags

# Wait for the GitHub Actions release workflow to complete
echo "==> Waiting for release workflow to complete..."
sleep 5
RUN_ID=$(gh run list --workflow=release.yml --branch="$TAG" --limit=1 --json databaseId --jq '.[0].databaseId')
if [ -z "$RUN_ID" ]; then
    echo "Error: Could not find release workflow run for $TAG"
    exit 1
fi
if gh run watch "$RUN_ID" --exit-status; then
    echo "==> Release workflow completed."
else
    echo "==> Release workflow had failures (this is expected if crates.io was already published)."
    echo "    Verify the GitHub release assets exist before continuing."
    read -r -p "    Continue? [y/N] " response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Download CLI tarballs and compute SHA256s
echo "==> Computing SHA256 checksums..."
TMPDIR=$(mktemp -d)
TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin" "aarch64-unknown-linux-gnu" "x86_64-unknown-linux-gnu")

for target in "${TARGETS[@]}"; do
    gh release download "$TAG" --pattern "inky-${target}.tar.gz" --dir "$TMPDIR"
done

SHA_AARCH64_APPLE=$(shasum -a 256 "$TMPDIR/inky-aarch64-apple-darwin.tar.gz" | awk '{print $1}')
SHA_X86_64_APPLE=$(shasum -a 256 "$TMPDIR/inky-x86_64-apple-darwin.tar.gz" | awk '{print $1}')
SHA_AARCH64_LINUX=$(shasum -a 256 "$TMPDIR/inky-aarch64-unknown-linux-gnu.tar.gz" | awk '{print $1}')
SHA_X86_64_LINUX=$(shasum -a 256 "$TMPDIR/inky-x86_64-unknown-linux-gnu.tar.gz" | awk '{print $1}')

echo "  aarch64-apple-darwin: $SHA_AARCH64_APPLE"
echo "  x86_64-apple-darwin:  $SHA_X86_64_APPLE"
echo "  aarch64-unknown-linux-gnu: $SHA_AARCH64_LINUX"
echo "  x86_64-unknown-linux-gnu:  $SHA_X86_64_LINUX"

# Update Homebrew tap
TAP_DIR="$(cd "$(dirname "$0")"/../../homebrew-inky && pwd)"
if [ -d "$TAP_DIR" ]; then
    echo "==> Updating Homebrew formula..."
    FORMULA="$TAP_DIR/Formula/inky.rb"

    # Start from the template in homebrew dir and fill in values
    cat > "$FORMULA" << RUBY
class Inky < Formula
  desc "Transform email templates into email-safe HTML"
  homepage "https://github.com/foundation/inky"
  version "${VERSION}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/foundation/inky/releases/download/${TAG}/inky-aarch64-apple-darwin.tar.gz"
      sha256 "${SHA_AARCH64_APPLE}"
    else
      url "https://github.com/foundation/inky/releases/download/${TAG}/inky-x86_64-apple-darwin.tar.gz"
      sha256 "${SHA_X86_64_APPLE}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/foundation/inky/releases/download/${TAG}/inky-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "${SHA_AARCH64_LINUX}"
    else
      url "https://github.com/foundation/inky/releases/download/${TAG}/inky-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "${SHA_X86_64_LINUX}"
    end
  end

  def install
    bin.install "inky"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/inky --version")
  end
end
RUBY

    cd "$TAP_DIR"
    git add Formula/inky.rb
    git commit -m "Update inky to ${TAG}"
    git push origin main
    cd - > /dev/null
    echo "==> Homebrew formula updated."
else
    echo "Warning: Homebrew tap not found at $TAP_DIR — skipping formula update."
fi

# Clean up
rm -rf "$TMPDIR"

# Tag inky-go module
GO_DIR="$(cd "$(dirname "$0")"/../../inky-go 2>/dev/null && pwd)"
if [ -d "$GO_DIR" ]; then
    echo "==> Tagging inky-go $TAG..."
    cd "$GO_DIR"
    git tag -a "$TAG" -m "$TAG"
    git push origin --tags
    cd - > /dev/null
    echo "==> inky-go $TAG tagged and pushed."
    echo "==> Triggering pkg.go.dev indexing..."
    curl -s "https://proxy.golang.org/github.com/foundation/inky-go/@v/${TAG}.info" > /dev/null 2>&1 || true
else
    echo "Warning: inky-go repo not found at ../inky-go — skipping Go module tag."
fi

# Publishing to crates.io, npm, PyPI, and RubyGems is handled by the
# release workflow in CI. If any publish step fails, use bin/publish.sh
# as a manual fallback.

echo "==> Done! Inky $TAG has been released."
echo "    Publishing to registries will happen automatically via CI."
echo "    If needed, run bin/publish.sh as a fallback."
