//! Fontdue is both a font parser and rasterizer.
//!
//! This is a #![no_std] crate, but still requires alloc.

#![no_std]
#![allow(dead_code)]

extern crate alloc;

mod font;
mod math;
mod parse;
mod raster;
/// Functionality for the lower level raw data that was parsed from the font. Raw is unstable and
/// prone to change.
pub mod raw;
mod table;

pub use crate::font::*;

/// Alias for Result<T, &'static str>.
pub type FontResult<T> = Result<T, &'static str>;
