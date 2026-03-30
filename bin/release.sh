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
gh run watch "$RUN_ID" --exit-status
echo "==> Release workflow completed."

# Download CLI tarballs and compute SHA256s
echo "==> Computing SHA256 checksums..."
TMPDIR=$(mktemp -d)
TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin" "aarch64-unknown-linux-gnu" "x86_64-unknown-linux-gnu")

for target in "${TARGETS[@]}"; do
    gh release download "$TAG" --pattern "inky-${target}.tar.gz" --dir "$TMPDIR"
done

declare -A SHAS
for target in "${TARGETS[@]}"; do
    SHAS[$target]=$(shasum -a 256 "$TMPDIR/inky-${target}.tar.gz" | awk '{print $1}')
    echo "  $target: ${SHAS[$target]}"
done

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
      sha256 "${SHAS[aarch64-apple-darwin]}"
    else
      url "https://github.com/foundation/inky/releases/download/${TAG}/inky-x86_64-apple-darwin.tar.gz"
      sha256 "${SHAS[x86_64-apple-darwin]}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/foundation/inky/releases/download/${TAG}/inky-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "${SHAS[aarch64-unknown-linux-gnu]}"
    else
      url "https://github.com/foundation/inky/releases/download/${TAG}/inky-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "${SHAS[x86_64-unknown-linux-gnu]}"
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

# Publish to package registries
echo ""
read -r -p "==> Publish to package registries now? [y/N] " response
if [[ "$response" =~ ^[Yy]$ ]]; then
    bin/publish.sh
fi

echo "==> Done! Inky $TAG has been released."
