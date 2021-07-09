//! Fontdue is a font parser, rasterizer, and layout tool.
//!
//! This is a #![no_std] crate, but still requires the alloc crate.
//!
//! ## Optional features
//! * `serde_derive` - Implementations of
//! [`Serialize`](https://docs.serde.rs/serde/ser/trait.Serialize.html) and
//! [`Deserialize`](https://docs.serde.rs/serde/de/trait.Deserialize.html) for important `fontdue`
//! types. Note that serialization works between builds of `fontdue` with and without SIMD enabled.

#![no_std]
#![allow(dead_code)]
#![allow(clippy::style)]
#![allow(clippy::complexity)]

extern crate alloc;

mod font;
/// Tools for laying out strings of text.
pub mod layout;
mod math;
mod platform;
mod raster;
mod table;
mod unicode;

pub use crate::font::*;

/// Alias for Result<T, &'static str>.
pub type FontResult<T> = Result<T, &'static str>;
