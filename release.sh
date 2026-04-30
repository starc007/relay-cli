#!/usr/bin/env sh
set -e

# Usage: ./release.sh [version]
# Example: ./release.sh 0.2.0
# If no version given, prompts for it.

CURRENT=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ -n "$1" ]; then
  VERSION="$1"
else
  printf "current version: %s\nnew version (enter to keep): " "$CURRENT"
  read -r VERSION
fi

# Default to current if empty
VERSION="${VERSION:-$CURRENT}"
VERSION="${VERSION#v}"

echo "releasing v$VERSION..."

# Ensure clean working tree
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "error: uncommitted changes. commit or stash first."
  exit 1
fi

# Bump Cargo.toml only if version changed
if [ "$VERSION" != "$CURRENT" ]; then
  sed -i '' "s/^version = \"$CURRENT\"/version = \"$VERSION\"/" Cargo.toml
  cargo build -q 2>/dev/null || true
  git add Cargo.toml Cargo.lock
  git commit -m "chore: bump version to v$VERSION"
fi

# Check tag doesn't already exist
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
  echo "error: tag v$VERSION already exists. choose a different version."
  exit 1
fi

git tag "v$VERSION"
git push origin main
git push origin "v$VERSION"

echo "released v$VERSION — GitHub Actions is building binaries now."
echo "track progress: https://github.com/starc007/relay-cli/actions"
