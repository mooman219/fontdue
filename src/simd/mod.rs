#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
mod simd_none;
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use simd_none::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod simd_x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use simd_x86::*;
