#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

old=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)
release=${old%%-*}
major=${release%%.*}
rest=${release#*.}
minor=${rest%%.*}
patch=${rest#*.}

case "${1:-patch}" in
  patch) new="$major.$minor.$((patch + 1))" ;;
  minor) new="$major.$((minor + 1)).0" ;;
  major) new="$((major + 1)).0.0" ;;
  *) echo "usage: scripts/bump.sh [patch|minor|major]" >&2; exit 1 ;;
esac

sed -i '' "s/^version = \"$old\"/version = \"$new\"/" Cargo.toml
cargo update --workspace --quiet
echo "$old -> $new"
