//! Fontdue is both a font parser and rasterizer.
//!
//! This is a #![no_std] crate, but still requires alloc.

#![no_std]
#![allow(dead_code)]

extern crate alloc;

mod math;
mod parse;
mod raster;
/// It holds the lower level raw data that was parsed from the font. Raw is unstable and prone to
/// change.
pub mod raw;
mod table;

pub use crate::math::*;
pub use crate::raster::*;

pub type FontResult<T> = Result<T, &'static str>;
