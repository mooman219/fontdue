#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd")))]
mod simd_core;
#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd")))]
pub use simd_core::*;

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd"))]
mod simd_x86;
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd"))]
pub use simd_x86::*;

mod float;
pub use float::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_ceil_test() {
        use core::mem::transmute;
        let mut y = 3.0;
        while y < 9.0 {
            assert_eq!(ceil(y), f32::ceil(y));
            y = unsafe { transmute::<u32, f32>(transmute::<f32, u32>(y) + 1) };
        }

        assert_eq!(ceil(-1.5), -1.0);
        assert_eq!(ceil(-1.0), -1.0);
        assert_eq!(ceil(-0.5), 0.0);
        assert_eq!(ceil(0.0), 0.0);
        assert_eq!(ceil(0.5), 1.0);
        assert_eq!(ceil(1.0), 1.0);
        assert_eq!(ceil(1.5), 2.0);
    }

    #[test]
    fn platform_floor_test() {
        use core::mem::transmute;
        let mut y = -3.0;
        while y > -9.0 {
            assert_eq!(ceil(y), f32::ceil(y));
            y = unsafe { transmute::<u32, f32>(transmute::<f32, u32>(y) + 1) };
        }

        assert_eq!(floor(-1.5), -2.0);
        assert_eq!(floor(-1.0), -1.0);
        assert_eq!(floor(-0.5), -1.0);
        assert_eq!(floor(0.0), 0.0);
        assert_eq!(floor(0.5), 0.0);
        assert_eq!(floor(1.0), 1.0);
        assert_eq!(floor(1.5), 1.0);
    }

    #[test]
    fn platform_fract_test() {
        assert_eq!(fract(-1.5), -0.5);
        assert_eq!(fract(-1.0), 0.0);
        assert_eq!(fract(-0.5), -0.5);
        assert_eq!(fract(0.0), 0.0);
        assert_eq!(fract(0.5), 0.5);
        assert_eq!(fract(1.0), 0.0);
        assert_eq!(fract(1.5), 0.5);
    }

    #[test]
    fn platform_trunc_test() {
        assert_eq!(trunc(-1.5), -1.0);
        assert_eq!(trunc(-1.0), -1.0);
        assert_eq!(trunc(-0.5), 0.0);
        assert_eq!(trunc(0.0), 0.0);
        assert_eq!(trunc(0.5), 0.0);
        assert_eq!(trunc(1.0), 1.0);
        assert_eq!(trunc(1.5), 1.0);
    }
}
