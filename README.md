# Tether Soundscape rs

A multi-layered audio sequencer, remote-controllable via Tether, to create soundscapes.

![screenshot animation](./soundscape.gif)

## Sample bank JSON
Currently, the Sample Bank JSON files are created "by hand". Later versions will allow creation, editing and saving of these via the GUI. See `./test.json` file for an example.

## Remote control (Input from Tether)

### Single Clip Commands
On the topic `+/+/clipCommands`

Has the following fields
- `command` (required): one of the following strings: "hit", "add", "remove"
  - "hit" does not loop
  - "add" does loop
- `clipName` (required): string name for the targetted clip
- `fadeDuration` (optional): an integer value for milliseconds to fade in or out (command-dependent)
- `panPosition`, `panSpread` (both optional): if `panPosition` is specified, this will override any per-clip panning specified in the Sample Bank JSON

### Scene Messages
On the topic `+/+/scenes`

Has the following fields
- `mode` (optional, default is "loopAll"): one of the following strings: "loopAll", "onceAll", "onceRandom",
- `clipNames` (required): zero or more clip names; if zero are provided, the system will transition to an empty scene (silence all clips)
- `fade_duration` (optional):  an integer value for milliseconds to transition from current scene to the new one

### Global Controls
On the topic `+/+/globalControls`

**TODO: these are not functional yet**
### Examples

Single clip hit:
```
tether-send --host 127.0.0.1 --topic dummy/dummy/clipCommands --message \{\"command\":\"hit\"\,\"clipName\":\"frog\"\}
```

Single clip hit, specify panning (ignored if in Stereo Mode):
```
tether-send --host 127.0.0.1 --topic dummy/dummy/clipCommands --message \{\"command\":\"hit\"\,\"clipName\":\"frog\"\,\"panPosition\":0,\"panSpread\":1\}
```


Scene with two clips (default mode is "loopAll"):
```
tether-send --host 127.0.0.1 --topic dummy/dummy/scenes --message \{\"clipNames\":\[\"frog\"\,\"squirrel\"]\}
```

Scene where system should "pick one random" from the list:
```
tether-send --host 127.0.0.1 --topic dummy/dummy/scenes --message \{\"mode\":\"random\",\"clipNames\":\[\"frog\"\,\"squirrel\"]\}
```

Remove single clip
```
tether-send --host 127.0.0.1 --topic dummy/dummy/clipCommands --message \{\"command\":\"remove\",\"clipName\":\"frog\"\}
```

Add single clip, custom fade duration
```
tether-send --host 127.0.0.1 --topic dummy/dummy/clipCommands --message \{\"command\":\"add\",\"clipName\":\"squirrel2\",\"fadeDuration\":5000\}
```

Scene with zero clips (silence all), custom fade duration:
```
tether-send --host 127.0.0.1 --topic dummy/dummy/scenes --message \{\"clipNames\":\[\],\"fadeDuration\":500\}
```

## Output to Tether

### State
This agent publishes frequently (every UPDATE_INTERVAL ms) on the topic `soundscape/unknown/state`, which can be useful for driving animation, lighting effects, visualisation, etc. in sync with playback. The state messages include the following fields:

- `isPlaying`: whether or not the audio stream is playing
- `clips`: an array of currently playing clips (only), with the following information for each:
  - `id` (int)
  - `name` (string)
  - `progress` (float, normalised to range [0,1])
  - `currentVolume` (float, normalised to range [0,1])
  - `looping` (boolean)

To minimise traffic, the agent will only publish an empty clip list (`clips: []`) **once** and then resume as soon as at least one clip begins playing again.

### Events
TODO: discrete events (clip begin/end) should be published in addition to the stream of "state" messages. This could be useful for driving external applications that only need to subscribe to significant begin/end events.

___
## Why 🦀 Rust?:
- Minimal memory/CPU footprint for high performance
- Cross-platform but without any need to install browser, use Electron, etc.
- Visualisation via Nannou
- Great way to learn about low-level audio sample/buffer control, multi-threading in Rust (Nannou always uses separate "realtime" thread for audio)

___ 

## TODO - rodio/egui version:
- [x] Re-implement Phases
- [x] Fade in/out should use Phase/Tweens
- [x] Volume respected from sample bank?
- [x] Text-only mode
- [x] Panning reimplemented: use https://docs.rs/rodio/latest/rodio/source/struct.ChannelVolume.html ?
- [x] GUI show Tether enabled/connected status
- [x] Publish state regularly / events on events
- [x] GUI show incoming messages and/or counts
- [x] Demonstrate running (headless?) on Raspberry Pi
- [ ] Volume should be overrideable (as is the case for panning) in messages
- [ ] Refine the panning position/spread format and document it. Should panning be normalised or in range [0;channels-1]? Should spread have a minimum of 1 (="only target channel or adding up to 1 if between two channels")?
- [ ] Must be able to specify Group/ID for Tether (publishing)
- [ ] Allow input plugs to be subscribed to with a specified group (optional), so `+/someGroup/clipCommands` rather than the default `+/+/clipCommands`, and also publish on `soundscape/someGroup/state` 
- [ ] Stream/global level instructions, e.g. "play", "pause" (all), "silenceAll", "master volume", etc.
- [ ] Allow MIDI to trigger clips (MIDI Mediator and/or directly)
- [ ] Allow bank to be created, edited, saved directly from GUI, start from "blank" or load demo if nothing
- [ ] Drag and drop samples into bank
- [ ] Visualise clip playback in circles, not just progress bars
- [ ] Make use of tempo, quantisation for timing
- [ ] Provide utility/test modes, e.g. tone per channel
- [ ] Optionally connect to [Ableton link](https://docs.rs/ableton-link/latest/ableton_link/)
- [ ] Basic ADSR (or just Attack-Release) triggering for samples
- [ ] GUI show output levels per channel somehow? (depends on https://github.com/RustAudio/rodio/issues/475)
