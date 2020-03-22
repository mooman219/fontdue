#![allow(non_camel_case_types)]

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[inline(always)]
pub fn fraction(value: f32) -> f32 {
    value.fract()
}

#[inline(always)]
pub fn truncate(value: f32) -> f32 {
    value.trunc()
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct f32x4([f32; 4]);

impl f32x4 {
    #[inline(always)]
    pub fn new(x0: f32, x1: f32, x2: f32, x3: f32) -> Self {
        f32x4([x0, x1, x2, x3])
    }

    #[inline(always)]
    pub fn splat(value: f32) -> Self {
        f32x4([value, value, value, value])
    }

    #[inline(always)]
    pub fn zero() -> Self {
        f32x4([0.0, 0.0, 0.0, 0.0])
    }

    #[inline(always)]
    pub fn copied(self) -> [f32; 4] {
        self.0
    }

    #[inline(always)]
    pub fn borrowed(&self) -> &[f32; 4] {
        &self.0
    }

    #[inline(always)]
    pub fn trunc(self) -> Self {
        f32x4([self.0[0].trunc(), self.0[1].trunc(), self.0[2].trunc(), self.0[3].trunc()])
    }

    #[inline(always)]
    pub fn fract(self) -> Self {
        f32x4([self.0[0].fract(), self.0[1].fract(), self.0[2].fract(), self.0[3].fract()])
    }
}

impl Add for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn add(self, other: f32x4) -> f32x4 {
        f32x4([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
            self.0[3] + other.0[3],
        ])
    }
}

impl AddAssign for f32x4 {
    #[inline(always)]
    fn add_assign(&mut self, other: f32x4) {
        self.0[0] += other.0[0];
        self.0[1] += other.0[1];
        self.0[2] += other.0[2];
        self.0[3] += other.0[3];
    }
}

impl Sub for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn sub(self, other: f32x4) -> f32x4 {
        f32x4([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
            self.0[3] - other.0[3],
        ])
    }
}

impl SubAssign for f32x4 {
    #[inline(always)]
    fn sub_assign(&mut self, other: f32x4) {
        self.0[0] -= other.0[0];
        self.0[1] -= other.0[1];
        self.0[2] -= other.0[2];
        self.0[3] -= other.0[3];
    }
}

impl Mul for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn mul(self, other: f32x4) -> f32x4 {
        f32x4([
            self.0[0] * other.0[0],
            self.0[1] * other.0[1],
            self.0[2] * other.0[2],
            self.0[3] * other.0[3],
        ])
    }
}

impl MulAssign for f32x4 {
    #[inline(always)]
    fn mul_assign(&mut self, other: f32x4) {
        self.0[0] *= other.0[0];
        self.0[1] *= other.0[1];
        self.0[2] *= other.0[2];
        self.0[3] *= other.0[3];
    }
}

impl Div for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn div(self, other: f32x4) -> f32x4 {
        f32x4([
            self.0[0] / other.0[0],
            self.0[1] / other.0[1],
            self.0[2] / other.0[2],
            self.0[3] / other.0[3],
        ])
    }
}

impl DivAssign for f32x4 {
    #[inline(always)]
    fn div_assign(&mut self, other: f32x4) {
        self.0[0] /= other.0[0];
        self.0[1] /= other.0[1];
        self.0[2] /= other.0[2];
        self.0[3] /= other.0[3];
    }
}
