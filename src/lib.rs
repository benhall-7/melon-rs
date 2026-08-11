pub mod app;
pub mod audio;
pub mod config;
pub mod events;
pub mod frontend;
pub mod input;
pub mod melon;
pub mod replay;
pub mod utils;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EmuState {
    Running,
    Paused,
    Stepping,
    Stopped,
}

#[derive(Debug, Clone, Copy)]
pub enum EmuStateChange {
    PlayPause,
    Step,
    Stop,
}
