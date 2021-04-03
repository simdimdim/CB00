use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::GfxGraphics;
use graphics::Context;
use piston_window::G2dTextureContext;

pub trait Draw {
    fn draw(
        &mut self,
        ctx: &mut G2dTextureContext,
        c: Context,
        g: &mut GfxGraphics<Resources, CommandBuffer>,
    );
}
pub trait Prepare {
    fn load_textures(
        &mut self,
        ctx: &mut G2dTextureContext,
    );
}
