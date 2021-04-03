#![feature(exclusive_range_pattern)]
#![feature(destructuring_assignment)]
#![feature(bool_to_option)]

pub mod app;
pub mod common;
pub mod ui;

pub use self::app::App;
