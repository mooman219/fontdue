use alloc::vec::*;

#[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd")))]
pub fn get_bitmap(a: &Vec<f32>, length: usize) -> Vec<u8> {
    use crate::platform::{abs, clamp};
    use alloc::vec;
    let mut height = 0.0;
    assert!(length <= a.len());
    let mut output = vec![0; length];
    for i in 0..length {
        unsafe {
            height += a.get_unchecked(i);
            // Clamping because as u8 is undefined outside of its range in rustc.
            *(output.get_unchecked_mut(i)) = clamp(abs(height) * 255.9, 0.0, 255.0) as u8;
        }
    }
    output
}

#[allow(clippy::uninit_vec)]
#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd"))]
pub fn get_bitmap(a: &Vec<f32>, length: usize) -> Vec<u8> {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    unsafe {
        // Allocate a 4 byte aligned vector of bytes, and skip zeroing it. Turns out zeroing takes a
        // while on very large sizes.
        let mut output = {
            // Aligned length is ceil(length / 4).
            let aligned_length = (length + 3) >> 2;
            let mut aligned: Vec<u32> = Vec::with_capacity(aligned_length);
            let ptr = aligned.as_mut_ptr();
            let cap = aligned.capacity() << 2;
            core::mem::forget(aligned);
            Vec::from_raw_parts(ptr as *mut u8, aligned_length << 2, cap)
        };
        // offset = Zeroed out lanes
        let mut offset = _mm_setzero_ps();
        // Negative zero is important here.
        let nzero = _mm_castps_si128(_mm_set1_ps(-0.0));
        for i in (0..output.len()).step_by(4) {
            // x = Read 4 floats from self.a
            let mut x = _mm_loadu_ps(a.get_unchecked(i));
            // x += (0.0, x[0], x[1], x[2])
            x = _mm_add_ps(x, _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(x), 4)));
            // x += (0.0, 0.0, x[0], x[1])
            x = _mm_add_ps(x, _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(x), 8)));
            // x += offset
            x = _mm_add_ps(x, offset);

            // y = x * 255.9
            let y = _mm_mul_ps(x, _mm_set1_ps(255.9));
            // y = abs(y)
            let y = _mm_andnot_ps(_mm_castsi128_ps(nzero), y);
            // y = Convert y to i32s and truncate
            let mut y = _mm_cvttps_epi32(y);
            // y = Take the first byte of each of the 4 values in y and pack them into
            // the first 4 bytes of y.
            y = _mm_packus_epi16(_mm_packs_epi32(y, nzero), nzero);

            // Store the first 4 u8s from y in output.
            let pointer: &mut i32 = core::mem::transmute::<&mut u8, &mut i32>(output.get_unchecked_mut(i));
            *pointer = core::mem::transmute::<__m128i, [i32; 4]>(y)[0];
            // offset = (x[3], x[3], x[3], x[3])
            offset = _mm_set1_ps(core::mem::transmute::<__m128, [f32; 4]>(x)[3]);
        }
        output.truncate(length);
        output
    }
}
