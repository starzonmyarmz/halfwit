#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

crate=$(sed -n 's/^name = "\(.*\)"/\1/p' plugin/Cargo.toml)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)
size=$(sed -n 's/.*size: (\([0-9]*\), \([0-9]*\)).*/\1 \2/p' plugin/src/lib.rs)

if ! command -v pngquant >/dev/null; then
  echo "screenshot.sh needs pngquant: brew install pngquant" >&2
  exit 1
fi

pw=target/playwright
if [ ! -x "$pw/node_modules/.bin/playwright" ]; then
  npm install --prefix "$pw" --no-save --no-audit --no-fund playwright@1.64.1
fi
"$pw/node_modules/.bin/playwright" install webkit

mkdir -p docs
# shellcheck disable=SC2086
NODE_PATH="$pw/node_modules" node scripts/screenshot.cjs "docs/$crate.png" $size "$version"
sips --resampleHeight 852 "docs/$crate.png" >/dev/null
pngquant --quality 70-95 --speed 1 --force --skip-if-larger --output "docs/$crate.png" "docs/$crate.png" || [ $? -eq 98 ]
rm -f "docs/$crate.jpg"

half=$(( $(sips -g pixelWidth "docs/$crate.png" | awk '/pixelWidth/ { print $2 }') / 2 ))
sed -i '' -E "s#<img src=\"docs/$crate\\.(jpg|png)\" width=\"[0-9]+#<img src=\"docs/$crate.png\" width=\"$half#" README.md
echo "Wrote docs/$crate.png ($half px wide in the README)"
