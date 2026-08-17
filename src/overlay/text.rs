use super::{Color, Point};

pub const DEFAULT_CELL_WIDTH: f32 = 8.0;
pub const DEFAULT_CELL_HEIGHT: f32 = 10.0;
pub const DEFAULT_ADVANCE: f32 = 6.0;
pub const DEFAULT_FONT_SIZE: f32 = 9.0;

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

/// Text rendered with egui's built-in monospace font in fixed-size cells.
///
/// The cell dimensions provide deterministic layout metrics independently of
/// egui's glyph bearings. Glyphs are centered in their cells and may overhang.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub pos: Point,
    pub text: String,
    pub color: Color,
    pub cell_width: f32,
    pub cell_height: f32,
    pub advance: f32,
    pub font_size: f32,
    pub align: TextAlign,
    /// Drawn exactly to the fixed-cell bounds, without implicit padding.
    pub background: Option<Color>,
}

impl Text {
    pub fn new(pos: Point, text: impl Into<String>, color: Color) -> Self {
        Self {
            pos,
            text: text.into(),
            color,
            cell_width: DEFAULT_CELL_WIDTH,
            cell_height: DEFAULT_CELL_HEIGHT,
            advance: DEFAULT_ADVANCE,
            font_size: DEFAULT_FONT_SIZE,
            align: TextAlign::default(),
            background: None,
        }
    }

    pub fn cell_size(mut self, width: f32, height: f32) -> Self {
        self.cell_width = width;
        self.cell_height = height;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn character_advance(mut self, advance: f32) -> Self {
        self.advance = advance;
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn width(&self) -> f32 {
        match self.text.chars().count() {
            0 => 0.0,
            count => self.cell_width + (count - 1) as f32 * self.advance,
        }
    }
}
