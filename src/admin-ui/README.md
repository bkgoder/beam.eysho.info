# Beam Admin UI

Dioxus web admin surface for bkg-beam.

Run with dx serve from this folder after installing the Dioxus CLI.

The UI is intentionally kept outside the root Cargo workspace for now because the workspace CI checks native Rust crates while Dioxus web builds target the browser.
