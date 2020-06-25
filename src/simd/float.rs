use core::mem::transmute;

// github.com/rust-lang/libm (MIT) Copyright (c) 2018 Jorge Aparicio
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

// github.com/rust-lang/libm (MIT) Copyright (c) 2018 Jorge Aparicio
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

/// Average Error 0.00224 / Largest Error 0.00468.
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
