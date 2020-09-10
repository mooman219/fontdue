#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline(always)]
pub fn as_i32(value: f32) -> i32 {
    value as i32
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn as_i32(value: f32) -> i32 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    unsafe { _mm_cvtss_si32(_mm_set_ss(value)) }
}
