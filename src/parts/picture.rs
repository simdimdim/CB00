use super::common::{Draw, Prepare};
use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::{Flip, GfxGraphics, ImageSize, Texture, TextureSettings};
use graphics::{image, Context, Transformed};
use piston_window::texture;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Picture {
    pub path: PathBuf,
    pub w:    u32,
    pub h:    u32,
    pub size: u64,
    pub tex:  Option<Texture<Resources>>,
}
impl Default for Picture {
    fn default() -> Self {
        let pb = PathBuf::from(".");
        Self {
            path: pb.clone(),
            w:    0,
            h:    0,
            size: pb.metadata().unwrap().len(),
            tex:  None,
        }
    }
}
impl<'a> Draw<'a> for Picture {
    type Params = (f64, &'a (f64, f64));

    fn draw(
        &self,
        c: Context,
        g: &mut GfxGraphics<Resources, CommandBuffer>,
        params: Self::Params,
    ) {
        let transform = c
            .trans(params.1 .0, params.1 .1)
            .transform
            .append_transform(graphics::math::scale(params.0, params.0));
        if let Some(texture) = &self.tex {
            image(texture, transform, g);
        }
    }
}
impl<'a> Prepare<'a> for Picture {
    fn prepare(
        &mut self,
        ctx: Self::Input,
    ) {
        self.tex = match Texture::from_path(
            ctx,
            &self.path,
            Flip::None,
            &TextureSettings::new().filter(texture::Filter::Nearest),
        ) {
            Ok(t) => Some(t),
            Err(_) => None,
        };
        if let Some(t) = &self.tex {
            (self.w, self.h) = t.get_size();
        }
        self.size = self.path.metadata().unwrap().len();
    }
}
