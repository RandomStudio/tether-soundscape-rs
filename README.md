# Tether Soundscape rs

A multi-layered audio sequencer, remote-controllable via Tether, to create soundscapes.

Using 🦀 Rust because:
- Minimal memory/CPU footprint for high performance
- Cross-platform but without any need to install browser, use Electron, etc.
- Visualisation via Nannou
- Great way to learn about low-level audio sample/buffer control, multi-threading in Rust (Nannou always uses separate "realtime" thread for audio)

TODO:
- [ ] Demonstrate running on Raspberry Pi - this might require disabling the GUI while Nannou is behind on its WGPU dependency
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
Some dependencies:
`sudo apt install libasound2-dev libssl-dev cmake`

This works more easily using the 64-bit version of Raspberry Pi OS, because the reported platform aarch64 is actually correct in this case. **The application can compile successfully and run, but panics as soon as the window/graphics need to be initialised.**

In theory, WGPU (and therefore, Nannou) should work just fine via Vulkan:
`sudo apt install vulkan-tools mesa-vulkan-drivers`
...However Nannou is a bit behind on its WGPU (and egui WGPU backend) which means that the Pi is not well supported yet.

For now, a good compromise might be to allow Tether Soundscape to launch on the Pi without any window or graphics at all.
### A note on cross-compiling

The ALSO dependency does mean that cross-compiling from MacOS is currently stuck at building the [alsa-sys](https://crates.io/crates/alsa-sys) crate. This might be solvable, e.g. with https://github.com/cross-rs/cross
