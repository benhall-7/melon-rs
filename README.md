# melon-rs

melon-rs is an experimental frontend for the melonDS emulator. This frontend compiles the core code to melonDS using the `cxx` FFI layer, and interacts with it through an OpenGL window.

## license

This code is licensed under GPL v3. To understand the terms, review the `LICENSE` file.

## features

- 60fps emulation (audio not included)
- Play/pause button
- Frame step button
- Save file loading
- Input configurability (via the `config.yml` file)

## games

So far, only Kirby Super Star Ultra has been tested, as this is the only game I intended to test. 2D games are all likely to work, but I can't say the same for 3D games (e.g. Animal Crossing: Wild World). You're welcome to test it out if you like

## todo

- Savestate support
- Input recording and playback
- Audio support
- Scripting environment
- Code hooking
- Debugging support (is this possible?)

## caveats

You might have to install the melonDS prerequisites before attempting to compile. I've only run this on mac. It's likely to work on linux too, and windows is a toss-up.
