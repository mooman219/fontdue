use crate::math::Polygons;
use crate::simd::*;
use alloc::vec;
use alloc::vec::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

pub struct Raster {
    w: usize,
    h: usize,
    a: Vec<f32>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Format {
    Rgba32,
    Rgb24,
    A8,
}

#[inline(always)]
unsafe fn set_a8(output: &mut Vec<u8>, index: usize, value: u8) {
    *(output.get_unchecked_mut(index)) = value;
}


#[inline(always)]
unsafe fn set_rgb24(output: &mut Vec<u8>, index: usize, value: u8) {
    let start = index * 3;
    *(output.get_unchecked_mut(start)) = value;
    *(output.get_unchecked_mut(start + 1)) = value;
    *(output.get_unchecked_mut(start + 2)) = value;
}

#[inline(always)]
unsafe fn set_rgba32(output: &mut Vec<u8>, index: usize, value: u8) {
    let start = index * 4;
    *(output.get_unchecked_mut(start)) = value;
    *(output.get_unchecked_mut(start + 1)) = value;
    *(output.get_unchecked_mut(start + 2)) = value;
    *(output.get_unchecked_mut(start + 3)) = value;
}

#[inline(always)]
unsafe fn set_a8_sse_x86(output: &mut Vec<u8>, index: usize, value: __m128i) {
    // Store the first 4 u8s from y in output. The cast again is a nop.
    _mm_store_ss(core::mem::transmute(output.get_unchecked_mut(index)), _mm_castsi128_ps(value));
}

#[inline(always)]
unsafe fn set_rgb24_sse_x86(output: &mut Vec<u8>, index: usize, value: __m128i) {
    // Store the first 4 u8s from y in output. The cast again is a nop.
    // This stores the first 4u8s 4 times. We only need it stored 3 times, however we'll overwrite
    // the last excess write with the next write (if there is one)
    // This uses an unaligned store, since we're walking over 96 bits at a time
    // (which is not going to result in 16-byte aligned pointers)
    _mm_storeu_ps(core::mem::transmute(output.get_unchecked_mut(index * 3)), _mm_castsi128_ps(value));
}

#[inline(always)]
unsafe fn set_rgba32_sse_x86(output: &mut Vec<u8>, index: usize, value: __m128i) {
    // Store the first 4 u8s from y in output. The cast again is a nop.
    // This stores the first 4u8s 4 times, but we need the 4th value stored to be an alpha value.
    // Since we're using all the bytes in the lane, this is conviniently going to reuslt in a 16-byte
    // aligned address on every iteration
    _mm_store_ps(core::mem::transmute(output.get_unchecked_mut(index * 4)), _mm_castsi128_ps(value));
}

impl Raster {
    pub fn new(w: usize, h: usize) -> Raster {
        Raster {
            w,
            h,
            a: vec![0.0; w * h + 3],
        }
    }

    pub fn draw(&mut self, polygons: &Polygons, scale: f32) {
        let scale = f32x4::splat(scale);
        for line in &polygons.lines {
            let abcd = line.abcd * scale;
            self.line(abcd, line.x_mod, line.y_mod);
        }
    }

    #[inline(always)]
    fn add(&mut self, index: usize, height: f32, mid_x: f32) {
        unsafe {
            let mid_x = f32x4::single(mid_x).fract_first();
            *self.a.get_unchecked_mut(index) += height * (1.0 - mid_x);
            *self.a.get_unchecked_mut(index + 1) += height * mid_x;
        }
    }

    #[inline(always)]
    fn line(&mut self, pos: f32x4, x_mod: f32, y_mod: f32) {
        let &[x0, y0, x1, y1] = pos.borrowed();
        let dx = x1 - x0;
        let dy = y1 - y0;
        let sx = (1f32).copysign(dx);
        let sy = (1f32).copysign(dy);
        let mut x = x0.floor() + x_mod;
        let mut y = y0.floor() + y_mod;
        let tdx = if dx == 0.0 {
            1048576.0
        } else {
            1.0 / dx
        };
        let tdy = 1.0 / dy;
        let mut tmx = tdx * (x - x0);
        let mut tmy = tdy * (y - y0);
        let tdx = tdx.abs();
        let tdy = tdy.abs();

        let mut x_prev = x0;
        let mut y_prev = y0;
        let mut x_next: f32;
        let mut y_next: f32;
        let mut index = (x0 as usize + y0 as usize * self.w) as isize;
        let index_x_inc = sx as isize;
        let index_y_inc = (sy * self.w as f32) as isize;

        while tmx < 1.0 || tmy < 1.0 {
            let prev_index = index;
            if tmx < tmy {
                y_next = tmx * dy + y0;
                x_next = x;
                tmx += tdx;
                x += sx;
                index += index_x_inc;
            } else {
                y_next = y;
                x_next = tmy * dx + x0;
                tmy += tdy;
                y += sy;
                index += index_y_inc;
            }
            self.add(prev_index as usize, y_prev - y_next, (x_prev + x_next) / 2.0);
            x_prev = x_next;
            y_prev = y_next;
        }
        let index = (x1 as usize + y1 as usize * self.w) as usize;
        self.add(index, y_prev - y1, (x_prev + x1) / 2.0);
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn get_bitmap(&self, format: Format) -> Vec<u8> {
        let length = self.w * self.h;
        let mut height = 0.0;
        assert!(length <= self.a.len());
        let bufSize = match format {
            Format::A8 => length,
            Format::Rgb24 => length * 3,
            Format::Rgba32 => length * 4,
        };
        let mut output = vec![0; bufSize];
        let set = match format {
            Format::A8 => set_a8,
            Format::Rgb24 => set_rgb24,
            Format::Rgba32 => set_rgba32,
        };
        for i in 0..length {
            unsafe {
                height += self.a.get_unchecked(i);
                set(output, i, (height * 255.0) as u8);
            }
        }
        output
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn get_bitmap(&self, format: Format) -> Vec<u8> {
        let length = self.w * self.h;
        let width_multiplier = match format {
            Format::A8 => 1,
            Format::Rgb24 => 3,
            Format::Rgba32 => 4,
        };
        let aligned_length = (length + 3) & !3;
        let buf_size = aligned_length * width_multiplier;
        assert!(buf_size <= (self.a.len() * width_multiplier));
        // Turns out zeroing takes a while on very large sizes.
        let mut output = Vec::with_capacity(buf_size);
        let set = match format {
            Format::A8 => set_a8_sse_x86,
            Format::Rgb24 => set_rgb24_sse_x86,
            Format::Rgba32 => set_rgba32_sse_x86,
        };
        unsafe {
            output.set_len(buf_size);
            // offset = Zeroed out lanes
            let mut offset = _mm_setzero_ps();
            let lookup = match format {
                // lookup = The 4 bytes (12, 8, 4, 0) in all lanes
                Format::A8 => _mm_set1_epi32(0x0c_08_04_00),
                // lookup = for each 4-byte lane, replicate the first byte into the 2nd and 3rd bytes (making an rgb sequence),
                // and pack the equivalent from the next lane down - zero out the now unused last lane
                Format::Rgb24 => _mm_set_epi8(
                    -1, -1, -1, -1, 0xc, 0xc, 0xc, 0x8, 0x8, 0x8, 0x4, 0x4, 0x4, 0x0, 0x0, 0x0,
                ),
                // lookup = for each 4-byte lane, replicate the first byte into the 2nd, 3rd, and 4th bytes (making an rgba sequence),
                // This may not _strictly speaking_ be desired - the text wasn't really rasterized with an alpha channel in mind,
                // so using the character mask for the alpha channel _may_ wash out the result.
                Format::Rgba32 => _mm_set_epi8(
                    0xc, 0xc, 0xc, 0xc, 0x8, 0x8, 0x8, 0x8, 0x4, 0x4, 0x4, 0x4, 0x0, 0x0, 0x0, 0x0,
                ),
            };
            for i in (0..aligned_length).step_by(4) {
                // x = Read 4 floats from self.a
                let mut x = _mm_loadu_ps(self.a.get_unchecked(i));
                // x += Shift x register left by 32 bits (Padding with 0s). The casts are to
                // satisfy the type requirements, they are otherwise nops.
                x = _mm_add_ps(x, _mm_castsi128_ps(_mm_slli_si128(_mm_castps_si128(x), 4)));
                // x += (0.0, 0.0, x[0], x[2])
                x = _mm_add_ps(x, _mm_shuffle_ps(_mm_setzero_ps(), x, 0x40));
                // x += offset
                x = _mm_add_ps(x, offset);

                // y = x * 255.0
                let y = _mm_mul_ps(x, _mm_set1_ps(255.0));
                // y = Convert y to i32s and truncate
                let mut y = _mm_cvttps_epi32(y);
                // (SSSE3) y = Take the first byte of each of the 4 values in y and pack them into
                // the first 4 bytes of y. This produces the same value in all 4 lanes.
                y = _mm_shuffle_epi8(y, lookup);
                set(&mut output, i, y);
                // offset = (x[3], x[3], x[3], x[3])
                offset = _mm_shuffle_ps(x, x, 0b11_11_11_11);
            }
        }
        output.truncate(length * width_multiplier);
        output
    }
}
