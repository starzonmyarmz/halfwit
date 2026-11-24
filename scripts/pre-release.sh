#!/bin/sh
# scripts/release runs this with --check before changing anything, then with the new version
# after writing it, so the release commit carries a screenshot showing that version.
set -eu
cd "$(dirname "$0")/.."

if [ "${1:-}" = --check ]; then
  for tool in pngquant magick node; do
    if ! command -v $tool >/dev/null; then
      echo "pre-release: $tool is missing (brew install pngquant imagemagick node)" >&2
      exit 1
    fi
  done
  exit 0
fi

scripts/screenshot.sh
