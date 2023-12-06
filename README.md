# melon-rs

melon-rs is an experimental frontend for the melonDS emulator. This frontend compiles the core code to melonDS using the `cxx` FFI layer, and interacts with it through an OpenGL window.

## license

This code is licensed under GPL v3. To understand the terms, review the `LICENSE` file.

## features

- Smooth 60fps emulation (audio included!)
- Play/pause button
- Frame step button
- Save file loading
- Input configurability (via the `config.yml` file)
- Savestate support
- Determinism (AKA emulated time)
- Input recording and playback (janky)

## games

So far, only Kirby Super Star Ultra has been tested, as this is the only game I intended to test. Most games are likely to work, although maybe there's some that require special code I haven't handled. You're welcome to test it out if you like

## todo

- Input recording and playback (not janky)
- Touch screen support
- Scripting environment
- Code hooking
- Debugging support (is this possible?)

## caveats

You might have to install the melonDS prerequisites before attempting to compile. I've only run this on mac. It's likely to work on linux too, and windows is a toss-up.
