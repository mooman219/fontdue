use crate::platform::trunc;

#[inline(always)]
pub fn fract(value: f32) -> f32 {
    value - trunc(value)
}
