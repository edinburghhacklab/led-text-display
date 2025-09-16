# led-text-display

## structure

  - `crates/logic` holds the actual rendering logic. it uses `embedded-graphics` to draw things, with a wrapper that deals with swapping between screens.
  - `crates/runner` is the entrypoint of stuff that actually runs on the pi. it holds code that listens on the mqtt bus, and calls out to `logic`. it uses `rpi-led-panel` to draw the output to the led matrix.
  - `crates/simulator` is a development tool that shows you the rendered stuff in a window.

for more specific code structure stuff, run `cargo doc --open`. probably `crates/logic/src/screens/mod.rs` is a good place to start.

## development

if you just want to visualise things, you can do `cargo run --bin simulator` to run a local sim. see `crates/simulator/src/main.rs` for the information that this displays.

to deploy to the matrix, you'll need a rust toolchain that supports `armv7-unknown-linux-musleabihf`, and the associated linker stuff. `shell.nix` can help with this.

run `just run` to build, upload, and run it on the led matrix. when you're done, re-run `just upload` to make sure the correct version is on there, then on the windowpi do `sudo systemctl start led-matrix`.
