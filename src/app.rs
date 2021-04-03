use crate::common::{Draw, Prepare};
use average::WeightedMean;
use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::{Flip, GfxGraphics, TextureSettings};
use graphics::Context;
use home;
use indexmap::IndexMap;
use levenshtein::levenshtein as lev;
use piston_window::{
    texture,
    G2dTextureContext,
    OpenGL,
    PistonWindow,
    Size,
    Texture,
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
use url::Origin;

const EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "bmp", "png"];
fn contains(s: &str) -> bool {
    let test = EXTENSIONS.iter().position(|&r| r == s).unwrap_or(999);
    match test {
        0..4 => true,
        _ => false,
    }
}

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
impl Default for App {
    fn default() -> Self {
        let mut headers = header::HeaderMap::new();
        headers
            .insert(header::REFERER, "https://manganelo.com/".parse().unwrap());
        let mut pm = HashMap::new();
        pm.insert(0, vec![Folder::default()]);
        Self {
            title:    "Reader".to_string(),
            pane:     0,
            panemap:  pm,
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
            url:     Url::from_directory_path(
                home::home_dir().unwrap().to_str().unwrap(),
            )
            .ok()
            .unwrap(),
            items:   IndexMap::new(),
            changed: true,
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
pub struct App {
    pub title:    String,
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
    url:     Url,
    items:   IndexMap<Url, (PathBuf, Option<Texture<Resources>>)>,
    changed: bool,
}
impl App {
    pub fn new() -> Self {
        Self {
            title:    "".to_string(),
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
            let path = format!("{}{}", "/tmp/readerapp/", name[name.len() - 1]);
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
}

impl Folder {
    pub fn new(path: &str) -> Self {
        match Url::parse(path) {
            Ok(url) => Self {
                url,
                items: IndexMap::new(),
                changed: true,
            },
            Err(_) => Self::default(),
        }
    }

    pub fn add(&mut self) {
        match self.scheme() {
            "file" => {
                let path = PathBuf::from(self.path());
                let dir = match path.is_dir() {
                    true => path.read_dir(),
                    false => path.parent().unwrap().read_dir(),
                };
                for entry in dir.ok().unwrap().filter_map(|a| a.ok()) {
                    entry
                        .path()
                        .is_file()
                        .then_some(contains(
                            entry
                                .path()
                                .extension()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or(""),
                        ))
                        .unwrap_or_default()
                        .then(|| self.add_from_path(entry.path()));
                }
                // dir.ok()
                //     .unwrap()
                //     .into_iter()
                //     .filter_map(|f| f.ok())
                //     .filter(|entry| {
                //         entry.path().is_file() &&
                //             EXTENSIONS.contains(
                //                 &entry
                //                     .path()
                //                     .extension()
                //                     .unwrap()
                //                     .to_str()
                //                     .unwrap(),
                //             )
                //     })
                //     .for_each(|entry| {
                //         self.items.insert(
                //
                // Url::from_directory_path(entry.path()).ok().unwrap(),
                //             (entry.path(), None),
                //         );
                //     });
                // for entry in dir.ok().unwrap().filter_map(|a| a.ok()) {
                //     if entry.path().is_file() &&
                //         vec!["jpg"].contains(
                //
                // &entry.path().extension().unwrap().to_str().unwrap(),
                //         )
                //     {
                //         self.items.insert(
                //
                // Url::from_directory_path(entry.path()).ok().unwrap(),
                //             (entry.path(), None),
                //         );
                //     }
                // }
            }
            "http" | "https" => {}
            _ => {}
        }
        self.changed = true;
    }

    pub fn add_from_path(
        &mut self,
        pb: PathBuf,
    ) -> Option<(PathBuf, Option<Texture<Resources>>)> {
        self.changed = true;
        self.items
            .insert(Url::from_directory_path(&pb).ok().unwrap(), (pb, None))
    }

    #[allow(unused_variables, unreachable_code)]
    pub fn add_from_url(
        &mut self,
        url: Url,
    ) -> Option<(PathBuf, Option<Texture<Resources>>)> {
        self.changed = true;
        todo!("Needs logic for temp dir allocation and file dl");
        self.items.insert(url, (PathBuf::from(url.path()), None))
    }

    fn prepare(
        &mut self,
        ctx: &mut G2dTextureContext,
    ) {
        if self.changed {
            self.add();
            self.load_textures(ctx);
            self.changed = false;
        }
    }

    fn scheme(&self) -> &str { self.url.scheme() }

    fn _origin(&self) -> Origin { self.url.origin() }

    fn path(&self) -> &str { self.url.path() }
}
impl Draw for Folder {
    fn draw(
        &mut self,
        ctx: &mut G2dTextureContext,
        _c: Context,
        _g: &mut GfxGraphics<Resources, CommandBuffer>,
    ) {
        if self.changed {
            self.prepare(ctx);
            self.changed = false;
        }
    }
}
impl Prepare for Folder {
    fn load_textures(
        &mut self,
        ctx: &mut G2dTextureContext,
    ) {
        for (_, (pb, tex)) in self.items.iter_mut() {
            *tex = Some(
                Texture::from_path(
                    ctx,
                    pb,
                    Flip::None,
                    &TextureSettings::new().filter(texture::Filter::Nearest),
                )
                .ok()
                .unwrap(),
            );
        }
    }
}
impl Draw for App {
    fn draw(
        &mut self,
        ctx: &mut G2dTextureContext,
        c: Context,
        g: &mut GfxGraphics<Resources, CommandBuffer>,
    ) {
        self.panemap.get_mut(&self.pane).unwrap()[0].draw(ctx, c, g);
    }
}
