#![feature(exclusive_range_pattern)]
#![feature(destructuring_assignment)]
#![feature(bool_to_option)]
#![feature(associated_type_defaults)]

pub mod parts;

pub use self::parts::App;

const APPNAME: &str = "Reader";
