pub(self) mod sys;
pub mod nds;
pub mod subscriptions;

pub fn init_renderer() {
    // defaulting to OpenGL renderer
    sys::gpu::InitRenderer(0);
}
