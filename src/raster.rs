use crate::math::{Geometry, Line};
use crate::platform::{abs, copysign, f32x4, fract};
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

impl Raster {
    pub fn new(w: usize, h: usize) -> Raster {
        Raster {
            w,
            h,
            a: vec![0.0; w * h + 3],
        }
    }

    pub fn draw(&mut self, geometry: &Geometry, scale: f32, offset_x: f32, offset_y: f32) {
        let scale = f32x4::splat(scale);
        let offset = f32x4::new(offset_x, offset_y, offset_x, offset_y);
        for line in &geometry.lines {
            self.line(line, line.coords * scale + offset);
        }
    }

    #[inline(always)]
    fn add(&mut self, index: usize, height: f32, mid_x: f32) {
        // This is fast and hip.
        unsafe {
            let mid_x = fract(mid_x);
            *self.a.get_unchecked_mut(index) += height * (1.0 - mid_x);
            *self.a.get_unchecked_mut(index + 1) += height * mid_x;
        }

        // This is safe but slow.
        // let mid_x = fract(mid_x);
        // self.a[index] += height * (1.0 - mid_x);
        // self.a[index + 1] += height * mid_x;
    }

    #[inline(always)]
    fn line(&mut self, line: &Line, coords: f32x4) {
        let (x0, y0, x1, y1) = coords.copied();
        let (start_x, start_y, end_x, end_y) = coords.sub_integer(line.nudge).trunc().copied();
        let (mut target_x, mut target_y, _, _) =
            (coords + line.adjustment).sub_integer(line.nudge).trunc().copied();
        let dx = x1 - x0;
        let dy = y1 - y0;
        let sx = copysign(1f32, dx);
        let sy = copysign(1f32, dy);
        let tdx = if dx == 0.0 {
            1048576.0 // 2^20, doesn't really matter.
        } else {
            1.0 / dx
        };
        let tdy = 1.0 / dy;
        let mut tmx = tdx * (target_x - x0);
        let mut tmy = tdy * (target_y - y0);
        let tdx = abs(tdx);
        let tdy = abs(tdy);
        let mut x_prev = x0;
        let mut y_prev = y0;
        let mut index = (start_x + start_y * self.w as f32) as isize;
        let index_x_inc = sx as isize;
        let index_y_inc = copysign(self.w as f32, sy) as isize;
        // The (tmx < 1.0 || tmy < 1.0) condition does not work due to rounding errors in f32, so
        // dist is used instead to cap the iteration count.
        let mut dist = (abs(start_x - end_x) + abs(start_y - end_y)) as u32;
        while dist > 0 {
            dist -= 1;
            let prev_index = index;
            let y_next: f32;
            let x_next: f32;
            if tmx < tmy {
                y_next = tmx * dy + y0;
                x_next = target_x;
                tmx += tdx;
                target_x += sx;
                index += index_x_inc;
            } else {
                y_next = target_y;
                x_next = tmy * dx + x0;
                tmy += tdy;
                target_y += sy;
                index += index_y_inc;
            }
            self.add(prev_index as usize, y_prev - y_next, (x_prev + x_next) / 2.0);
            x_prev = x_next;
            y_prev = y_next;
        }
        self.add((end_x + end_y * self.w as f32) as usize, y_prev - y1, (x_prev + x1) / 2.0);
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn get_bitmap(&self) -> Vec<u8> {
        let length = self.w * self.h;
        let mut height = 0.0;
        assert!(length <= self.a.len());
        let mut output = vec![0; length];
        for i in 0..length {
            unsafe {
                height += self.a.get_unchecked(i);
                *(output.get_unchecked_mut(i)) = ((height) * 255.9) as u8;
            }
        }
        output
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn get_bitmap(&self) -> Vec<u8> {
        let length = self.w * self.h;
        let aligned_length = (length + 3) & !3;
        assert!(aligned_length <= self.a.len());
        // Turns out zeroing takes a while on very large sizes.
        let mut output = Vec::with_capacity(aligned_length);
        unsafe {
            output.set_len(aligned_length);
            // offset = Zeroed out lanes
            let mut offset = _mm_setzero_ps();
            // lookup = The 4 bytes (12, 8, 4, 0) in all lanes
            let lookup = _mm_set1_epi32(0x0c_08_04_00);
            for i in (0..aligned_length).step_by(4) {
                // x = Read 4 floats from self.a
                let mut x = _mm_loadu_ps(self.a.get_unchecked(i));
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
                // (SSSE3) y = Take the first byte of each of the 4 values in y and pack them into
                // the first 4 bytes of y. This produces the same value in all 4 lanes.
                y = _mm_shuffle_epi8(y, lookup);

                // Store the first 4 u8s from y in output. The cast again is a nop.
                _mm_store_ss(core::mem::transmute(output.get_unchecked_mut(i)), _mm_castsi128_ps(y));
                // offset = (x[3], x[3], x[3], x[3])
                offset = _mm_set1_ps(core::mem::transmute::<__m128, [f32; 4]>(x)[3]);
            }
        }
        output.truncate(length);
        output
    }
}
