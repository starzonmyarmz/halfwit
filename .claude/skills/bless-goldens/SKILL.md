---
name: bless-goldens
description: Add or rewrite golden WAV renders in plugin/tests/golden after an intended change to the sound, and get the user to listen before committing. Use when a golden test fails because of an intended DSP change, or when adding a new golden render.
---

# Bless golden renders

A failing golden test means the sound changed. Rewrite the files only when
the change was intended, and never commit them before the user has
listened.

1. Run `cargo test` and note which golden tests fail. If a failure
   wasn't expected from the change, stop and find out why instead of
   blessing it.
2. Rewrite them: `BLESS=1 cargo test`, then `cargo test` again to
   check they pass.
3. List each changed file in `plugin/tests/golden`, say what should
   sound different and why, and offer to play them with
   `afplay plugin/tests/golden/<name>.wav`. For a subtle change, also
   play the old version from git so the user can compare:
   `git show HEAD:plugin/tests/golden/<name>.wav > <scratch>/<name>-old.wav`.
4. Wait for the user to say they've listened and it sounds right. If
   it doesn't, restore the files with `git checkout plugin/tests/golden`
   and go back to the DSP.
5. Commit the golden files with the DSP change that caused them, and
   say in the commit message what changed in the sound.
