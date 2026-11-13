#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

crate=$(sed -n 's/^name = "\(.*\)"/\1/p' plugin/Cargo.toml)
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)
size=$(sed -n 's/.*size: (\([0-9]*\), \([0-9]*\)).*/\1 \2/p' plugin/src/lib.rs)

pw=target/playwright
if [ ! -x "$pw/node_modules/.bin/playwright" ]; then
  npm install --prefix "$pw" --no-save --no-audit --no-fund playwright@1.64.1
fi
"$pw/node_modules/.bin/playwright" install webkit

mkdir -p docs
# shellcheck disable=SC2086
NODE_PATH="$pw/node_modules" node scripts/screenshot.cjs "docs/$crate.png" $size "$version"
sips -s format jpeg -s formatOptions 85 --resampleHeight 852 "docs/$crate.png" --out "docs/$crate.jpg" >/dev/null
rm "docs/$crate.png"

half=$(( $(sips -g pixelWidth "docs/$crate.jpg" | awk '/pixelWidth/ { print $2 }') / 2 ))
sed -i '' -E "s|(<img src=\"docs/$crate\\.jpg\" width=\")[0-9]+|\\1$half|" README.md
echo "Wrote docs/$crate.jpg ($half px wide in the README)"
