use super::{
    common::{Draw, Prepare, Source},
    contains,
    picture::Picture,
};
use crate::APPNAME;
use directories_next::{ProjectDirs, UserDirs};
use gfx_device_gl::{CommandBuffer, Resources};
use gfx_graphics::GfxGraphics;
use graphics::Context;
use itertools::Itertools;
use piston_window::G2dTextureContext;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::{header::HeaderMap, Client, Url};
use std::{collections::BTreeMap, fmt::Display, path::PathBuf};
use url::Origin;

impl Default for Folder {
    fn default() -> Self {
        Self {
            url:       Url::from_directory_path(Folder::home_dir()).ok().unwrap(),
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
impl Folder {
    pub fn new(path: &str) -> Self {
        let p = PathBuf::from(&path);
        let url = if p.canonicalize().is_ok() {
            Url::from_file_path(p.canonicalize().unwrap().as_path())
                .expect("Couldn't parse folder path")
        } else {
            Url::parse(path).expect("Couldn't parse url path")
        };

        Self {
            url,
            ..Self::default()
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
    fn add_from(&mut self, link: impl Into<Source>) -> Option<Picture> {
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
    async fn download(&mut self, client: &Client, headers: Option<HeaderMap>) {
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

    #[inline]
    pub fn home_dir() -> String {
        let user_dirs = UserDirs::new();
        user_dirs.unwrap().home_dir().to_str().unwrap().to_string()
    }

    #[inline]
    pub fn cache_dir() -> String {
        let proj_dirs = ProjectDirs::from("", "", APPNAME);
        proj_dirs.unwrap().cache_dir().to_str().unwrap().to_string()
    }

    pub fn next_page(&mut self) {
        if (self.index + 1) * (self.batch as usize) < self.items.values().len() {
            self.index += 1;
        }
    }

    pub fn prev_page(&mut self) { self.index = self.index.saturating_sub(1); }

    pub fn more(&mut self) {
        self.batch = (self.items.len() / self.index.max(1))
            .min(self.batch as usize + 1) as u8;
        self.changed = true;
    }

    pub fn less(&mut self) {
        self.batch = self.batch.saturating_sub(2) + 1;
        self.changed = true;
    }

    pub fn toggle_direction(&mut self) { self.direction ^= true; }

    // TODO: Severe optimisation required.
    pub fn folder_stats(&mut self, w: f64, h: f64) {
        self.stats = (1..=self.batch as usize / 2 + 1)
            .into_par_iter()
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

impl Draw<'_> for Folder {
    type Params = (f64, f64);

    fn draw(
        &self, c: Context, g: &mut GfxGraphics<Resources, CommandBuffer>,
        dim: Self::Params,
    ) {
        let z = self.stats;
        self.items
            .values()
            .chunks(self.batch as usize)
            .into_iter()
            .nth(self.index)
            .expect("Picking a chunk.")
            .enumerate()
            .fold((0., 0., 0., 0.), |(x, y, w, h), (n, p)| {
                let scale = match (
                    (z.1.0 as f64).min(dim.0) / (z.1.0 as f64).max(dim.0),
                    (z.1.1 as f64).min(dim.1) / (z.1.1 as f64).max(dim.1),
                ) {
                    (s, t) if s < t => s,
                    (s, t) if s > t => t,
                    (a, _) => a,
                };
                let _gap = (dim.0 * (z.1.0 as f64 / dim.0).ceil() -
                    (z.1.0 as f64)) *
                    scale /
                    2.;
                p.draw(c, g, (scale, &((x * scale), y * scale)));
                // dbg!(&(n, z.0, z.1, y, h, &scale));
                (
                    if (x + p.w as f64) < z.1.0 as f64 {
                        x + p.w as f64
                    } else {
                        0.
                    },
                    if (x + p.w as f64) < z.1.0 as f64 {
                        y
                    } else {
                        y + h
                    },
                    if (x + p.w as f64) > z.1.0 as f64 {
                        w + p.w as f64
                    } else {
                        0.
                    },
                    if z.0 > n { h.max(p.h as f64) } else { 0. },
                )
            });
    }
}
impl<'a> Prepare<'a> for Folder {
    type Input = (&'a mut G2dTextureContext, f64, f64);

    fn prepare(&mut self, params: Self::Input) {
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

impl Display for Folder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = (self.size as f64).log(1024.);
        let s2 = match s as i32 {
            0..1 => format!("{:.2}", s),
            1..2 => format!("{:.2}K", s),
            2..3 => format!("{:.2}M", s),
            3..4 => format!("{:.2}G", s),
            4..5 => format!("{:.2}T", s),
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
