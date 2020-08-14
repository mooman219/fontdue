// [See license/rust-lang/libm] Copyright (c) 2018 Jorge Aparicio
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
pub fn as_i32(value: f32) -> i32 {
    value as i32
}

#[inline(always)]
pub fn fract(value: f32) -> f32 {
    value - trunc(value)
}

// [See license/rust-lang/libm] Copyright (c) 2018 Jorge Aparicio
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
