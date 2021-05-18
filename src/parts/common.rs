use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::GfxGraphics;
use graphics::Context;
use piston_window::G2dTextureContext;
use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

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

pub trait Store: DeserializeOwned + Serialize {
    fn name() -> String;
    fn save(&self) {}
    fn load(&self) {}
}

#[derive(Debug)]
pub enum Source {
    Path(PathBuf),
    Url(Url),
}
impl From<PathBuf> for Source {
    fn from(x: PathBuf) -> Self { Self::Path(x) }
}
impl From<Url> for Source {
    fn from(x: Url) -> Self { Self::Url(x) }
}
