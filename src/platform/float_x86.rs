#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::mem::transmute;

use crate::platform::{is_negative, is_positive};

#[inline(always)]
pub fn trunc(value: f32) -> f32 {
    unsafe { _mm_cvtss_f32(_mm_cvtepi32_ps(_mm_cvttps_epi32(_mm_set_ss(value)))) }
}

#[inline(always)]
pub fn as_i32(value: f32) -> i32 {
    unsafe { _mm_cvtss_si32(_mm_set_ss(value)) }
}

#[inline(always)]
pub fn fract(value: f32) -> f32 {
    unsafe {
        let packed = _mm_set_ss(value);
        _mm_cvtss_f32(_mm_sub_ps(packed, _mm_cvtepi32_ps(_mm_cvttps_epi32(packed))))
    }
}

#[inline(always)]
pub fn ceil(mut value: f32) -> f32 {
    unsafe {
        // The gist: add 1, subtract epsilon, then truncate. If negative, just truncate.
        if is_positive(value) {
            value = transmute::<u32, f32>(transmute::<f32, u32>(value + 1.0) - 1);
        }
        _mm_cvtss_f32(_mm_cvtepi32_ps(_mm_cvttps_epi32(_mm_set_ss(value))))
    }
}

#[inline(always)]
pub fn floor(mut value: f32) -> f32 {
    unsafe {
        // The gist: sub 1, sub epsilon, then truncate. If positive, just truncate.
        if is_negative(value) {
            value = transmute::<u32, f32>(transmute::<f32, u32>(value - 1.0) - 1);
        }
        _mm_cvtss_f32(_mm_cvtepi32_ps(_mm_cvttps_epi32(_mm_set_ss(value))))
    }
}
