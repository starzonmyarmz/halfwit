---
name: screenshot
description: Retake the plugin's README screenshot (docs/<name>.jpg) after a visible change to the editor in ui/index.html. Use after any UI change you'd see in the window, or when asked for a new screenshot.
---

# Retake the screenshot

The README shows `docs/<name>.jpg`, where `<name>` is the crate name. The
Screenshot workflow retakes it on every push to main that touches the UI,
the version or `PAGE.size`, and commits it when it changed. Retake it
locally too when a branch changes the UI, so the PR shows the new look and
the `alt` text gets updated.

1. Check the mock's `specs` in `ui/index.html` match the plugin's
   parameters and defaults, so the shot shows the real controls at their
   defaults.
2. Run `scripts/screenshot.sh`. It renders the page against its mock in
   WebKit at 2x and `PAGE.size`, with the version from `Cargo.toml`,
   writes `docs/<name>.jpg` 852 px tall, and sets the README `<img>`
   `width` to half its pixel width.
3. Look at the JPEG. If fonts haven't loaded, a knob is mid-glide or the
   mock's values are off, fix that and rerun it.
4. Keep the README `<img>` `alt` describing what's on the panel,
   including any new control. The workflow can't update it.
5. Commit the image with the UI change it shows, not on its own.
