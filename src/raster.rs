use crate::math::Line;
use crate::platform::{abs, as_i32, copysign, f32x4, fract};
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

    pub fn draw(&mut self, lines: &Vec<Line>, scale: f32, offset_x: f32, offset_y: f32) {
        let params = f32x4::new(1.0 / scale, 1.0 / scale, scale, scale);
        let scale = f32x4::splat(scale);
        let offset = f32x4::new(offset_x, offset_y, offset_x, offset_y);
        for line in lines {
            self.line(line, line.coords * scale + offset, line.params * params);
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
    fn line(&mut self, line: &Line, coords: f32x4, params: f32x4) {
        let (x0, y0, x1, y1) = coords.copied();
        let (start_x, start_y, end_x, end_y) = coords.sub_integer(line.nudge).trunc().copied();
        let (mut target_x, mut target_y, _, _) =
            (coords + line.adjustment).sub_integer(line.nudge).trunc().copied();
        let (tdx, tdy, dx, dy) = params.copied();
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
        // The (tmx < 1.0 || tmy < 1.0) condition does not work due to rounding errors in f32, so
        // dist is used instead to cap the iteration count.
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
            self.add(prev_index as usize, y_prev - y_next, (x_prev + x_next) / 2.0);
            x_prev = x_next;
            y_prev = y_next;
        }
        self.add(as_i32(end_x + end_y * self.w as f32) as usize, y_prev - y1, (x_prev + x1) / 2.0);
    }

    #[inline(always)]
    pub fn get_bitmap(&self) -> Vec<u8> {
        crate::platform::get_bitmap(&self.a, self.w * self.h)
    }
}
