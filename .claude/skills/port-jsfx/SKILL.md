---
name: port-jsfx
description: Port a REAPER JSFX effect into this plugin's Rust engine, cross-checked sample for sample and locked in with golden renders. Use when the user asks to port, convert or base the plugin on a JSFX.
---

# Port a JSFX

JSFX sources are in REAPER's `Effects` folder, or `~/Code/jsfx`.

The JSFX is only a prototype. Never mention it in this repo: not in the
README, CLAUDE.md, code, comments, test names, parameter tables or commit
messages. Describe the plugin on its own terms.

1. **Port the engine** to `plugin/src/engine.rs` in f64, with the
   JSFX's sliders one for one in a `Settings` struct. Keep `@init`,
   `@slider`, `@block` and `@sample` order and state as they are, even
   where the code could be tidier.
2. **Cross-check it.** In a scratch directory outside the repo, write a
   line-by-line Python transliteration of the JSFX and a small binary or
   test that renders the Rust engine with the same `Settings`. Run both
   over a real DI from `plugin/tests/di` and compare to within about
   1e-8. If the JSFX calls `rand()`, use the same seeded RNG in both
   (Ghosted uses xorshift64*) and draw in the same order. Fix the port
   until they match, and tell the user the final max difference.

   Have a subagent write the Python transliteration from the JSFX
   alone, in parallel with step 1, without showing it the Rust port.
   That's faster, and two independent readings catch misreadings of the
   JSFX that one reader would make the same way in both.
3. **Lock it in** with golden renders in `plugin/tests/engine.rs`, using
   the `bless-goldens` skill.
4. **Map the front panel.** If the panel has fewer knobs than the JSFX
   has sliders, add a `Controls` struct whose `settings()` maps knobs
   to `Settings` (see Ghosted), in a separate step and commit, so the
   engine itself can still be compared with the JSFX.
5. **Add what the JSFX lacks**, each as its own commit:
   - glides on parameters that click when moved
   - slew limits on modulated delay times (an unlimited glide can read
     a delay line backwards)
   - subnormal flushing of state and output below about 1e-30
   - corner frequencies clamped below Nyquist at low sample rates
6. Run `cargo test` and `scripts/clap-validator.sh` after each step.
