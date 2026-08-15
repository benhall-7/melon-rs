/// RGBA color with straight alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgba_u32(raw: u32) -> Self {
        Self {
            r: ((raw >> 24) & 0xff) as u8,
            g: ((raw >> 16) & 0xff) as u8,
            b: ((raw >> 8) & 0xff) as u8,
            a: (raw & 0xff) as u8,
        }
    }
}
