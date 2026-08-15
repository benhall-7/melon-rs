mod draw;

pub use draw::draw_screen;

use crate::overlay::Overlay;

/// Emulation status forwarded to the UI for overlay hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderStatus {
    pub frame: u64,
    pub paused: bool,
}

/// Where a screen was drawn in window coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    pub min_x: f32,
    pub min_y: f32,
    pub width: f32,
    pub height: f32,
}

/// Console and window state available each repaint.
#[derive(Debug, Clone, Copy)]
pub struct RenderContext {
    pub frame: u64,
    pub paused: bool,
    /// Where the top screen was drawn in window coordinates.
    pub top_screen: ScreenRect,
    /// Where the bottom screen was drawn in window coordinates.
    pub bottom_screen: ScreenRect,
}

/// Builds an overlay each time the window repaints.
pub trait RenderHook: Send {
    fn on_render(&mut self, ctx: RenderContext) -> Overlay;
}
