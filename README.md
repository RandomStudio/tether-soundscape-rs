# Tether Soundscape rs

A multi-layered audio sequencer, remote-controllable via Tether, to create soundscapes. Runs in a full GUI mode or headless - even on a Raspberry Pi!

![screenshot animation](./soundscape.gif)


## Why 🦀 Rust?:
- Minimal memory/CPU footprint for high performance
- Cross-platform but without any need to install browser, use Electron, etc.
- Full GUI or headless (text-only) modes are possible
- Great way to learn about low-level audio sample/buffer control, multi-threading in Rust


## Sample bank JSON
Currently, the Sample Bank JSON files are created "by hand". Later versions will allow creation, editing and saving of these via the GUI. See `./test.json` file for an example.

### Volume and Panning defaults and overrides
Clips in the Sample Bank may optionally be given a `volume` and/or `panning` setting.

If an incoming `clipCommands` message specifies `volume` or `panning` values, then these will override any defaults specified in the JSON.

If neither a JSON-specified value nor a message-specified override is available for one or both of these, a default will be applied (full volume and centred panning).

See [Conventions](#conventions) for more detail on how these values are intended to be used.

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
   - `panSpread` on its own will be ignored
   - `panPosition` on its own will apply a default spread value (`0.0`)

See the [Conventions](#conventions) section for more detail on how these values are defined. 

### Scene Messages
On the topic `+/+/scenes`

Has the following fields
- `mode` (optional, default is "loopAll"): one of the following strings: "loopAll", "onceAll", "onceRandom",
- `clipNames` (required): zero or more clip names; if zero are provided, the system will transition to an empty scene (silence all clips)
- `fade_duration` (optional):  an integer value for milliseconds to transition from current scene to the new one

### Global Controls
On the topic `+/+/globalControls`

Has the following fields:
- `command`: one of the following:
  - "pause": pause (but do not stop or remove) all currently playing clips; ignored if already paused
  - "play": resume all clips; ignored if not already paused
  - "silence": immediately stop all clips (fast fade out)
  - "masterVolume": set all clips to the specified volume; in future this should probably adjust a final mix or output level
- `volume`: only used when command is "masterVolume"

### Examples
A project file for [Tether Egui](https://github.com/RandomStudio/tether-egui) is provided in `./soundscape-widgets.json` for easy testing of the remote control functions.

Alternatively, use the `tether send` commands below if using [Tether Utils](https://crates.io/crates/tether-utils).

Single clip hit:
```
tether send --plug.topic dummy/dummy/clipCommands --message \{\"command\":\"hit\"\,\"clipName\":\"frog\"\}
```

Single clip hit, specify panning (ignored if in Stereo Mode):
```
tether send --plug.topic dummy/dummy/clipCommands --message \{\"command\":\"hit\"\,\"clipName\":\"frog\"\,\"panPosition\":0,\"panSpread\":1\}
```


Scene with two clips (default mode is "loopAll"):
```
tether send --plug.topic dummy/dummy/scenes --message \{\"clipNames\":\[\"frog\"\,\"squirrel\"]\}
```

Scene where system should "pick one random" from the list:
```
tether send --plug.topic dummy/dummy/scenes --message \{\"mode\":\"random\",\"clipNames\":\[\"frog\"\,\"squirrel\"]\}
```

Remove single clip
```
tether send --plug.topic dummy/dummy/clipCommands --message \{\"command\":\"remove\",\"clipName\":\"frog\"\}
```

Add single clip, custom fade duration
```
tether send --plug.topic dummy/dummy/clipCommands --message \{\"command\":\"add\",\"clipName\":\"squirrel2\",\"fadeDuration\":5000\}
```

Scene with zero clips (silence all), custom fade duration:
```
tether send --plug.topic dummy/dummy/scenes --message \{\"clipNames\":\[\],\"fadeDuration\":500\}
```

## Output to Tether

### State
This agent publishes frequently on the topic `soundscape/any/state`, which can be useful for driving animation, lighting effects, visualisation, etc. in sync with playback. The state messages include the following fields:

- `isPlaying`: whether or not the audio stream is playing
- `clips`: an array of currently playing clips (only), with the following information for each:
  - `id` (int)
  - `name` (string)
  - `progress` (float, normalised to range [0,1])
  - `currentVolume` (float, normalised to range [0,1])
  - `looping` (boolean)

To minimise traffic, the agent will only publish an empty clip list (`clips: []`) **once** and then resume as soon as at least one clip begins playing again.

### Events
Discrete events (clip begin/end) are published on the `events` Plug, e.g. `soundscape/any/events`. This can be useful for driving external applications that only need to subscribe to significant begin/end events.

## Conventions
`volume` values are a multiplier, so `0.0` means silence and `1.0` means "full volume". A value > 1.0 will amplify the volume relative to the original source.

`panning` is separated into two distance keys (in JSON file and/or messages) and a tuple (in Rust, internally) - `position` followed by `spread`. These values are meant to be used as follows:
 - `position` (`panPosition` in JSON) is a value in the range `[0; output_channel_count - 1]`. So, in a 4 channel setup, position `3.0` would be "full right", i.e. loudest in channel 4.
 - `spread` (`panSpread` in JSON) is a multiple of the "width" of a channel. So, `0.0` means that the signal will be as focussed as possible, i.e. "1 channel width".


## TODO:
- [x] Demonstrate running (headless?) on Raspberry Pi
- [x] Volume should be overrideable (as is the case for panning) in messages
- [x] Refine the panning position/spread format and document it. Should panning be normalised or in range [0;channels-1]? Should spread have a minimum of 1 (="only target channel or adding up to 1 if between two channels")?
- [x] Must be able to specify Group/ID for Tether (publishing)
- [x] Allow input plugs to be subscribed to with a specified group (optional), so `+/someGroup/clipCommands` rather than the default `+/+/clipCommands`, and also publish on `soundscape/someGroup/state` 
- [x] Stream/global level instructions, e.g. "play", "pause" (all), "silence all", "master volume", etc.
- [ ] Allow MIDI to trigger clips (MIDI Mediator and/or directly)
- [ ] Allow bank to be created, edited, saved directly from GUI, start from "blank" or load demo if nothing
- [ ] Drag and drop samples into bank
- [ ] Visualise clip playback in circles, not just progress bars
- [ ] Make use of tempo, quantisation for timing
- [ ] Provide utility/test modes, e.g. tone per channel
- [ ] Optionally connect to [Ableton link](https://docs.rs/ableton-link/latest/ableton_link/)
- [ ] Basic ADSR (or just Attack-Release) triggering for samples
- [ ] GUI show output levels per channel somehow? (depends on https://github.com/RustAudio/rodio/issues/475)
- [ ] Replace generic/empty `Err(())` returns with something better, e.g. anyhow crate
