//! Game-agnostic overlay primitives in console screen space.
//!
//! Coordinates are expressed in pixels on a 256×192 NDS screen, origin top-left.
//! Consumers map game/world coordinates into this space; melon-rs maps this
//! space into the window when drawing.

mod cmd;
mod color;
mod overlay;

pub use cmd::{DrawCmd, Line, Point, Rect, Text};
pub use color::Color;
pub use overlay::{Overlay, Screen};
