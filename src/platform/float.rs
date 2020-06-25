use core::mem::transmute;

// [See license/libm] Copyright (c) 2018 Jorge Aparicio
pub fn ceil(x: f32) -> f32 {
    let mut ui = x.to_bits();
    let e = (((ui >> 23) & 0xff).wrapping_sub(0x7f)) as i32;
    if e >= 23 {
        return x;
    }
    if e >= 0 {
        let m = 0x007fffff >> e;
        if (ui & m) == 0 {
            return x;
        }
        if ui >> 31 == 0 {
            ui += m;
        }
        ui &= !m;
    } else {
        if ui >> 31 != 0 {
            return -0.0;
        } else if ui << 1 != 0 {
            return 1.0;
        }
    }
    f32::from_bits(ui)
}

// [See license/libm] Copyright (c) 2018 Jorge Aparicio
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

// [See license/libm] Copyright (c) 2018 Jorge Aparicio
pub fn trunc(x: f32) -> f32 {
    let mut i: u32 = x.to_bits();
    let mut e: i32 = (i >> 23 & 0xff) as i32 - 0x7f + 9;
    let m: u32;
    if e >= 23 + 9 {
        return x;
    }
    if e < 9 {
        e = 1;
    }
    m = -1i32 as u32 >> e;
    if (i & m) == 0 {
        return x;
    }
    i &= !m;
    f32::from_bits(i)
}

#[inline(always)]
pub fn abs(value: f32) -> f32 {
    unsafe { transmute::<u32, f32>(transmute::<f32, u32>(value) & 0x7fffffff) }
}

#[inline(always)]
pub fn copysign(value: f32, sign: f32) -> f32 {
    unsafe {
        transmute::<u32, f32>(
            (transmute::<f32, u32>(value) & 0x7fffffff) | (transmute::<f32, u32>(sign) & 0x80000000),
        )
    }
}

/// A fast approximate atan2 function. It has an average error of 0.00224 radians, with the largest
/// error being 0.00468 radians. This converts to a largest error of 0.2681442481 degrees, which is
/// acceptable for where I use atan2 in fontdue. There are faster methods that use a LUT, but cache
/// thrashing is bad imo and LUTs are bad to micro-benchmark.
pub fn atan2(y: f32, x: f32) -> f32 {
    const PI: f32 = 3.1415927;
    const PI_2: f32 = PI / 2.0;
    const MINUS_PI_2: f32 = -PI_2;

    if x == 0.0 {
        if y > 0.0 {
            return PI_2;
        }
        if y == 0.0 {
            return 0.0;
        }
        return MINUS_PI_2;
    }

    let atan;
    let z = y / x;
    if abs(z) < 1.0 {
        atan = z / (1.0 + 0.28 * z * z);
        if x < 0.0 {
            if y < 0.0 {
                return atan - PI;
            } else {
                return atan + PI;
            }
        }
        return atan;
    } else {
        atan = PI_2 - z / (z * z + 0.28);
        if y < 0.0 {
            return atan - PI;
        } else {
            return atan;
        }
    }
}
