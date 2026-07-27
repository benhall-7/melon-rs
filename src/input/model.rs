use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    /// Held active-high state for the twelve standard NDS buttons.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ButtonMask: u16 {
        const A      = 1 << 0;
        const B      = 1 << 1;
        const SELECT = 1 << 2;
        const START  = 1 << 3;
        const RIGHT  = 1 << 4;
        const LEFT   = 1 << 5;
        const UP     = 1 << 6;
        const DOWN   = 1 << 7;
        const R      = 1 << 8;
        const L      = 1 << 9;
        const X      = 1 << 10;
        const Y      = 1 << 11;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ConsoleButton {
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

impl From<ConsoleButton> for ButtonMask {
    fn from(button: ConsoleButton) -> Self {
        match button {
            ConsoleButton::A => Self::A,
            ConsoleButton::B => Self::B,
            ConsoleButton::Select => Self::SELECT,
            ConsoleButton::Start => Self::START,
            ConsoleButton::Right => Self::RIGHT,
            ConsoleButton::Left => Self::LEFT,
            ConsoleButton::Up => Self::UP,
            ConsoleButton::Down => Self::DOWN,
            ConsoleButton::R => Self::R,
            ConsoleButton::L => Self::L,
            ConsoleButton::X => Self::X,
            ConsoleButton::Y => Self::Y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TouchPoint {
    pub x: u8,
    pub y: u8,
}

impl TouchPoint {
    pub fn new(x: u8, y: u8) -> Option<Self> {
        (y < 192).then_some(Self { x, y })
    }
}

/// Complete held input state presented to the emulated console.
///
/// This intentionally excludes one-shot system actions and frontend commands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConsoleInputState {
    pub buttons: ButtonMask,
    pub touch: Option<TouchPoint>,
    pub lid_closed: bool,
}

/// A one-shot action that occurs at an input boundary rather than remaining held.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SystemAction {
    Reset,
    PowerCycle,
    InsertCartridge,
    EjectCartridge,
}

/// Monotonically increasing identity for the input boundary being sampled.
///
/// This is a frame index today. Its representation can later be replaced by a
/// frame/poll or cycle timestamp without changing the accumulator semantics.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct BoundaryIndex(pub u64);

/// Input selected for one emulation boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryInput {
    pub boundary: BoundaryIndex,
    pub state: ConsoleInputState,
    pub actions: Vec<SystemAction>,
}
