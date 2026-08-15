use super::Color;
use super::Text;

/// A point in console screen space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Axis-aligned rectangle outline or fill.
#[derive(Debug, Clone, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub width: f32,
    pub height: f32,
    pub stroke: Color,
    pub stroke_width: f32,
    pub fill: Option<Color>,
}

impl Rect {
    pub fn stroke(origin: Point, width: f32, height: f32, stroke: Color) -> Self {
        Self {
            origin,
            width,
            height,
            stroke,
            stroke_width: 1.0,
            fill: None,
        }
    }
}

/// Line segment.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub from: Point,
    pub to: Point,
    pub color: Color,
    pub width: f32,
}

impl Line {
    pub fn new(from: Point, to: Point, color: Color) -> Self {
        Self {
            from,
            to,
            color,
            width: 1.0,
        }
    }
}

/// One drawable primitive on a screen.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCmd {
    Rect(Rect),
    Line(Line),
    Text(Text),
}
