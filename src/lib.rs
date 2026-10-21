//! Fontdue is a font parser, rasterizer, and layout tool.
//!
//! This is a no_std crate, but still requires the alloc crate.

#![cfg_attr(all(not(test), not(feature = "std"), feature = "hashbrown"), no_std)]
#![allow(dead_code)]
#![allow(clippy::style)]
#![allow(clippy::complexity)]
#![allow(clippy::misnamed_getters)]

extern crate alloc;

mod font;
mod hash;
/// Tools for laying out strings of text.
pub mod layout;
mod math;
mod platform;
mod raster;
mod table;
mod unicode;

pub use crate::font::*;
use thiserror::Error;
use ttf_parser::FaceParsingError;

#[cfg(feature = "hashbrown")]
pub(crate) use hashbrown::{HashMap, HashSet};
#[cfg(not(feature = "hashbrown"))]
pub(crate) use std::collections::{HashMap, HashSet};

/// Alias for Result<T, FontError>.
pub type FontResult<T> = Result<T, FontError>;

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum FontError {
    #[error("An attempt to read out of bounds detected.")]
    MalformedFont,
    #[error("Face data must start with 0x00010000, 0x74727565, 0x4F54544F or 0x74746366.")]
    UnknownMagic,
    #[error("The face index is larger than the number of faces in the font.")]
    FaceIndexOutOfBounds,
    #[error("The head table is missing or malformed.")]
    NoHeadTable,
    #[error("The hhea table is missing or malformed.")]
    NoHheaTable,
    #[error("The maxp table is missing or malformed.")]
    NoMaxpTable,
    #[error("Attempted to map a codepoint out of bounds.")]
    CodepointOutOfBounds,
}

impl From<FaceParsingError> for FontError {
    fn from(error: FaceParsingError) -> FontError {
        match error {
            FaceParsingError::MalformedFont => FontError::MalformedFont,
            FaceParsingError::UnknownMagic => FontError::UnknownMagic,
            FaceParsingError::FaceIndexOutOfBounds => FontError::FaceIndexOutOfBounds,
            FaceParsingError::NoHeadTable => FontError::NoHeadTable,
            FaceParsingError::NoHheaTable => FontError::NoHeadTable,
            FaceParsingError::NoMaxpTable => FontError::NoMaxpTable,
        }
    }
}
