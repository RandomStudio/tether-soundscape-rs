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
- [x] Allow starting/fixed "maximum" volume per clip to be applied
- [x] Apply fade in/out volume controls
- [ ] Get sample rate from files/metadata
- [ ] Allow "scenes" to be triggered (with transition)
- [ ] Make use of tempo, quantisation for timing
- [ ] Optionally connect to [Ableton link](https://docs.rs/ableton-link/latest/ableton_link/)
- [ ] On-screen UI (Egui), CLI params, etc.
- [ ] Low/no graphics mode
- [ ] Add Tether remote control commands, as per API in [original](https://github.com/RandomStudio/tether-soundscape)

