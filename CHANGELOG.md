# Changelog

What changed for people using Halfwit, newest first. Code-only work stays in the git history. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow [SemVer](https://semver.org).

## [Unreleased]

## [0.4.0] - 2026-11-23

### Changed
- Halfwit has new plugin IDs and lists its maker as Daniel Marino. Hosts see this version as a new plugin, so projects saved with 0.3.0 or earlier need Halfwit added again.

## [0.3.0] - 2026-11-13

### Added
- Gate: Note, the highest note the bass plays under, so it can follow riffs on the low strings and drop out for leads up the neck. Off by default.

### Changed
- Drive sits in the top row with Level, Tone and Blend, as four equal knobs, and the Gate panel has more room.

## [0.2.0] - 2026-11-13

### Added
- A Dynamics knob on the gate, so the bass follows how hard you play and stray noises stay quiet.
- The version is printed on the pedal, so you can tell which build your host loaded.

### Fixed
- Low notes no longer jump up an octave when their second harmonic is louder than the fundamental.
- Pick scrapes and fret noise barely open the gate, which now listens to the same band-limited signal as the detector.

## [0.1.0] - 2026-11-12

### Added
- Halfwit, an octave-divider bass layer for guitar, as a VST3 and CLAP plugin for macOS and Windows.
- Level, Drive, Tone and Blend knobs, a Raw or Smooth feel, a gate, and a footswitch.
- A separate Bass output, so the bass can go to its own track and amp. By default the guitar leaves the main output and the bass leaves the Bass output; Mix and Bass put them on the main output instead.
