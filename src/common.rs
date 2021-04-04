use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::GfxGraphics;
use graphics::Context;
use piston_window::G2dTextureContext;

pub trait Draw<'a> {
    type Params = Option<()>;

    fn draw(
        &self,
        // ctx: &mut G2dTextureContext,
        c: Context,
        g: &mut GfxGraphics<Resources, CommandBuffer>,
        _: Self::Params,
    );
}
pub trait Prepare<'a> {
    type Input = &'a mut G2dTextureContext;

    fn prepare(
        &mut self,
        _: Self::Input,
    );
}
pub trait Update {
    type Input;

    fn update(
        &mut self,
        _: Self::Input,
    );
}
