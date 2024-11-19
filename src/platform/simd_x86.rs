#![allow(non_camel_case_types)]

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct f32x4(__m128);

impl f32x4 {
    #[inline(always)]
    pub fn new(x0: f32, x1: f32, x2: f32, x3: f32) -> Self {
        f32x4(unsafe { _mm_set_ps(x3, x2, x1, x0) })
    }

    #[inline(always)]
    pub fn new_u32(x0: u32, x1: u32, x2: u32, x3: u32) -> Self {
        f32x4(unsafe {
            _mm_set_ps(
                core::mem::transmute::<u32, f32>(x3),
                core::mem::transmute::<u32, f32>(x2),
                core::mem::transmute::<u32, f32>(x1),
                core::mem::transmute::<u32, f32>(x0),
            )
        })
    }

    #[inline(always)]
    pub fn splat(value: f32) -> Self {
        f32x4(unsafe { _mm_set1_ps(value) })
    }

    pub fn sub_integer(&self, other: f32x4) -> f32x4 {
        f32x4(unsafe { _mm_castsi128_ps(_mm_sub_epi32(_mm_castps_si128(self.0), _mm_castps_si128(other.0))) })
    }

    #[inline(always)]
    pub fn zero() -> Self {
        f32x4(unsafe { _mm_setzero_ps() })
    }

    #[inline(always)]
    pub fn copied(self) -> (f32, f32, f32, f32) {
        unsafe { core::mem::transmute::<__m128, (f32, f32, f32, f32)>(self.0) }
    }

    #[inline(always)]
    pub fn trunc(self) -> Self {
        unsafe { f32x4(_mm_cvtepi32_ps(_mm_cvttps_epi32(self.0))) }
    }

    #[inline(always)]
    pub fn sqrt(self) -> Self {
        unsafe { f32x4(_mm_sqrt_ps(self.0)) }
    }
}

impl Add for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn add(self, other: f32x4) -> f32x4 {
        unsafe { f32x4(_mm_add_ps(self.0, other.0)) }
    }
}

impl AddAssign for f32x4 {
    #[inline(always)]
    fn add_assign(&mut self, other: f32x4) {
        self.0 = unsafe { _mm_add_ps(self.0, other.0) };
    }
}

impl Sub for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn sub(self, other: f32x4) -> f32x4 {
        unsafe { f32x4(_mm_sub_ps(self.0, other.0)) }
    }
}

impl SubAssign for f32x4 {
    #[inline(always)]
    fn sub_assign(&mut self, other: f32x4) {
        self.0 = unsafe { _mm_sub_ps(self.0, other.0) };
    }
}

impl Mul for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn mul(self, other: f32x4) -> f32x4 {
        unsafe { f32x4(_mm_mul_ps(self.0, other.0)) }
    }
}

impl MulAssign for f32x4 {
    #[inline(always)]
    fn mul_assign(&mut self, other: f32x4) {
        self.0 = unsafe { _mm_mul_ps(self.0, other.0) };
    }
}

impl Div for f32x4 {
    type Output = f32x4;
    #[inline(always)]
    fn div(self, other: f32x4) -> f32x4 {
        unsafe { f32x4(_mm_div_ps(self.0, other.0)) }
    }
}

impl DivAssign for f32x4 {
    #[inline(always)]
    fn div_assign(&mut self, other: f32x4) {
        self.0 = unsafe { _mm_div_ps(self.0, other.0) };
    }
}
