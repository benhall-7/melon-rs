pub mod nds;
pub mod subscriptions;
pub(self) mod sys;

use crate::melon::sys::gpu::RenderSettings;

pub fn init_renderer() {
    sys::gpu::InitRenderer(0);
}

pub fn set_render_settings() {
    // defaulting to OpenGL renderer
    sys::gpu::SetRenderSettings(0, &mut RenderSettings {
        Soft_Threaded: false,
        GL_ScaleFactor: 1,
        GL_BetterPolygons: false,
    });
}
