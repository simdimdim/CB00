use average::WeightedMean;
use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::GfxGraphics;
use graphics::Context;
use indexmap::IndexMap;
use levenshtein::levenshtein as lev;
use piston_window::{
    G2dTextureContext,
    OpenGL,
    PistonWindow,
    Size,
    Window,
    WindowSettings,
};
use reqwest::{header, Client, Url};
use sdl2_window::Sdl2Window;
use select::{document::Document, predicate::Name};
use std::{
    cmp::max,
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

impl Default for Settings {
    fn default() -> Self {
        let fullscreen = false;
        let vsync = false;
        let capture = false;
        let esc_exit = true;
        let transparent = true;
        let ups = 30;
        let fps = max(ups * 2, 60);
        let samples = 16;
        let opengl = OpenGL::V4_5;
        let mut window = WindowSettings::new("Reader", [1., 1.])
            .exit_on_esc(esc_exit)
            .samples(samples)
            .vsync(vsync)
            .graphics_api(opengl);
        window.set_transparent(transparent);
        Self {
            window,
            fullscreen,
            vsync,
            capture,
            esc_exit,
            transparent,
            ups,
            fps,
            samples,
            opengl,
        }
    }
}
impl Default for App<'_> {
    fn default() -> Self {
        let mut headers = header::HeaderMap::new();
        headers
            .insert(header::REFERER, "https://manganelo.com/".parse().unwrap());
        Self {
            title:    "Reader",
            pane:     0,
            panemap:  HashMap::new(),
            settings: Settings::default(),
            width:    0.,
            height:   0.,
            ar:       0.,
            client:   Client::builder()
                .default_headers(headers)
                .build()
                .ok()
                .unwrap(),
        }
    }
}
impl Default for Folder {
    fn default() -> Self {
        Self {
            location: Url::parse(
                fs::canonicalize(&PathBuf::from("~"))
                    .ok()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )
            .ok()
            .unwrap(),
            items:    IndexMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct Settings {
    pub fullscreen:  bool,
    pub vsync:       bool,
    pub capture:     bool,
    pub esc_exit:    bool,
    pub transparent: bool,
    pub ups:         u64,
    pub fps:         u64,
    pub samples:     u8,
    pub opengl:      OpenGL,
    pub window:      WindowSettings,
}
#[derive(Clone)]
pub struct App<'a> {
    pub title:    &'a str,
    pane:         u16,
    panemap:      HashMap<u16, Vec<Folder>>,
    pub settings: Settings,
    pub width:    f64,
    pub height:   f64,
    pub ar:       f64,
    client:       Client,
}
#[derive(Clone, Debug)]
pub struct Folder {
    location: Url,
    items:    IndexMap<u32, Url>,
}
impl App<'_> {
    pub fn new() -> Self {
        Self {
            title:    "",
            pane:     0,
            panemap:  HashMap::new(),
            settings: Settings::default(),
            width:    1.,
            height:   1.,
            ar:       1.,
            client:   Client::new(),
        }
    }

    pub async fn test(&mut self) {
        let html: String = self
            .client
            .get("https://manganelo.com/chapter/ni924247/chapter_22")
            .send()
            .await
            .ok()
            .unwrap()
            .text()
            .await
            .ok()
            .unwrap();
        let doc = Document::from(html.as_str());
        let mut pics = vec![];
        let links: Vec<_> = doc
            .select(Name("img"))
            .filter_map(|n| n.attr("src"))
            // .map(|a| Url::parse(a).unwrap().path().to_string())
            .collect();
        let mut dist: Vec<i32> = links[1..]
            .iter()
            .map(|a| lev(&links[0], &a) as i32)
            .collect();
        dist.push(0);
        dist.rotate_right(1);
        let m: WeightedMean = dist
            .clone()
            .iter()
            .zip(&dist)
            .filter(|(&a, _)| a > 0)
            .map(|(&x, &w)| (f64::from(x), f64::from(w)))
            .collect();
        dist.iter_mut().for_each(|a| *a -= m.mean() as i32);
        let mut zip: Vec<_> = dist.iter().zip(&links).collect();
        zip.retain(|(&a, _)| a < 3 && a >= 0);
        for (n, (_, &url)) in zip.into_iter().enumerate() {
            pics.push(
                self.client
                    .get(url)
                    .send()
                    .await
                    .ok()
                    .unwrap()
                    .bytes()
                    .await
                    .ok()
                    .unwrap(),
            );
            let tmp = Url::parse(url).clone().ok().unwrap();
            let name =
                tmp.path_segments().map(|c| c.collect::<Vec<_>>()).unwrap();
            let path = format!("{}{}", "/tmp/manganelo/", name[name.len() - 1]);
            fs::create_dir_all(&path).ok().unwrap();
            if let Ok(mut dest) = File::create(path) {
                dest.write(&pics[n]).ok().unwrap();
            }
        }
    }

    pub fn resize(
        &mut self,
        window: &PistonWindow<Sdl2Window>,
    ) {
        Size {
            width:  self.width,
            height: self.height,
        } = window.window.draw_size();
        self.ar = self.width / self.height;
    }

    pub fn display(
        &self,
        _ctx: &mut G2dTextureContext,
        _c: Context,
        _g: &mut GfxGraphics<Resources, CommandBuffer>,
    ) {
        match self.pane {
            _ => {}
        }
    }
}
impl Folder {
    pub fn new(path: &str) -> Self {
        if let Ok(url) = Url::parse(&path) {
            Self {
                location: url,
                items:    IndexMap::new(),
            }
        } else {
            Self::default()
        }
    }

    pub fn add(&mut self) {
        match self.location.scheme() {
            "file" => {}
            "http" | "https" => {
                // let texture = Texture::from_path(
                //         ctx,
                //         PathBuf::from(name.clone()).as_path(),
                //         Flip::None,
                //         &TextureSettings::new().filter(texture::Filter::
                // Nearest);
            }
            _ => {}
        }
    }
}
