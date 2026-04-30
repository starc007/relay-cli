#!/usr/bin/env sh
set -e

# Usage: ./release.sh [version]
# Example: ./release.sh 0.2.0
# If no version given, prompts for it.

CURRENT=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ -n "$1" ]; then
  VERSION="$1"
else
  printf "current version: %s\nnew version: " "$CURRENT"
  read -r VERSION
fi

if [ -z "$VERSION" ]; then
  echo "version required"
  exit 1
fi

# Strip leading v if provided
VERSION="${VERSION#v}"

echo "releasing v$VERSION..."

# Ensure clean working tree
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "error: uncommitted changes. commit or stash first."
  exit 1
fi

# Update version in Cargo.toml
sed -i '' "s/^version = \"$CURRENT\"/version = \"$VERSION\"/" Cargo.toml

# Update Cargo.lock
cargo update -p relay-cli --precise "$VERSION" 2>/dev/null || cargo build -q 2>/dev/null || true

# Commit version bump
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to v$VERSION"

# Tag and push
git tag "v$VERSION"
git push origin main
git push origin "v$VERSION"

echo "released v$VERSION — GitHub Actions is building binaries now."
echo "track progress: https://github.com/starc007/relay-cli/actions"
