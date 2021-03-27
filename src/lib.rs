//! Fontdue is a font parser, rasterizer, and layout tool.
//!
//! This is a #![no_std] crate, but still requires the alloc crate.
#![no_std]
#![allow(dead_code)]
#![allow(
    clippy::excessive_precision,
    clippy::approx_constant,
    clippy::float_cmp,
    clippy::needless_bool,
    clippy::upper_case_acronyms,
    clippy::many_single_char_names,
    clippy::wildcard_in_or_patterns,
    clippy::needless_return,
    clippy::transmute_int_to_float,
    clippy::transmute_float_to_int,
    clippy::ptr_arg,
    clippy::transmute_ptr_to_ptr,
    clippy::let_and_return,
    clippy::redundant_slicing,
)]

extern crate alloc;

mod font;
/// Tools for laying out strings of text.
pub mod layout;
mod math;
mod platform;
mod raster;
mod unicode;

pub use crate::font::*;

/// Alias for Result<T, &'static str>.
pub type FontResult<T> = Result<T, &'static str>;
