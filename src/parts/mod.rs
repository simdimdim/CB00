pub mod app;
pub mod common;
pub mod folder;
pub mod picture;
pub mod resources;
pub mod storage;
pub mod ui;

pub use self::{app::App, common::*, folder::Folder, picture::Picture};

const EXTENSIONS: [&str; 4] = ["jpg", "jpeg", "bmp", "png"];
fn contains(s: &str) -> bool {
    let test = EXTENSIONS.iter().position(|&r| r == s).unwrap_or(999);
    match test {
        0..4 => true,
        _ => false,
    }
}
