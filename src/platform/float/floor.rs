#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
// [See license/rust-lang/libm] Copyright (c) 2018 Jorge Aparicio
pub fn floor(x: f32) -> f32 {
    let mut ui = x.to_bits();
    let e = (((ui >> 23) as i32) & 0xff) - 0x7f;

    if e >= 23 {
        return x;
    }
    if e >= 0 {
        let m: u32 = 0x007fffff >> e;
        if (ui & m) == 0 {
            return x;
        }
        if ui >> 31 != 0 {
            ui += m;
        }
        ui &= !m;
    } else {
        if ui >> 31 == 0 {
            ui = 0;
        } else if ui << 1 != 0 {
            return -1.0;
        }
    }
    f32::from_bits(ui)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
pub fn floor(mut value: f32) -> f32 {
    use crate::platform::is_negative;
    use core::mem::transmute;

    unsafe {
        // The gist: sub 1, sub epsilon, then truncate. If positive, just truncate.
        if is_negative(value) {
            value = transmute::<u32, f32>(transmute::<f32, u32>(value - 1.0) - 1);
        }
        _mm_cvtss_f32(_mm_cvtepi32_ps(_mm_cvttps_epi32(_mm_set_ss(value))))
    }
}
