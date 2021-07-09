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
/// The arctangent function.
pub fn atan2f(y: f32, x: f32) -> f32 {
    const PI: f32 = 3.1415927410e+00; /* 0x40490fdb */
    const PI_LO: f32 = -8.7422776573e-08; /* 0xb3bbbd2e */

    if x.is_nan() || y.is_nan() {
        return x + y;
    }
    let mut ix = x.to_bits();
    let mut iy = y.to_bits();

    if ix == 0x3f800000 {
        /* x=1.0 */
        return super::atan(y);
    }
    let m = ((iy >> 31) & 1) | ((ix >> 30) & 2); /* 2*sign(x)+sign(y) */
    ix &= 0x7fffffff;
    iy &= 0x7fffffff;

    /* when y = 0 */
    if iy == 0 {
        return match m {
            0 | 1 => y,   /* atan(+-0,+anything)=+-0 */
            2 => PI,      /* atan(+0,-anything) = pi */
            3 | _ => -PI, /* atan(-0,-anything) =-pi */
        };
    }
    /* when x = 0 */
    if ix == 0 {
        return if m & 1 != 0 {
            -PI / 2.
        } else {
            PI / 2.
        };
    }
    /* when x is INF */
    if ix == 0x7f800000 {
        return if iy == 0x7f800000 {
            match m {
                0 => PI / 4.,           /* atan(+INF,+INF) */
                1 => -PI / 4.,          /* atan(-INF,+INF) */
                2 => 3. * PI / 4.,      /* atan(+INF,-INF)*/
                3 | _ => -3. * PI / 4., /* atan(-INF,-INF)*/
            }
        } else {
            match m {
                0 => 0.,      /* atan(+...,+INF) */
                1 => -0.,     /* atan(-...,+INF) */
                2 => PI,      /* atan(+...,-INF) */
                3 | _ => -PI, /* atan(-...,-INF) */
            }
        };
    }
    /* |y/x| > 0x1p26 */
    if (ix + (26 << 23) < iy) || (iy == 0x7f800000) {
        return if m & 1 != 0 {
            -PI / 2.
        } else {
            PI / 2.
        };
    }

    /* z = atan(|y/x|) with correct underflow */
    let z = if (m & 2 != 0) && (iy + (26 << 23) < ix) {
        /*|y/x| < 0x1p-26, x < 0 */
        0.
    } else {
        super::atan(super::abs(y / x))
    };
    match m {
        0 => z,                /* atan(+,+) */
        1 => -z,               /* atan(-,+) */
        2 => PI - (z - PI_LO), /* atan(+,-) */
        _ => (z - PI_LO) - PI, /* case 3 */ /* atan(-,-) */
    }
}

pub fn atan2(x: f32, y: f32) -> f32 {
    use core::f32::consts::PI;
    const PI_2: f32 = PI / 2.0;
    const M_PI_2: f32 = -PI / 2.0;
    const PI_4: f32 = PI / 4.0;
    const PI_3_4: f32 = PI_4 * 3.0;
    let mut r;
    let c;
    let abs_y = super::abs(y);
    if x == 0.0 {
        if y > 0.0 {
            return PI_2;
        }
        if y == 0.0 {
            return 0.0;
        }
        return M_PI_2;
    } else if x > 0.0 {
        r = (x - abs_y) / (x + abs_y);
        c = PI_4;
    } else {
        r = (x + abs_y) / (abs_y - x);
        c = PI_3_4;
    }
    r = 0.1963 * r * r * r - 0.9817 * r + c;
    return super::copysign(r, y);
}
