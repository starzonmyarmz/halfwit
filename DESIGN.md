# Design

How Halfwit looks and feels. When this file and your instincts disagree, this file wins. Change it on purpose, in the same commit as the code.

## The look in one line

A mustard powder-coated stompbox with black knobs and a chrome footswitch: the octave pedal a two-piece band leans on.

## Principles

Every plugin gets its own look, but they share an approach.

- **Skeuomorphic.** The editor is a physical object (Patina an OP-1-style synth, Halfwit a stompbox, Ghosted a vintage radio), with real materials like powder coat, bakelite, walnut, brass and grille cloth, built in CSS gradients and shadows rather than images.
- **One theme.** Pick the object from the plugin's vibe and name, and let it run through everything: the wordmark, the art and the control labels. Names should be playful and capture the vibe.
- **Restrained.** One knob style and colour family per plugin, and few controls. Fold many settings into a handful of knobs with evocative names rather than exposing every one.
- **Colour and art mean something.** If two waves or knobs differ in colour, it's because they differ in function; otherwise make them the same. Art can follow parameters (Ghosted's ghosts fade with Haunt, the dial needle follows Distance), but don't fake audio metering.
- **Plain labels.** Words a guitarist understands, not DSP terms or fieldset jargon. Numbers go in a tooltip readout.
- **Type with character.** Embed a display face for the wordmark (Futura-like Jost, pillowy Modak), never a system or "commercial" font, and centre it optically on its badge.
- **Drawn, not vector-perfect.** Vary repeated elements (size, tilt, expression), and give outlines weight that swells on the shadow side, like inked comic art.
- **A footswitch on effects.** Pedals and other effects get an on/off toggle (`stomp`, on by default). Off fades the effect out and passes the dry signal; a lamp or LED shows the state, and the art reacts (Ghosted's dial dims and its ghosts leave).
- **The version on the panel.** Print it small and quiet as part of the object, like a model number on a pedal's base plate or a serial plate on a radio's back (`v1.2.0`), so anyone can tell which build they're running. Local installs show `X.Y.Z-dev+<sha>`.
- **Physical details behave physically.** Shadows stay put while a knob turns, pressed keys sink, lit things glow.

## This plugin

Settled with the user; keep to it.

- **Object:** a stompbox in mustard powder coat, with a screw in each corner, jacks along the top edge (`◂ OUT`, `◂ BASS`, `IN ◂`), and a metal footswitch with a red LED.
- **Palette:** see Tokens. Red belongs to the LED only.
- **Wordmark font:** Jost Black, embedded from `ui/fonts`. HALFWIT is cut in half horizontally and the lower half slides right, under a sine wave and the square wave it divides into, with the tagline OCTAVE DIVIDER · BASS SPLITTER.
- **Controls:** a top row of four big knobs (Level, Drive, Tone, Blend); then a Raw/Smooth Feel toggle and a three-position Output selector (Split, Mix, Bass), next to a Gate panel of four small trims (Thresh, Rel, Dyn, Note). The Output selector's Split and Bass labels overflow its box, so leave room for them.
- **Version placement:** it ends the fine print by the bottom edge, `÷2 · TWO-PIECE APPROVED · V0.1.0`.

## Tokens

Every color, size and space comes from a token. If nothing fits, add a token here before using a new value. They live as custom properties at the top of `ui/index.html`.

| Token | Value | Used for |
|---|---|---|
| `--board` | `#181716` | the page behind the pedal |
| `--coat`, `--coat-hi`, `--coat-lo` | `#d8a72a`, `#f0c85a`, `#9e7413` | the mustard powder coat and its light and shade |
| `--ink`, `--ink-soft` | `#1b1611`, 55% `--ink` | printed labels and art |
| `--cream` | `#f4ead2` | knob pointers, the readout text and the focus ring |
| `--chrome-hi`, `--chrome`, `--chrome-lo` | `#fbfbf9`, `#c4c6c8`, `#6d7074` | screws, jacks and the footswitch |
| `--knob`, `--knob-hi` | `#151413`, `#34312e` | knobs |
| `--led`, `--led-off` | `#ff3b2f`, `#5a1712` | the footswitch LED only |
| `--pill` | `#120f0c` | the value readout under a knob |
| `--display`, `--label` | Jost | the wordmark and labels |

## Components

## Patterns

## Adding something new

1. Look for an existing component or pattern that already does it.
2. Build it from tokens only.
3. Check Reduce Motion if it moves.
4. Retake the screenshot if it shows in one.
5. Record it above.

## Known drift

Existing debt to clean up, not to copy.
