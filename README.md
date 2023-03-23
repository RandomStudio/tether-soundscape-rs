# Tether Soundscape rs

A multi-layered audio sequencer, remote-controllable via Tether, to create soundscapes.

Using 🦀 Rust because:
- Minimal memory/CPU footprint for high performance
- Cross-platform but without any need to install browser, use Electron, etc.
- Visualisation via Nannou
- Great way to learn about low-level audio sample/buffer control, multi-threading in Rust (Nannou always uses separate "realtime" thread for audio)

TODO:
- [ ] Demonstrate running on Raspberry Pi
- [x] Apply "loop" as well as trigger/hit/once-off functions
- [x] Allow clips to be stopped/removed while playing (without stopping whole stream)
- [ ] Allow starting/fixed "maximum" volume per clip to be applied
- [ ] Apply fade in/out volume controls
- [ ] Make use of tempo, quantisation for timing
- [ ] Optionally connect to [Ableton link](https://docs.rs/ableton-link/latest/ableton_link/)
- [ ] On-screen UI (Egui), CLI params, etc.
- [ ] Low/no graphics mode
- [ ] Add Tether remote control commands, as per API in [original](https://github.com/RandomStudio/tether-soundscape)

## Compiling for Raspberry Pi
The only(?) part of the application which has a system dependency is ALSA.
`sudo apt install libasound2-dev`

This does mean that cross-compiling from MacOS is currently stuck at building the [alsa-sys](https://crates.io/crates/alsa-sys) crate. This might be solvable, e.g. with https://github.com/cross-rs/cross
