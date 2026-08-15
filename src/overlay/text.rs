use super::Color;
use super::Point;

/// Built-in font selection for overlay text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextFont {
    #[default]
    Proportional,
    Monospace,
}

/// Anchor point for overlay text relative to `pos`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    LeftTop,
    CenterTop,
    RightTop,
    LeftCenter,
    Center,
    RightCenter,
    LeftBottom,
    CenterBottom,
    RightBottom,
}

/// Outline drawn behind text for legibility on busy backgrounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    pub color: Color,
    /// Width in console pixels.
    pub width: f32,
}

impl Outline {
    pub fn new(color: Color, width: f32) -> Self {
        Self { color, width }
    }
}

/// Text label. `size` is in console pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub pos: Point,
    pub text: String,
    pub color: Color,
    pub size: f32,
    pub font: TextFont,
    pub align: TextAlign,
    pub outline: Option<Outline>,
}

impl Text {
    pub fn new(pos: Point, text: impl Into<String>, color: Color) -> Self {
        Self {
            pos,
            text: text.into(),
            color,
            size: 8.0,
            font: TextFont::default(),
            align: TextAlign::default(),
            outline: None,
        }
    }

    pub fn monospace(mut self) -> Self {
        self.font = TextFont::Monospace;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn outline(mut self, outline: Outline) -> Self {
        self.outline = Some(outline);
        self
    }
}
