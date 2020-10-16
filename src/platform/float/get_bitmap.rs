use alloc::vec::*;
#[cfg(all(target_arch = "x86", feature = "simd"))]
use core::arch::x86::*;
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
use core::arch::x86_64::*;

#[cfg(any(not(any(target_arch = "x86", target_arch = "x86_64")), not(feature = "simd")))]
pub fn get_bitmap(a: &Vec<f32>, length: usize) -> Vec<u8> {
    use alloc::vec;
    let mut height = 0.0;
    assert!(length <= a.len());
    let mut output = vec![0; length];
    for i in 0..length {
        unsafe {
            height += a.get_unchecked(i);
            *(output.get_unchecked_mut(i)) = ((height) * 255.9) as u8;
        }
    }
    output
}

#[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd"))]
pub fn get_bitmap(a: &Vec<f32>, length: usize) -> Vec<u8> {
    let aligned_length = (length + 3) & !3;
    assert!(aligned_length <= a.len());
    // Turns out zeroing takes a while on very large sizes.
    let mut output = Vec::with_capacity(aligned_length);
    unsafe {
        output.set_len(aligned_length);
        // offset = Zeroed out lanes
        let mut offset = _mm_setzero_ps();
        let zero = _mm_castps_si128(_mm_setzero_ps());
        for i in (0..aligned_length).step_by(4) {
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
            // y = Convert y to i32s and truncate
            let mut y = _mm_cvttps_epi32(y);
            // y = Take the first byte of each of the 4 values in y and pack them into
            // the first 4 bytes of y.
            y = _mm_packus_epi16(_mm_packs_epi32(y, zero), zero);

            // Store the first 4 u8s from y in output.
            let pointer: &mut i32 = core::mem::transmute(output.get_unchecked_mut(i));
            *pointer = core::mem::transmute::<__m128i, [i32; 4]>(y)[0];
            // offset = (x[3], x[3], x[3], x[3])
            offset = _mm_set1_ps(core::mem::transmute::<__m128, [f32; 4]>(x)[3]);
        }
    }
    output.truncate(length);
    output
}

// AVX is just slower, likely a bad impl.
// #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), feature = "simd"))]
// pub fn get_bitmap(a: &Vec<f32>, length: usize) -> Vec<u8> {
//     let aligned_length = (length + 7) & !7;
//     assert!(aligned_length <= a.len());
//     // Turns out zeroing takes a while on very large sizes.
//     let mut output = Vec::with_capacity(aligned_length);
//     unsafe {
//         output.set_len(aligned_length);
//         // offset = Zeroed out lanes
//         let mut offset = _mm256_setzero_ps();
//         let zero = _mm256_castps_si256(_mm256_setzero_ps());
//         for i in (0..aligned_length).step_by(8) {
//             let mut x = _mm256_loadu_ps(a.get_unchecked(i));

//             x = _mm256_add_ps(x, _mm256_castsi256_ps(_mm256_slli_si256(_mm256_castps_si256(x), 4)));
//             x = _mm256_add_ps(x, _mm256_castsi256_ps(_mm256_slli_si256(_mm256_castps_si256(x), 8)));
//             x = _mm256_add_ps(x, _mm256_castsi256_ps(_mm256_slli_si256(_mm256_castps_si256(x), 16)));
//             x = _mm256_add_ps(x, offset);

//             let y = _mm256_mul_ps(x, _mm256_set1_ps(255.9));
//             let mut y = _mm256_cvttps_epi32(y);
//             y = _mm256_packus_epi16(_mm256_packs_epi32(y, zero), zero);

//             let pointer: &mut f32 = core::mem::transmute(output.get_unchecked_mut(i));
//             *pointer = core::mem::transmute::<__m256i, [f32; 8]>(y)[0];
//             offset = _mm256_set1_ps(core::mem::transmute::<__m256, [f32; 8]>(x)[7]);
//         }
//         output.truncate(length);
//         output
//     }
// }
