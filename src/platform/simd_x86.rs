#![allow(non_camel_case_types)]

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(feature = "serde_derive")]
use {
    core::fmt,
    serde::{
        de,
        de::{Deserializer, SeqAccess, Visitor},
        ser::{SerializeTupleStruct as _, Serializer},
        Deserialize, Serialize,
    },
};

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
                core::mem::transmute(x3),
                core::mem::transmute(x2),
                core::mem::transmute(x1),
                core::mem::transmute(x0),
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
        unsafe { core::mem::transmute(self.0) }
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

#[cfg(feature = "serde_derive")]
impl Serialize for f32x4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (x0, x1, x2, x3) = self.copied();

        let mut state = serializer.serialize_tuple_struct("f32x4", 4)?;
        state.serialize_field(&x0)?;
        state.serialize_field(&x1)?;
        state.serialize_field(&x2)?;
        state.serialize_field(&x3)?;
        state.end()
    }
}

#[cfg(feature = "serde_derive")]
impl<'de> Deserialize<'de> for f32x4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct f32x4Visitor;

        impl<'de> Visitor<'de> for f32x4Visitor {
            type Value = f32x4;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(concat!("struct ", "f32x4"))
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<f32x4, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let x0 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let x1 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let x2 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
                let x3 = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?;
                Ok(f32x4::new(x0, x1, x2, x3))
            }
        }

        deserializer.deserialize_tuple_struct("f32x4", 4, f32x4Visitor)
    }
}

#[cfg(feature = "serde_derive")]
mod tests {
    #[allow(unused_imports)]
    use {super::f32x4, alloc::format};

    const DEBUG: &str = "f32x4 { x0: 1, x1: 2, x2: 3, x3: 4 }";
    const SX0: &str = "[]";
    const SX1: &str = "[1.0]";
    const SX2: &str = "[1.0,2.0]";
    const SX3: &str = "[1.0,2.0,3.0]";
    const SX4: &str = "[1.0,2.0,3.0,4.0]";
    const SX5: &str = "[1.0,2.0,3.0,4.0,5.0]";

    #[test]
    fn test_f32x4_debug() {
        let a = f32x4::new(1f32, 2f32, 3f32, 4f32);
        let debug = format!("{:?}", &a);
        assert_eq!(DEBUG, debug);
    }

    #[test]
    fn test_f32x4_serde() {
        let a = f32x4::new(1f32, 2f32, 3f32, 4f32);
        let serialized = serde_json::to_string(&a).unwrap();
        assert_eq!(SX4, serialized);

        let deserialized = serde_json::from_str(&serialized).unwrap();
        assert_eq!(a, deserialized);

        let deserialized = serde_json::from_str::<f32x4>(SX0);
        assert!(deserialized.is_err());

        let deserialized = serde_json::from_str::<f32x4>(SX1);
        assert!(deserialized.is_err());

        let deserialized = serde_json::from_str::<f32x4>(SX2);
        assert!(deserialized.is_err());

        let deserialized = serde_json::from_str::<f32x4>(SX3);
        assert!(deserialized.is_err());

        let deserialized = serde_json::from_str::<f32x4>(SX5);
        assert!(deserialized.is_err());
    }
}
