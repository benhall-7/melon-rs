pub mod app;
pub mod audio;
pub mod config;
pub mod events;
pub mod frontend;
pub mod input;
pub mod melon;
pub mod observe;
pub mod overlay;
pub mod render;
pub mod replay;
pub mod run;
pub mod utils;

pub use input::ConsoleInputState;
pub use observe::{FrameObserver, FrameView};
pub use overlay::{
    Color, DrawCmd, Text, Line, Overlay, Point, Rect, Screen, TextAlign, DEFAULT_ADVANCE,
    DEFAULT_CELL_HEIGHT, DEFAULT_CELL_WIDTH, DEFAULT_FONT_SIZE,
};
pub use render::{RenderContext, RenderHook, RenderStatus, ScreenRect};
pub use run::{run, RunParams};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EmuState {
    Running,
    Paused,
    Stepping,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmuStateChange {
    PlayPause,
    Step,
    Stop,
}
