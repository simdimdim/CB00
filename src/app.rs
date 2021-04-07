use crate::common::{Draw, Prepare};
use average::WeightedMean;
use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::{Flip, GfxGraphics, ImageSize, TextureSettings};
use graphics::{image, Context, Transformed};
use header::HeaderValue;
use home;
use itertools::Itertools;
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
use reqwest::{
    header::{self, HeaderMap},
    Client,
    Url,
};
use sdl2_window::Sdl2Window;
use select::{document::Document, predicate::Name};
use std::{
    cmp::max,
    collections::{BTreeMap, HashMap},
    fmt::{Debug, Display},
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
        Self {
            title:    "Reader".to_string(),
            current:  0,
            panes:    HashMap::new(),
            settings: Settings::default(),
            width:    1.,
            height:   1.,
            ar:       1.,
            client:   Client::new(),
        }
    }
}
impl Default for Folder {
    fn default() -> Self {
        Self {
            url:       Url::from_directory_path(
                home::home_dir().unwrap().to_str().unwrap(),
            )
            .ok()
            .unwrap(),
            items:     BTreeMap::new(),
            changed:   true,
            direction: true,
            size:      0,
            maxdim:    (0, 0),
            batch:     2,
            index:     0,
            stats:     (1, (1, 1), 0.),
        }
    }
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
    //add new fields to Debug impl
}
#[derive(Clone, Debug)]
pub struct App {
    pub title:    String,
    current:      u16,
    panes:        HashMap<u16, Vec<Folder>>,
    pub settings: Settings,
    pub width:    f64,
    pub height:   f64,
    pub ar:       f64,
    client:       Client,
}
#[derive(Clone, Debug)]
pub struct Folder {
    url:       Url,
    items:     BTreeMap<Url, Picture>,
    changed:   bool,
    direction: bool,
    size:      u64,
    maxdim:    (u32, u32),
    batch:     u8,
    index:     usize,
    stats:     (usize, (u32, u32), f64),
}
#[derive(Clone, Debug)]
pub struct Picture {
    pub path: PathBuf,
    pub w:    u32,
    pub h:    u32,
    pub size: u64,
    pub tex:  Option<Texture<Resources>>,
}
#[derive(Debug)]
enum Source {
    Path(PathBuf),
    Url(Url),
}

impl App {
    pub fn new() -> Self {
        Self {
            title: "main".to_string(),
            ..App::default()
        }
    }

    pub async fn test(&mut self) {
        let mut headers = header::HeaderMap::new();
        headers
            .insert(header::REFERER, "https://manganelo.com/".parse().unwrap());
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
        const PATH: &str = "/tmp/readerapp/";
        for (n, (_, &url)) in zip.into_iter().enumerate() {
            pics.push(
                self.client
                    .get(url)
                    .header(
                        header::REFERER,
                        "https://manganelo.com/".parse::<HeaderValue>().unwrap(),
                    )
                    .send()
                    .await
                    .ok()
                    .unwrap()
                    .bytes()
                    .await
                    .ok()
                    .unwrap(),
            );
            let tmp = Url::parse(url).ok().unwrap();
            let name =
                tmp.path_segments().map(|c| c.collect::<Vec<_>>()).unwrap();
            fs::create_dir_all(PATH).ok().unwrap();
            if let Ok(mut dest) =
                File::create(format!("{}{}", PATH, name[name.len() - 1]))
            {
                dest.write(&pics[n]).ok().unwrap();
            }
        }
    }

    pub async fn fetch(
        &mut self,
        _folder: Folder,
    ) {
    }

    pub fn next_page(&mut self) {
        self.panes
            .get_mut(&self.current)
            .unwrap()
            .first_mut()
            .unwrap()
            .next_page();
    }

    pub fn prev_page(&mut self) {
        self.panes
            .get_mut(&self.current)
            .unwrap()
            .first_mut()
            .unwrap()
            .prev_page();
    }

    pub fn more(&mut self) {
        self.panes
            .get_mut(&self.current)
            .unwrap()
            .iter_mut()
            .for_each(|f| f.more())
    }

    pub fn less(&mut self) {
        self.panes
            .get_mut(&self.current)
            .unwrap()
            .iter_mut()
            .for_each(|f| f.less())
    }

    pub fn toggle_direction(&mut self) {
        self.panes
            .get_mut(&self.current)
            .unwrap()
            .iter_mut()
            .for_each(|f| f.toggle_direction());
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
                ..Self::default()
            },
            Err(_) => Self::default(),
        }
    }

    pub fn read(&mut self) {
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
                        .then(|| self.add_from(entry.path()));
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
                // &entry.path().extension().unwrap().to_str().unwrap(),
                //         )
                //     {
                //         self.items.insert(
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

    #[allow(unused_variables, unreachable_code)]
    fn add_from(
        &mut self,
        link: impl Into<Source>,
    ) -> Option<Picture> {
        self.changed = true;
        match link.into() {
            Source::Path(pb) => self.items.insert(
                Url::from_directory_path(&pb).ok().unwrap(),
                Picture {
                    path: pb,
                    ..Picture::default()
                },
            ),
            Source::Url(url) => {
                todo!("Needs logic for temp dir allocation and file dl");
                self.items.insert(url, Picture {
                    path: PathBuf::from(url.path()),
                    ..Picture::default()
                })
            }
        }
    }

    #[allow(dead_code, unused_variables, unreachable_code)]
    async fn download(
        &mut self,
        client: &Client,
        headers: Option<HeaderMap>,
    ) {
        let request = client
            .get(self.url.clone())
            .send()
            .await
            .ok()
            .unwrap()
            .text()
            .await
            .ok()
            .unwrap();
        todo!("Get webpage and find pics");
        let mut request = client.get(self.url.clone());
        todo!("Get webpage and find pics");
        if let Some(headers) = headers {
            for (k, v) in headers.iter() {
                request = request.header(k, v);
            }
        }
        todo!("Get webpage and find pics");
        todo!("iter over pics and add them to self.items");
        self.changed = true;
    }

    fn scheme(&self) -> &str { self.url.scheme() }

    pub fn _origin(&self) -> Origin { self.url.origin() }

    pub fn path(&self) -> &str { self.url.path() }

    pub fn next_page(&mut self) {
        if (self.index + 1) as usize * (self.batch as usize) <
            self.items.values().len()
        {
            self.index += 1;
        }
    }

    pub fn prev_page(&mut self) { self.index = self.index.saturating_sub(1); }

    pub fn more(&mut self) {
        self.batch = (self.items.len() / self.index.max(1) as usize)
            .min(self.batch as usize + 1) as u8;
        self.changed = true;
    }

    pub fn less(&mut self) {
        self.batch = self.batch.saturating_sub(2) + 1;
        self.changed = true;
    }

    pub fn toggle_direction(&mut self) { self.direction ^= true; }

    // TODO: Severe optimisation required.
    pub fn folder_stats(
        &mut self,
        w: f64,
        h: f64,
    ) {
        self.stats = (1..=self.items.len() / self.batch as usize)
            .map(|n| {
                (
                    self.items.len() / n,
                    self.items
                        .values()
                        .chunks(self.batch as usize)
                        .into_iter()
                        .nth(self.index)
                        .unwrap()
                        .chunks(n)
                        .into_iter()
                        .map(|c| c.fold((0, 0), |a, b| (a.0 + b.w, a.1.max(b.h))))
                        .fold((0, 0), |a, b| (a.0.max(b.0), a.1 + b.1)),
                )
            })
            .map(|(a, b)| {
                let x = w / b.0 as f64;
                let y = h / b.1 as f64;
                (a, b, (x - y).abs())
            })
            .min_by(|a, b| {
                a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Less)
            })
            .unwrap();
    }
}

impl From<PathBuf> for Source {
    fn from(x: PathBuf) -> Self { Self::Path(x) }
}
impl From<Url> for Source {
    fn from(x: Url) -> Self { Self::Url(x) }
}
impl Debug for Settings {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("fullscreen", &self.fullscreen)
            .field("vsync", &self.vsync)
            .field("capture", &self.capture)
            .field("esc_exit", &self.esc_exit)
            .field("transparent", &self.transparent)
            .field("ups", &self.ups)
            .field("fps", &self.fps)
            .field("samples", &self.samples)
            .field("opengl", &self.opengl)
            .finish()
    }
}
impl Display for Folder {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let s = (self.size as f64).log(1024.);
        let s2 = match s as i32 {
            0..1 => format!("{:.2}", s).to_string(),
            1..2 => format!("{:.2}K", s).to_string(),
            2..3 => format!("{:.2}M", s).to_string(),
            3..4 => format!("{:.2}G", s).to_string(),
            4..5 => format!("{:.2}T", s).to_string(),
            _ => "".to_string(),
        };
        write!(
            f,
            "Name: {},\nSize: {}",
            self.url
                .to_file_path()
                .ok()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            s2,
        )
    }
}
impl<'a> Prepare<'a> for App {
    fn prepare(
        &mut self,
        ctx: Self::Input,
    ) {
        self.panes.entry(0).or_insert(vec![Folder::default()]);
        for item in self.panes.values_mut().into_iter().flatten() {
            // item.download(&self.client, None);
            item.prepare((ctx, self.width, self.height));
        }
    }
}
impl<'a> Prepare<'a> for Folder {
    type Input = (&'a mut G2dTextureContext, f64, f64);

    fn prepare(
        &mut self,
        params: Self::Input,
    ) {
        // TODO: check for changes
        if self.changed {
            self.read();
            self.items
                .values_mut()
                .for_each(|pic| pic.prepare(params.0));
            self.size = self
                .items
                .values()
                .fold(0, |acc, Picture { size, .. }| acc + size);
            self.maxdim = self
                .items
                .values()
                .fold((0, 0), |acc, pic| (acc.0 + pic.w, acc.1 + pic.h));
        }
        self.folder_stats(params.1, params.2);
        self.changed = false;
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
impl Draw<'_> for Folder {
    type Params = (f64, f64);

    fn draw(
        &self,
        c: Context,
        g: &mut GfxGraphics<Resources, CommandBuffer>,
        dim: Self::Params,
    ) {
        let z = self.stats;
        let scale = dim.0 / z.1 .0 as f64;
        self.items
            .values()
            .into_iter()
            .chunks(self.batch as usize)
            .into_iter()
            .nth(self.index as usize)
            .unwrap()
            .enumerate()
            .fold((0., 0., 0.), |(x, y, h), (n, p)| {
                let gap = (dim.0 - ((z.1 .0 as f64) * scale)) / 2.;
                p.draw(c, g, (scale, &((x + gap) * scale, y as f64 * scale)));
                match n / z.0 {
                    0 => (x + p.w as f64, y + h, 0.),
                    _ => (0., y, (y as f64).max(p.h as f64)),
                }
            });
    }
}
impl Draw<'_> for App {
    fn draw(
        &self,
        c: Context,
        g: &mut GfxGraphics<Resources, CommandBuffer>,
        _: Self::Params,
    ) {
        self.panes
            .get(&self.current)
            .unwrap()
            .iter()
            .for_each(|a| a.draw(c, g, (self.width, self.height)));
    }
}

// fn _draw(
//     el: &mut Element,
//     ctx: &mut G2dTextureContext,
//     c: Context,
//     g: &mut GfxGraphics<Resources, CommandBuffer>,
//     offsetx: &mut u32,
//     offsety: &mut (u32, u32),
//     width: &f64,
//     _height: &f64,
// ) {
//             let new_offsetx = (ZOOM * *w as f64) as u32;
//             if ((*offsetx + new_offsetx) as f64) < *width - new_offsetx as
// f64 {                 *offsetx += new_offsetx;
//                 offsety.1 = max(
//                     (*h as f64 * ZOOM) as u32,
//                     (offsety.1 as f64 * ZOOM) as u32,
//                 );
//             } else {
//                 // dbg!(&offsety);
//                 offsety.0 += offsety.1;
//                 offsety.1 = 0;
//                 *offsetx = 0;
//             }
