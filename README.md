# Tether Soundscape rs

A multi-layered audio sequencer, remote-controllable via Tether, to create soundscapes.

![screenshot animation](./soundscape.gif)

## Using 🦀 Rust because:
- Minimal memory/CPU footprint for high performance
- Cross-platform but without any need to install browser, use Electron, etc.
- Visualisation via Nannou
- Great way to learn about low-level audio sample/buffer control, multi-threading in Rust (Nannou always uses separate "realtime" thread for audio)

## Remote control
### Examples

Single clip hit:
```
tether-send --host 127.0.0.1 --topic dummy/dummy/instructions --message \{\"instructionType\":\"hit\"\,\"clipNames\":\[\"frog\"\]\}
```

Multiple clip hits:
```
tether-send --host 127.0.0.1 --topic dummy/dummy/instructions --message \{\"instructionType\":\"hit\"\,\"clipNames\":\[\"frog\"\,\"squirrel\"]\}
```

Scene with two clips:
```
tether-send --host 127.0.0.1 --topic dummy/dummy/instructions --message \{\"instructionType\":\"scene\",\"clipNames\":\[\"frog\"\,\"squirrel\"]\}
```

Remove single clip
```
tether-send --host 127.0.0.1 --topic dummy/dummy/instructions --message \{\"instructionType\":\"remove\",\"clipNames\":\[\"frog\"\]\}
```

Add single clip, custom fade duration
```
tether-send --host 127.0.0.1 --topic dummy/dummy/instructions --message \{\"instructionType\":\"add\",\"clipNames\":\[\"squirrel2\"\],\"fadeDuration\":5000\}
```

Scene with zero clips (silence all), custom fade duration:
```
tether-send --host 127.0.0.1 --topic dummy/dummy/instructions --message \{\"instructionType\":\"scene\"\,\"clipNames\":\[\],\"fadeDuration\":500\}
```


## TODO:
- [ ] Demonstrate running (headless) on Raspberry Pi
- [x] Apply "loop" as well as trigger/hit/once-off functions
- [x] Allow clips to be stopped/removed while playing (without stopping whole stream)
- [x] Allow starting/fixed "maximum" volume per clip to be applied
- [x] Apply fade in/out volume controls
- [x] Draw clip progress and volume
- [x] Allow "scenes" to be triggered (with transition)
- [ ] Make use of tempo, quantisation for timing
- [x] Env logging, CLI params
- [ ] Low/no graphics mode
- [ ] Add Tether remote control commands, as per API in [original](https://github.com/RandomStudio/tether-soundscape)
- [ ] Separate CLIP and STREAM sample rates are currently a problem - might need a separate Reader (and thread!) for each clip if sample rates are allowed to differ
- [ ] Optionally connect to [Ableton link](https://docs.rs/ableton-link/latest/ableton_link/)
- [ ] Possibly distribute radius by "index" not duration

