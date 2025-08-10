# melon-rs

melon-rs is an experimental frontend for the melonDS emulator. This frontend compiles the core code to melonDS using the `cxx` FFI layer, and interacts with it through an OpenGL window.

## license

This code is licensed under GPL v3. To understand the terms, review the `LICENSE` file.

## features

- Buttery smooth 60fps emulation
- Input configurability (via the `config.yml` file)
- Touchsceen support
- Save file loading
- Play/pause button
- Frame step button
- Savestate support
- Deterministic emulation
- Input recording and playback

## games

So far, only Kirby Super Star Ultra has been tested, as this is the only game I intended to test. Most games are likely to work, although maybe there's some that require special code I haven't handled. You're welcome to test it out if you like

## todo

- Video/audio encoding
- Sticky keys (eliminate missed inputs)
- Scripting environment
- Code hooking
- Debugging support (is it possible?)

## caveats

You might have to install the melonDS prerequisites before attempting to compile. I've run this on macOS and linux. Windows probably works too.
