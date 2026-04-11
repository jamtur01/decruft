#!/usr/bin/env bash
set -euo pipefail

# Usage: ./scripts/release.sh 0.2.0

if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

VERSION="$1"
TAG="v${VERSION}"

# Validate version format
if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$'; then
    echo "Error: version must be semver (e.g. 0.2.0)"
    exit 1
fi

# Check clean working tree
if [ -n "$(git status --porcelain)" ]; then
    echo "Error: working tree is dirty. Commit or stash first."
    exit 1
fi

# Check we're on main
BRANCH=$(git branch --show-current)
if [ "$BRANCH" != "main" ]; then
    echo "Error: must be on main (currently on $BRANCH)"
    exit 1
fi

# Check tag doesn't exist
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Error: tag $TAG already exists"
    exit 1
fi

# Check tests pass
echo "Running tests..."
cargo test --quiet
echo "Tests passed."

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
echo "Updated Cargo.toml to ${VERSION}"

# Verify it parses
cargo check --quiet

# Commit and tag
git add Cargo.toml Cargo.lock
git commit -m "release: v${VERSION}"
git tag -a "$TAG" -m "v${VERSION}"

echo ""
echo "Created commit and tag $TAG"
echo "Push with: git push origin main $TAG"
