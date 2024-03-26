mod as_i32;
mod atan;
mod atan2;
mod ceil;
mod floor;
mod fract;
mod get_bitmap;
mod sqrt;
mod trunc;

pub use as_i32::*;
pub use atan::*;
pub use atan2::*;
pub use ceil::*;
pub use floor::*;
pub use fract::*;
pub use get_bitmap::*;
pub use sqrt::*;
#[allow(unused_imports)]
pub use trunc::*;

/// Sets the high bit 0x80000000 on a float.
#[inline(always)]
pub fn abs(value: f32) -> f32 {
    f32::from_bits(value.to_bits() & 0x7fffffff)
}

/// Checks if the high bit 0x80000000 is set on a float.
#[inline(always)]
pub fn is_negative(value: f32) -> bool {
    value.to_bits() >= 0x80000000
}

/// Checks if the high bit 0x80000000 is not set on a float.
#[inline(always)]
pub fn is_positive(value: f32) -> bool {
    value.to_bits() < 0x80000000
}

/// Inverts the high bit 0x80000000 on a float.
#[inline(always)]
pub fn flipsign(value: f32) -> f32 {
    f32::from_bits(value.to_bits() ^ 0x80000000)
}

/// Assigns the high bit 0x80000000 on the sign to the value.
#[inline(always)]
pub fn copysign(value: f32, sign: f32) -> f32 {
    f32::from_bits((value.to_bits() & 0x7fffffff) | (sign.to_bits() & 0x80000000))
}

#[inline(always)]
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    let mut x = value;
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}
