/* Notice to anyone that wants to repurpose the raster for your library:
 * Please don't reuse this raster. Fontdue's raster is very unsafe, with nuanced invariants that
 * need to be accounted for. Fontdue sanitizes the input that the raster will consume to ensure it
 * is safe. Please be aware of this.
 */

use crate::math::Line;
use crate::platform::{abs, as_i32, copysign, f32x4, fract};
use crate::Glyph;
use alloc::vec;
use alloc::vec::*;

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

    pub(crate) fn draw(&mut self, glyph: &Glyph, scale_x: f32, scale_y: f32, offset_x: f32, offset_y: f32) {
        let params = f32x4::new(1.0 / scale_x, 1.0 / scale_y, scale_x, scale_y);
        let scale = f32x4::new(scale_x, scale_y, scale_x, scale_y);
        let offset = f32x4::new(offset_x, offset_y, offset_x, offset_y);
        for line in &glyph.v_lines {
            self.v_line(line, line.coords * scale + offset);
        }
        for line in &glyph.m_lines {
            self.m_line(line, line.coords * scale + offset, line.params * params);
        }
    }

    #[inline(always)]
    fn add(&mut self, index: usize, height: f32, mid_x: f32) {
        // This is fast and hip.
        unsafe {
            let m = height * mid_x;
            *self.a.get_unchecked_mut(index) += height - m;
            *self.a.get_unchecked_mut(index + 1) += m;
        }

        // This is safe but slow.
        // let m = height * mid_x;
        // self.a[index] += height - m;
        // self.a[index + 1] += m;
    }

    #[inline(always)]
    fn v_line(&mut self, line: &Line, coords: f32x4) {
        let (x0, y0, _, y1) = coords.copied();
        let temp = coords.sub_integer(line.nudge).trunc();
        let (start_x, start_y, end_x, end_y) = temp.copied();
        let (_, mut target_y, _, _) = (temp + line.adjustment).copied();
        let sy = copysign(1f32, y1 - y0);
        let mut y_prev = y0;
        let mut index = as_i32(start_x + start_y * self.w as f32);
        let index_y_inc = as_i32(copysign(self.w as f32, sy));
        let mut dist = as_i32(abs(start_y - end_y));
        let mid_x = fract(x0);
        while dist > 0 {
            dist -= 1;
            self.add(index as usize, y_prev - target_y, mid_x);
            index += index_y_inc;
            y_prev = target_y;
            target_y += sy;
        }
        self.add(as_i32(end_x + end_y * self.w as f32) as usize, y_prev - y1, mid_x);
    }

    #[inline(always)]
    fn m_line(&mut self, line: &Line, coords: f32x4, params: f32x4) {
        let (x0, y0, x1, y1) = coords.copied();
        let temp = coords.sub_integer(line.nudge).trunc();
        let (start_x, start_y, end_x, end_y) = temp.copied();
        let (tdx, tdy, dx, dy) = params.copied();
        let (mut target_x, mut target_y, _, _) = (temp + line.adjustment).copied();
        let sx = copysign(1f32, tdx);
        let sy = copysign(1f32, tdy);
        let mut tmx = tdx * (target_x - x0);
        let mut tmy = tdy * (target_y - y0);
        let tdx = abs(tdx);
        let tdy = abs(tdy);
        let mut x_prev = x0;
        let mut y_prev = y0;
        let mut index = as_i32(start_x + start_y * self.w as f32);
        let index_x_inc = as_i32(sx);
        let index_y_inc = as_i32(copysign(self.w as f32, sy));
        let mut dist = as_i32(abs(start_x - end_x) + abs(start_y - end_y));
        while dist > 0 {
            dist -= 1;
            let prev_index = index;
            let y_next: f32;
            let x_next: f32;
            if tmx < tmy {
                y_next = tmx * dy + y0; // FMA is not faster.
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
            self.add(prev_index as usize, y_prev - y_next, fract((x_prev + x_next) / 2.0));
            x_prev = x_next;
            y_prev = y_next;
        }
        self.add(as_i32(end_x + end_y * self.w as f32) as usize, y_prev - y1, fract((x_prev + x1) / 2.0));
    }

    #[inline(always)]
    pub fn get_bitmap(&self) -> Vec<u8> {
        crate::platform::get_bitmap(&self.a, self.w * self.h)
    }
}
