use super::{
    common::{Draw, Prepare},
    Folder,
};
use average::WeightedMean;
use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::GfxGraphics;
use graphics::Context;
use header::HeaderValue;
use levenshtein::levenshtein as lev;
use piston_window::{OpenGL, PistonWindow, Size, Window, WindowSettings};
use reqwest::{header, Client, Url};
use sdl2_window::Sdl2Window;
use select::{document::Document, predicate::Name};
use std::{cmp::max, collections::HashMap, fmt::Debug, fs::File, io::Write};

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
            cursor:   [0.; 2],
            width:    1.,
            height:   1.,
            ar:       1.,
            client:   Client::new(),
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
    cursor:       [f64; 2],
    pub width:    f64,
    pub height:   f64,
    pub ar:       f64,
    client:       Client,
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
            std::fs::create_dir_all(PATH).ok().unwrap();
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

    pub fn add_folder(
        &mut self,
        path: String,
    ) {
        self.panes
            .entry(self.current)
            .or_insert_with(|| vec![Folder::new(&path)]);
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

    pub fn cursor<'a>(
        &'a mut self,
        cursor: [f64; 2],
    ) -> &'a [f64; 2] {
        self.cursor = cursor;
        &self.cursor
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

impl<'a> Prepare<'a> for App {
    fn prepare(
        &mut self,
        ctx: Self::Input,
    ) {
        for item in self.panes.values_mut().into_iter().flatten() {
            // item.download(&self.client, None);
            item.prepare((ctx, self.width, self.height));
        }
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
