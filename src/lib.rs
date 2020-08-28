//! Fontdue is a font parser, rasterizer, and layout tool.
//!
//! This is a #![no_std] crate, but still requires the alloc crate.

#![no_std]
#![allow(dead_code)]

extern crate alloc;

mod font;
/// Tools for laying out strings of text.
pub mod layout;
mod math;
mod parse;
mod platform;
mod raster;
mod unicode;

pub use crate::font::*;

/// Alias for Result<T, &'static str>.
pub type FontResult<T> = Result<T, &'static str>;
