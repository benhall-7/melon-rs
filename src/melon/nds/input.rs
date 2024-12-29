use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NdsInput {
    Key(NdsKey),
    Touch((u8, u8)),
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
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
