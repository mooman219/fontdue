#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
mod simd_core;
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use simd_core::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod simd_x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use simd_x86::*;

mod float;
pub use float::*;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
mod float_core;
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub use float_core::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod float_x86;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use float_x86::*;
