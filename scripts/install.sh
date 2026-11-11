#!/bin/sh
set -eu
cd "$(dirname "$0")/.."

crate=halfwit
name=$(sed -n 's/^name = "\(.*\)"/\1/p' bundler.toml)

cargo xtask bundle "$crate" --release

dest="$HOME/Library/Audio/Plug-Ins"
mkdir -p "$dest/VST3" "$dest/CLAP"
for kind in vst3:VST3 clap:CLAP; do
  ext=${kind%%:*}
  dir=${kind##*:}
  rm -rf "$dest/$dir/$name.$ext"
  cp -R "target/bundled/$name.$ext" "$dest/$dir/"
  codesign --force --deep --sign - "$dest/$dir/$name.$ext"
  echo "Installed $dest/$dir/$name.$ext"
done
