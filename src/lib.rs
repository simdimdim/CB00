#![feature(exclusive_range_pattern)]
#![feature(destructuring_assignment)]
pub mod app;
pub mod element;
pub mod ui;

pub use self::{app::App, element::Element};
use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

pub struct Assets {
    pub path:     PathBuf,
    pub elements: HashMap<PathBuf, Element>,
}
impl Default for Assets {
    fn default() -> Self {
        Assets::new("/home/thedoctor/Multimedia/hmanga/hypnotist1/")
    }
}
impl Assets {
    fn new(path: &str) -> Self {
        let path = PathBuf::from(path);
        let elements = HashMap::new();
        Self { path, elements }
    }

    pub fn list_files(&mut self) -> Vec<PathBuf> {
        let mut v = Vec::new();
        for entry in self.path.read_dir().expect("read_dir call failed") {
            if let Ok(entry) = entry {
                if "jpg" == entry.path().extension().unwrap_or(OsStr::new("")) {
                    v.push(entry.path());
                }
            }
        }
        // v.sort_by(|a, b| a.file_name().partial_cmp(&b.file_name()).unwrap());
        for e in v.iter() {
            self.elements.insert(e.clone(), Element::new(e));
        }
        v
    }
}
