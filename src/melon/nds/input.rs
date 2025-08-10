use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NdsInputState {
    pub keys: NdsKeyMask,
    pub touch: Option<(u8, u8)>,
    pub lid_changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NdsInput {
    KeyPress(NdsKey),
    KeyRelease(NdsKey),
    TouchScreen((u8, u8)),
    ReleaseScreen,
    OpenCloseLid,
    // TODO: mic and camera inputs?
    // But, that would be hard to save replays for...
    // IDEA: separate replay file for this, with special encoding?
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NdsKeyboardInput {
    Key(NdsKey),
    OpenCloseLid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NdsKey {
    A,
    B,
    Select,
    Start,
    Right,
    Left,
    Up,
    Down,
    R,
    L,
    X,
    Y,
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct NdsKeyMask: u32 {
        const A = 0b0000_0000_0001;
        const B = 0b0000_0000_0010;
        const Select = 0b0000_0000_0100;
        const Start = 0b0000_0000_1000;
        const Right = 0b0000_0001_0000;
        const Left = 0b0000_0010_0000;
        const Up = 0b0000_0100_0000;
        const Down = 0b0000_1000_0000;
        const R = 0b0001_0000_0000;
        const L = 0b0010_0000_0000;
        const X = 0b0100_0000_0000;
        const Y = 0b1000_0000_0000;
    }
}

impl NdsInputState {
    pub fn new() -> Self {
        NdsInputState {
            keys: NdsKeyMask::empty(),
            touch: None,
            lid_changed: false,
        }
    }

    pub fn register_input(&mut self, input: NdsInput) {
        match input {
            NdsInput::KeyPress(key) => self.keys.insert(key.into()),
            NdsInput::KeyRelease(key) => self.keys.remove(key.into()),
            NdsInput::TouchScreen((x, y)) => self.touch = Some((x, y)),
            NdsInput::ReleaseScreen => self.touch = None,
            // TODO, do I change self.lid_open as well, or wait until the frame to change that?
            NdsInput::OpenCloseLid => self.lid_changed = true,
        }
    }
}

impl NdsKeyboardInput {
    pub fn press(self) -> Option<NdsInput> {
        match self {
            NdsKeyboardInput::Key(key) => Some(NdsInput::KeyPress(key)),
            NdsKeyboardInput::OpenCloseLid => Some(NdsInput::OpenCloseLid),
        }
    }

    pub fn release(self) -> Option<NdsInput> {
        match self {
            NdsKeyboardInput::Key(key) => Some(NdsInput::KeyRelease(key)),
            // releasing the button has no function
            NdsKeyboardInput::OpenCloseLid => None,
        }
    }
}

impl From<NdsKey> for NdsKeyMask {
    fn from(value: NdsKey) -> Self {
        match value {
            NdsKey::A => NdsKeyMask::A,
            NdsKey::B => NdsKeyMask::B,
            NdsKey::Select => NdsKeyMask::Select,
            NdsKey::Start => NdsKeyMask::Start,
            NdsKey::Right => NdsKeyMask::Right,
            NdsKey::Left => NdsKeyMask::Left,
            NdsKey::Up => NdsKeyMask::Up,
            NdsKey::Down => NdsKeyMask::Down,
            NdsKey::R => NdsKeyMask::R,
            NdsKey::L => NdsKeyMask::L,
            NdsKey::X => NdsKeyMask::X,
            NdsKey::Y => NdsKeyMask::Y,
        }
    }
}
