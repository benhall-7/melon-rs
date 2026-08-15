use crate::input::ConsoleInputState;

/// Console state after one emulated frame, for analysis hooks.
#[derive(Clone, Copy, Debug)]
pub struct FrameView<'a> {
    pub frame: u64,
    pub main_ram: &'a [u8],
    pub input: ConsoleInputState,
}

/// Inspects each finished frame without affecting emulation.
pub trait FrameObserver: Send {
    fn on_frame(&mut self, view: FrameView<'_>);
}
