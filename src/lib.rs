pub mod app;
pub mod audio;
pub mod config;
pub mod events;
pub mod frontend;
pub mod input;
pub mod melon;
pub mod observe;
pub mod overlay;
pub mod replay;
pub mod render;
pub mod run;
pub mod utils;

pub use input::ConsoleInputState;
pub use observe::{FrameObserver, FrameView};
pub use overlay::{Color, DrawCmd, Line, Outline, Overlay, Point, Rect, Screen, Text, TextAlign, TextFont};
pub use render::{RenderContext, RenderHook, RenderStatus, ScreenRect};
pub use run::{RunParams, run};

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
