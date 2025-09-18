#![feature(iter_map_windows)]
#![feature(int_roundings)]

pub mod prelude {
    pub use super::common::prelude::*;
    pub use super::analyses::prelude::*;
}

pub mod common;
pub mod analyses;