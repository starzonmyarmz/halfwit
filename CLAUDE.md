# Halfwit

Read `README.md` first for what the plugin does, its controls, and how to
build and run it. This file has the maintainer details it leaves out.

## Layout

- `plugin/src/engine.rs`: all DSP, with no plugin framework types, so tests
  drive it directly.
- `plugin/src/params.rs`: host parameters, mapped to engine `Settings`.
- `plugin/src/editor.rs` and `plugin/src/bridge.rs`: nothing specific to
  Halfwit. The page's size and HTML come from `PAGE` in `lib.rs`, and the
  bridge works with any `#[derive(Params)]` struct.
- `ui/index.html`: the whole editor, compiled into the plugin, with a mock
  bridge for running it in a browser.

## Adding a parameter

It has to go in five places:

1. `Settings` in `engine.rs`, including its `Default`.
2. `HalfwitParams` in `params.rs`, and its `settings()` mapping.
3. The mock's `specs` in `ui/index.html`. The plugin's bridge picks new
   parameters up automatically, but the browser mock doesn't.
4. A `data-knob` (or `data-toggle`) element in the markup.
5. The controls table in `README.md`.

## Sound changes and golden renders

`plugin/tests/golden` holds renders of `plugin/tests/di/riff.wav` through
both feels. After an intended change to the sound, rerun with
`HALFWIT_BLESS=1 cargo test` to rewrite them. Never commit rewritten
renders until the user has listened to them, and say in the commit what
changed in the sound (levels, which notes or noises are affected).

## UI

- The window size is set in two places that must change together:
  `PAGE.size` in `lib.rs` and `.pedal`'s `inline-size` in `ui/index.html`.
  Leave room for the Output selector's Split and Bass labels, which
  overflow its box.
- The version, from the root `Cargo.toml`, ends the fine print by the
  bottom edge (`÷2 · TWO-PIECE APPROVED · V0.1.0`).
- The local preview server (`python3 -m http.server`) gets cached by the
  browser. Add a query string like `?v=2` after each edit.
- After a visible UI change, retake `docs/halfwit.jpg`: capture the full
  window at device scale 2, save it as a JPEG 852 px tall, and set the
  README `<img>` width to half the JPEG's width.

## Releases

`release.yml` runs on `vX.Y.Z` tags, which must match `version` in the
root `Cargo.toml`. To release: bump `version`, commit, then
`git tag vX.Y.Z && git push origin main vX.Y.Z`.

## nice-plug fork

The plugin is pinned to the `patina` branch of a nice-plug fork, which
adds `ActivateContext::gui_context()`. To upgrade, rebase that branch onto
upstream, push it, and update the `rev` in both `plugin/Cargo.toml` and
`xtask/Cargo.toml`.
