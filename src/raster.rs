// Raster from font-rs.
// Includes changes for a performance and correctness.

use crate::math::{Geometry, Point};
use alloc::vec;
use alloc::vec::*;
use core::cmp::min;

fn accumulate(src: &[f32]) -> Vec<u8> {
    let mut acc = 0.0;
    let mut output = Vec::with_capacity(src.len());
    unsafe { output.set_len(src.len()) };
    for i in 0..src.len() {
        acc += unsafe { src.get_unchecked(i) };
        let y = acc.abs();
        let y = if y < 1.0 {
            y
        } else {
            1.0
        };
        unsafe { *(output.get_unchecked_mut(i)) = (255.99998 * y) as u8 };
    }
    output
}

pub struct Raster {
    w: usize,
    h: usize,
    a: Vec<f32>,
}

impl Raster {
    pub fn new(w: usize, h: usize) -> Raster {
        Raster {
            w: w,
            h: h,
            a: vec![0.0; w * h + 4],
        }
    }

    pub fn draw(&mut self, geometry: &Geometry) {
        match geometry {
            Geometry::Line(a, b) => self.draw_line(a, b),
            Geometry::Curve(a, b, c) => self.draw_curve(a, b, c),
        }
    }

    #[inline(always)]
    fn add(&mut self, index: i32, value: f32) {
        // This can technically go out of bounds if the glyph isn't offset, or if the font is
        // malicious. These situation are corrected for in font.rs.
        self.a[index as usize] += value;
    }

    pub fn draw_line(&mut self, p0: &Point, p1: &Point) {
        if p0.y == p1.y {
            return;
        }
        let (dir, p0, p1) = if p0.y < p1.y {
            (1.0, p0, p1)
        } else {
            (-1.0, p1, p0)
        };
        let dxdy = (p1.x - p0.x) / (p1.y - p0.y);
        let mut x = p0.x;
        // note: implicit max of 0 because usize (TODO: really true?)
        let y0 = p0.y as usize;
        if p0.y < 0.0 {
            x -= p0.y * dxdy;
        }
        for y in y0..min(self.h, p1.y.ceil() as usize) {
            let linestart = (y * self.w) as i32;
            let dy = ((y + 1) as f32).min(p1.y) - (y as f32).max(p0.y);
            let xnext = x + dxdy * dy;
            let d = dy * dir;
            let (x0, x1) = if x < xnext {
                (x, xnext)
            } else {
                (xnext, x)
            };
            let x0floor = x0.floor();
            let x0i = x0floor as i32;
            let x1ceil = x1.ceil();
            let x1i = x1ceil as i32;
            if x1i <= x0i + 1 {
                let xmf = 0.5 * (x + xnext) - x0floor;
                self.add(linestart + x0i, d - d * xmf);
                self.add(linestart + x0i + 1, d * xmf);
            } else {
                let s = (x1 - x0).recip();
                let x0f = x0 - x0floor;
                let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
                let x1f = x1 - x1ceil + 1.0;
                let am = 0.5 * s * x1f * x1f;
                self.add(linestart + x0i, d * a0);
                if x1i == x0i + 2 {
                    self.add(linestart + x0i + 1, d * (1.0 - a0 - am));
                } else {
                    let a1 = s * (1.5 - x0f);
                    self.add(linestart + x0i + 1, d * (a1 - a0));
                    for xi in x0i + 2..x1i - 1 {
                        self.add(linestart + xi, d * s);
                    }
                    let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                    self.add(linestart + x1i - 1, d * (1.0 - a2 - am));
                }
                self.add(linestart + x1i, d * am);
            }
            x = xnext;
        }
    }

    pub fn draw_curve(&mut self, p0: &Point, p1: &Point, p2: &Point) {
        let devx = p0.x - 2.0 * p1.x + p2.x;
        let devy = p0.y - 2.0 * p1.y + p2.y;
        let devsq = devx * devx + devy * devy;
        if devsq < 0.333 {
            self.draw_line(p0, p2);
            return;
        }
        let tol = 3.0;
        let n = 1 + (tol * (devx * devx + devy * devy)).sqrt().sqrt().floor() as usize;
        let mut p = *p0;
        let nrecip = (n as f32).recip();
        let mut t = 0.0;
        for _i in 0..n - 1 {
            t += nrecip;
            let pn = Point::lerp(t, &Point::lerp(t, p0, p1), &Point::lerp(t, p1, p2));
            self.draw_line(&p, &pn);
            p = pn;
        }
        self.draw_line(&p, p2);
    }

    pub fn get_bitmap(&self) -> Vec<u8> {
        accumulate(&self.a[0..self.w * self.h])
    }
}
