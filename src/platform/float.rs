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

/*
 * origin: FreeBSD /usr/src/lib/msun/src/s_atanf.c
 *
 * Conversion to float by Ian Lance Taylor, Cygnus Support, ian@cygnus.com.
 *
 * ====================================================
 * Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
 *
 * Developed at SunPro, a Sun Microsystems, Inc. business.
 * Permission to use, copy, modify, and distribute this
 * software is freely granted, provided that this notice
 * is preserved.
 * ====================================================
 */
pub fn atan(mut x: f32) -> f32 {
    const ATAN_HI: [f32; 4] = [
        4.6364760399e-01, /* atan(0.5)hi 0x3eed6338 */
        7.8539812565e-01, /* atan(1.0)hi 0x3f490fda */
        9.8279368877e-01, /* atan(1.5)hi 0x3f7b985e */
        1.5707962513e+00, /* atan(inf)hi 0x3fc90fda */
    ];

    const ATAN_LO: [f32; 4] = [
        5.0121582440e-09, /* atan(0.5)lo 0x31ac3769 */
        3.7748947079e-08, /* atan(1.0)lo 0x33222168 */
        3.4473217170e-08, /* atan(1.5)lo 0x33140fb4 */
        7.5497894159e-08, /* atan(inf)lo 0x33a22168 */
    ];

    const A_T: [f32; 5] =
        [3.3333328366e-01, -1.9999158382e-01, 1.4253635705e-01, -1.0648017377e-01, 6.1687607318e-02];
    let x1p_120 = f32::from_bits(0x03800000); // 0x1p-120 === 2 ^ (-120)

    let z: f32;

    let mut ix = x.to_bits();
    let sign = (ix >> 31) != 0;
    ix &= 0x7fffffff;

    if ix >= 0x4c800000 {
        /* if |x| >= 2**26 */
        if x.is_nan() {
            return x;
        }
        z = ATAN_HI[3] + x1p_120;
        return if sign {
            -z
        } else {
            z
        };
    }
    let id = if ix < 0x3ee00000 {
        /* |x| < 0.4375 */
        if ix < 0x39800000 {
            return x;
        }
        -1
    } else {
        x = abs(x);
        if ix < 0x3f980000 {
            /* |x| < 1.1875 */
            if ix < 0x3f300000 {
                /*  7/16 <= |x| < 11/16 */
                x = (2. * x - 1.) / (2. + x);
                0
            } else {
                /* 11/16 <= |x| < 19/16 */
                x = (x - 1.) / (x + 1.);
                1
            }
        } else if ix < 0x401c0000 {
            /* |x| < 2.4375 */
            x = (x - 1.5) / (1. + 1.5 * x);
            2
        } else {
            /* 2.4375 <= |x| < 2**26 */
            x = -1. / x;
            3
        }
    };
    /* end of argument reduction */
    z = x * x;
    let w = z * z;
    /* break sum from i=0 to 10 aT[i]z**(i+1) into odd and even poly */
    let s1 = z * (A_T[0] + w * (A_T[2] + w * A_T[4]));
    let s2 = w * (A_T[1] + w * A_T[3]);
    if id < 0 {
        return x - x * (s1 + s2);
    }
    let id = id as usize;
    let z = ATAN_HI[id] - ((x * (s1 + s2) - ATAN_LO[id]) - x);
    if sign {
        -z
    } else {
        z
    }
}
