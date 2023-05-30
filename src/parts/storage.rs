use super::Store;
use serde::{Deserialize, Serialize};
use std::cmp::max;

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
        Self {
            fullscreen,
            vsync,
            capture,
            esc_exit,
            transparent,
            ups,
            fps,
            samples,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Settings {
    pub fullscreen:  bool,
    pub vsync:       bool,
    pub capture:     bool,
    pub esc_exit:    bool,
    pub transparent: bool,
    pub ups:         u64,
    pub fps:         u64,
    pub samples:     u8,
}
#[derive(Clone)]
struct Manager {}
impl Manager {
    //font dir browser
}

impl Store for Settings {
    #[inline]
    fn name() -> String { "Settings".to_string() }
}
