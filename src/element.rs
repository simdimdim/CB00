use gfx_device_gl::Resources;
use gfx_graphics::{Flip, ImageSize, TextureSettings};
use piston_window::{texture, G2dTextureContext, Texture};
use std::{ffi::OsStr, fmt::Display, path::PathBuf};
use Element::*;

#[derive(Clone, Debug)]
pub enum Element {
    Default,
    File(PathBuf, Option<Picture>),
    Folder(PathBuf),
}
#[derive(Clone, Debug)]
pub struct Picture {
    pub w:    u32,
    pub h:    u32,
    pub size: u64,
    pub tex:  Option<Texture<Resources>>,
}

impl Element {
    pub fn new(pb: &PathBuf) -> Self {
        // let name = pb.file_name().unwrap().to_str().unwrap().into();
        if pb.is_dir() {
            Self::Folder(pb.into())
        } else if pb.is_file() {
            if "jpg" == pb.extension().unwrap_or(OsStr::new("")) {
                Self::File(
                    pb.into(),
                    Some(Picture {
                        w:    0,
                        h:    0,
                        size: pb.metadata().unwrap().len(),
                        tex:  None,
                    }),
                )
            } else {
                Self::File(pb.into(), None)
            }
        } else {
            Element::Default
        }
    }

    pub fn t(
        &mut self,
        ctx: &mut G2dTextureContext,
    ) {
        match self {
            Default => {}
            File(name, pic) => {
                if let Some(Picture { w, h, tex, .. }) = pic {
                    let texture = Texture::from_path(
                        ctx,
                        PathBuf::from(name.clone()).as_path(),
                        Flip::None,
                        &TextureSettings::new().filter(texture::Filter::Nearest),
                    );
                    if let Ok(t) = texture {
                        (*w, *h) = t.get_size();
                        *tex = Some(t);
                    }
                }
            }
            Folder(..) => {}
        }
    }
}

impl Display for Element {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let mut helper = |name: &str, size: &u64, w, h| {
            let s = (*size as f64).log(1024.);
            let s2 = match s as i32 {
                0..1 => format!("{:.2}", s).to_string(),
                1..2 => format!("{:.2}K", s).to_string(),
                2..3 => format!("{:.2}M", s).to_string(),
                3..4 => format!("{:.2}G", s).to_string(),
                4..5 => format!("{:.2}T", s).to_string(),
                _ => "".to_string(),
            };
            write!(f, "Name: {},\nSize: {},\nw: {}, h: {}", name, s2, w, h)
        };
        match self {
            Default => Ok(()),
            File(pb, pic) => {
                let name = pb.file_name().unwrap().to_str().unwrap().into();
                if let Some(Picture { w, h, size, .. }) = pic {
                    helper(name, size, w, h)
                } else {
                    write!(f, "Name: {}", name)
                }
            }
            Folder(pb) => {
                write!(f, "Name: {}", pb.file_name().unwrap().to_str().unwrap())
            }
        }
    }
}
