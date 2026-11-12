# Halfwit

<img src="docs/halfwit.jpg" width="293" alt="Halfwit, a mustard-yellow stompbox with level, blend, tone and drive knobs, a feel toggle, an output selector, a gate panel, and a footswitch">

An octave-divider bass splitter for guitar, as a VST3 and CLAP plugin
for macOS and Windows. It's for two-piece bands with no bass player: it
builds a bass part from the guitar signal, one octave down, and sends it
out a separate Bass output so it can go to its own track, with its own
amp, while the guitar carries on untouched.

It's a digital take on the old flip-flop divider pedals, not a pitch
shifter. Single notes and riffs track tightly; chords turn to mush, and
that's part of the sound.

## How it works

1. A band-limited copy of the input feeds a zero-crossing detector with
   hysteresis, so harmonics and string noise don't trigger it.
2. Every rising crossing flips a square wave, which halves the
   frequency. In Smooth feel, the square drives a rounded wave instead.
3. A gate silences the layer between notes, then drive, a two-pole
   low-pass (Tone) and a DC blocker shape it.
4. The Output selector decides where it goes: by default the guitar
   leaves the main output and the bass leaves the Bass aux output.

The square's edges are anti-aliased with PolyBLEP. That correction
needs one sample of lookahead, so Halfwit reports one sample of
latency, which hosts compensate for.

## Controls

| Control | Range | What it does |
|---|---|---|
| Level | 0–100% | Bass layer level going into the drive |
| Blend | 0–100% | Bass layer output level |
| Tone | 40 Hz–2 kHz | Low-pass cutoff on the bass layer |
| Drive | 0–200% | Saturates the bass layer and hides divider glitches |
| Feel | Raw / Smooth | Raw is the hard square; Smooth is rounder, closer to a sine |
| Output | Split / Mix / Bass | Split: guitar on the main output, bass on the Bass output. Mix: both on the main output, like an octave pedal. Bass: only the bass, on the main output |
| Gate: Thresh | −80–0 dB | Input level below which the bass layer shuts off |
| Gate: Rel | 1–500 ms | How long the gate stays open after the input drops |
| Footswitch | On / Off | Turns the bass layer on and off; the guitar still passes |

Drag a knob up or down (hold Shift for fine moves), scroll, or use the
arrow keys. Double-click a knob to reset it. Click the Output selector
to step through its positions.

## Routing the bass to its own track

The Bass output is a second stereo bus. In REAPER:

1. Put Halfwit first on the guitar track, and set the track to 4
   channels (the routing button → Track channels). REAPER sends the Bass
   output to channels 3–4, and plugins after Halfwit, like an amp sim,
   only touch 1–2.
2. Make a Bass track, and add a send to it from the guitar track:
   audio from channels 3–4 into 1–2, post-FX.
3. Put a bass amp on the Bass track.

In hosts without multi-output support, use Mix, or Bass on a duplicate
of the guitar track.

## Tips

- Put Halfwit before your amp sim, like a pedal.
- The detector tracks best on the neck pickup with clean, single notes.
  Palm mutes and power chords work; full chords won't.
- Set Gate: Thresh just above your noise floor, especially with
  single-coil pickups, or hum will keep the divider chattering.

## Installing

Download the zip from
[Releases](https://github.com/starzonmyarmz/halfwit/releases) and follow
`INSTALL.txt` inside it.

## Building

```sh
cargo xtask bundle halfwit --release       # target/bundled/Halfwit.{vst3,clap}
scripts/install.sh                         # bundle, sign ad hoc and install on macOS
scripts/clap-validator.sh target/bundled/Halfwit.clap
```

## Tests

`cargo test` runs the engine offline:

- A 3-second guitar DI (`plugin/tests/di/riff.wav`) is rendered through
  both feels and compared against stored renders in
  `plugin/tests/golden`. After an intended change to the sound, rerun
  with `HALFWIT_BLESS=1` to rewrite them, then listen to the new files
  before committing.
- Synthetic tests check octave tracking from low E to high E, the gate
  closing after a note, footswitch-off passing the guitar through at
  the reported latency, Split and Mix routing, and finite output from
  22.05 to 192 kHz.

## Working on the UI

The editor is a webview showing `ui/index.html`, which is compiled into
the plugin. Without the plugin, the page runs against a mock, so you can
work on it in a browser:

```sh
cd ui && python3 -m http.server 8765    # open http://127.0.0.1:8765
```

To load the page live in the plugin, set `HALFWIT_UI_URL` to that URL
before starting the host.

`plugin/src/editor.rs` and `plugin/src/bridge.rs` contain nothing
specific to Halfwit: the page's size and HTML come from `PAGE` in
`lib.rs`, and the bridge works with any `#[derive(Params)]` struct.

## CI and releases

- `check.yml` runs on pushes to main and on PRs: the tests and
  clap-validator on macOS, and clap-validator on the Windows build.
- `release.yml` runs on `vX.Y.Z` tags. The tag must match `version` in
  the root `Cargo.toml`. It builds a universal macOS bundle and a
  Windows bundle, validates both, and publishes `Halfwit-X.Y.Z.zip`
  with `INSTALL.txt`.

To release: bump `version` in `Cargo.toml`, commit, then
`git tag vX.Y.Z && git push origin main vX.Y.Z`.

## Built with

- [nice-plug](https://codeberg.org/RustAudio/nice-plug), pinned to
  [a fork](https://github.com/starzonmyarmz/nice-plug/tree/patina) that
  adds `ActivateContext::gui_context()`. To upgrade, rebase the fork's
  `patina` branch, push it, and update the `rev` in `plugin/Cargo.toml`
  and `xtask/Cargo.toml`.
- [Jost](https://github.com/indestructible-type/Jost) by Owen Earl,
  under the SIL Open Font License (`ui/fonts/OFL.txt`).
- Started from
  [plugin-template](https://github.com/starzonmyarmz/plugin-template),
  with the editor from
  [Patina](https://github.com/starzonmyarmz/patina).
