# Plugin template

A starting point for VST3 + CLAP plugins in Rust for macOS and Windows,
built with [nice-plug](https://codeberg.org/RustAudio/nice-plug), with the
build and release setup from [Patina](https://github.com/starzonmyarmz/patina).
The plugin itself is a gain knob, there to prove the setup works.

It pins the same nice-plug fork as Patina
([starzonmyarmz/nice-plug](https://github.com/starzonmyarmz/nice-plug/tree/patina)),
which adds `ActivateContext::gui_context()` so MIDI controllers can set
parameters before the editor opens.

## Starting a plugin

Click **Use this template** on GitHub, then rename `Example`:

| What | Where |
|---|---|
| Crate name (`example`) | `plugin/Cargo.toml`, `bundler.toml`, `CRATE` in both workflows, `crate` in `scripts/install.sh` |
| Display name (`Example`) | `bundler.toml`, `NAME` in `plugin/src/lib.rs`, `PLUGIN` in both workflows, `INSTALL.txt` |
| Plugin IDs | `CLAP_ID` and `VST3_CLASS_ID` (exactly 16 bytes) in `plugin/src/lib.rs`. These must be unique, and must never change after a release. |
| Description, features, subcategories | `ClapPlugin` and `Vst3Plugin` in `plugin/src/lib.rs` |

## Building

```sh
cargo xtask bundle example --release       # target/bundled/Example.{vst3,clap}
scripts/install.sh                         # bundle, sign ad hoc and install on macOS
scripts/clap-validator.sh target/bundled/Example.clap
```

## CI and releases

- `check.yml` runs on pushes to main and on PRs: the tests and clap-validator
  on macOS, and clap-validator on the Windows build.
- `release.yml` runs on `vX.Y.Z` tags. The tag must match `version` in the
  root `Cargo.toml`. It builds a universal macOS bundle and a Windows bundle,
  validates both, and publishes `Example-X.Y.Z.zip` with `INSTALL.txt`. A
  version with a suffix (`1.0.0-beta.1`) becomes a prerelease.

To release: bump `version` in `Cargo.toml`, commit, then
`git tag vX.Y.Z && git push origin main vX.Y.Z`.

## Upgrading nice-plug

Rebase the fork's `patina` branch onto newer nice-plug, push it, and update
the `rev` in `plugin/Cargo.toml` and `xtask/Cargo.toml`.

## Adding a webview editor or a standalone app

Copy them from Patina (`plugin/src/editor.rs`, `plugin/src/chrome.rs`,
`app/`). They aren't here yet because they're built around Patina's
parameters, and the reusable shape should come from the second plugin that
needs them. Bring along the `[profile.dev.package.*]` overrides in Patina's
root `Cargo.toml`: debug builds otherwise abort when old objc-based crates
message nil.
