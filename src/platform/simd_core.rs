#![allow(non_camel_case_types)]

use core::mem::transmute;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct f32x4 {
    x0: f32,
    x1: f32,
    x2: f32,
    x3: f32,
}

impl f32x4 {
    #[inline(always)]
    pub const fn new(x0: f32, x1: f32, x2: f32, x3: f32) -> Self {
        f32x4 {
            x0,
            x1,
            x2,
            x3,
        }
    }

    #[inline(always)]
    pub fn new_u32(x0: u32, x1: u32, x2: u32, x3: u32) -> Self {
        unsafe {
            Self::new(
                transmute::<u32, f32>(x0),
                transmute::<u32, f32>(x1),
                transmute::<u32, f32>(x2),
                transmute::<u32, f32>(x3),
            )
        }
    }

    #[inline(always)]
    pub fn sub_integer(&self, other: f32x4) -> f32x4 {
        unsafe {
            Self::new(
                transmute::<u32, f32>(transmute::<f32, u32>(self.x0) - transmute::<f32, u32>(other.x0)),
                transmute::<u32, f32>(transmute::<f32, u32>(self.x1) - transmute::<f32, u32>(other.x1)),
                transmute::<u32, f32>(transmute::<f32, u32>(self.x2) - transmute::<f32, u32>(other.x2)),
                transmute::<u32, f32>(transmute::<f32, u32>(self.x3) - transmute::<f32, u32>(other.x3)),
            )
        }
    }

    #[inline(always)]
    pub const fn splat(value: f32) -> Self {
        Self::new(value, value, value, value)
    }

    #[inline(always)]
    pub const fn zero() -> Self {
        Self::splat(0.0)
    }

    #[inline(always)]
    pub const fn copied(self) -> (f32, f32, f32, f32) {
        (self.x0, self.x1, self.x2, self.x3)
    }

    #[inline(always)]
    pub fn trunc(self) -> Self {
        use super::trunc;
        Self::new(trunc(self.x0), trunc(self.x1), trunc(self.x2), trunc(self.x3))
    }

    #[inline(always)]
    pub fn sqrt(self) -> Self {
        use super::sqrt;
        Self::new(sqrt(self.x0), sqrt(self.x1), sqrt(self.x2), sqrt(self.x3))
    }
}

impl Add for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn add(self, other: f32x4) -> f32x4 {
        Self::new(self.x0 + other.x0, self.x1 + other.x1, self.x2 + other.x2, self.x3 + other.x3)
    }
}

impl AddAssign for f32x4 {
    #[inline(always)]
    fn add_assign(&mut self, other: f32x4) {
        self.x0 += other.x0;
        self.x1 += other.x1;
        self.x2 += other.x2;
        self.x3 += other.x3;
    }
}

impl Sub for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn sub(self, other: f32x4) -> f32x4 {
        Self::new(self.x0 - other.x0, self.x1 - other.x1, self.x2 - other.x2, self.x3 - other.x3)
    }
}

impl SubAssign for f32x4 {
    #[inline(always)]
    fn sub_assign(&mut self, other: f32x4) {
        self.x0 -= other.x0;
        self.x1 -= other.x1;
        self.x2 -= other.x2;
        self.x3 -= other.x3;
    }
}

impl Mul for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn mul(self, other: f32x4) -> f32x4 {
        Self::new(self.x0 * other.x0, self.x1 * other.x1, self.x2 * other.x2, self.x3 * other.x3)
    }
}

impl MulAssign for f32x4 {
    #[inline(always)]
    fn mul_assign(&mut self, other: f32x4) {
        self.x0 *= other.x0;
        self.x1 *= other.x1;
        self.x2 *= other.x2;
        self.x3 *= other.x3;
    }
}

impl Div for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn div(self, other: f32x4) -> f32x4 {
        Self::new(self.x0 / other.x0, self.x1 / other.x1, self.x2 / other.x2, self.x3 / other.x3)
    }
}

impl DivAssign for f32x4 {
    #[inline(always)]
    fn div_assign(&mut self, other: f32x4) {
        self.x0 /= other.x0;
        self.x1 /= other.x1;
        self.x2 /= other.x2;
        self.x3 /= other.x3;
    }
}
