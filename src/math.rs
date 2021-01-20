use crate::platform::{abs, atan2, clamp, f32x4};
use crate::{Glyph, OutlineBounds};
use alloc::vec::*;

#[cfg(feature = "serde_derive")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Debug)]
struct AABB {
    /// Coordinate of the left-most edge.
    xmin: f32,
    /// Coordinate of the right-most edge.
    xmax: f32,
    /// Coordinate of the bottom-most edge.
    ymin: f32,
    /// Coordinate of the top-most edge.
    ymax: f32,
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            xmin: 0.0,
            xmax: 0.0,
            ymin: 0.0,
            ymax: 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct CubeCurve {
    a: Point,
    b: Point,
    c: Point,
    d: Point,
}

impl CubeCurve {
    fn new(a: Point, b: Point, c: Point, d: Point) -> CubeCurve {
        CubeCurve {
            a,
            b,
            c,
            d,
        }
    }

    fn scale(&self, scale: f32) -> CubeCurve {
        CubeCurve {
            a: self.a.scale(scale),
            b: self.b.scale(scale),
            c: self.c.scale(scale),
            d: self.d.scale(scale),
        }
    }

    fn is_flat(&self, threshold: f32) -> bool {
        let (d1, d2, d3, d4) = f32x4::new(
            self.a.squared_distance(self.b),
            self.b.squared_distance(self.c),
            self.c.squared_distance(self.d),
            self.a.squared_distance(self.d),
        )
        .sqrt()
        .copied();
        (d1 + d2 + d3) < threshold * d4
    }

    fn split(&self) -> (CubeCurve, CubeCurve) {
        let q0 = self.a.midpoint(self.b);
        let q1 = self.b.midpoint(self.c);
        let q2 = self.c.midpoint(self.d);
        let r0 = q0.midpoint(q1);
        let r1 = q1.midpoint(q2);
        let s0 = r0.midpoint(r1);
        (CubeCurve::new(self.a, q0, r0, s0), CubeCurve::new(s0, r1, q2, self.d))
    }

    /// The point at time t in the curve.
    fn point(&self, t: f32) -> Point {
        let tm = 1.0 - t;
        let a = tm * tm * tm;
        let b = 3.0 * (tm * tm) * t;
        let c = 3.0 * tm * (t * t);
        let d = t * t * t;

        let x = a * self.a.x + b * self.b.x + c * self.c.x + d * self.d.x;
        let y = a * self.a.y + b * self.b.y + c * self.c.y + d * self.d.y;
        Point::new(x, y)
    }

    /// The slope of the tangent line at time t.
    fn slope(&self, t: f32) -> (f32, f32) {
        let tm = 1.0 - t;
        let a = 3.0 * (tm * tm);
        let b = 6.0 * tm * t;
        let c = 3.0 * (t * t);

        let x = a * (self.b.x - self.a.x) + b * (self.c.x - self.b.x) + c * (self.d.x - self.c.x);
        let y = a * (self.b.y - self.a.y) + b * (self.c.y - self.b.y) + c * (self.d.y - self.c.y);
        (x, y)
    }

    /// The angle of the tangent line at time t in rads.
    fn angle(&self, t: f32) -> f32 {
        let (x, y) = self.slope(t);
        abs(atan2(x, y))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct QuadCurve {
    a: Point,
    b: Point,
    c: Point,
}

impl QuadCurve {
    fn new(a: Point, b: Point, c: Point) -> QuadCurve {
        QuadCurve {
            a,
            b,
            c,
        }
    }

    fn scale(&self, scale: f32) -> QuadCurve {
        QuadCurve {
            a: self.a.scale(scale),
            b: self.b.scale(scale),
            c: self.c.scale(scale),
        }
    }

    fn is_flat(&self, threshold: f32) -> bool {
        let (d1, d2, d3, _) = f32x4::new(
            self.a.squared_distance(self.b),
            self.b.squared_distance(self.c),
            self.a.squared_distance(self.c),
            1.0,
        )
        .sqrt()
        .copied();
        (d1 + d2) < threshold * d3
    }

    fn split(&self) -> (QuadCurve, QuadCurve) {
        let q0 = self.a.midpoint(self.b);
        let q1 = self.b.midpoint(self.c);
        let r0 = q0.midpoint(q1);
        (QuadCurve::new(self.a, q0, r0), QuadCurve::new(r0, q1, self.c))
    }

    /// The point at time t in the curve.
    fn point(&self, t: f32) -> Point {
        let tm = 1.0 - t;
        let a = tm * tm;
        let b = 2.0 * tm * t;
        let c = t * t;

        let x = a * self.a.x + b * self.b.x + c * self.c.x;
        let y = a * self.a.y + b * self.b.y + c * self.c.y;
        Point::new(x, y)
    }

    /// The slope of the tangent line at time t.
    fn slope(&self, t: f32) -> (f32, f32) {
        let tm = 1.0 - t;
        let a = 2.0 * tm;
        let b = 2.0 * t;

        let x = a * (self.b.x - self.a.x) + b * (self.c.x - self.b.x);
        let y = a * (self.b.y - self.a.y) + b * (self.c.y - self.b.y);
        (x, y)
    }

    /// The angle of the tangent line at time t in rads.
    fn angle(&self, t: f32) -> f32 {
        let (x, y) = self.slope(t);
        abs(atan2(x, y))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    /// Absolute X coordinate.
    pub x: f32,
    /// Absolute Y coordinate.
    pub y: f32,
}

impl Default for Point {
    fn default() -> Self {
        Point {
            x: 0.0,
            y: 0.0,
        }
    }
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point {
            x,
            y,
        }
    }

    pub fn scale(&self, scale: f32) -> Point {
        Point {
            x: self.x * scale,
            y: self.y * scale,
        }
    }

    pub fn squared_distance(&self, other: Point) -> f32 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        x * x + y * y
    }

    pub fn midpoint(&self, other: Point) -> Point {
        Point {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub struct Line {
    /// X0, Y0, X1, Y1.
    pub coords: f32x4,
    /// start_x_nudge, start_y_nudge, end_x_nudge, end_y_nudge.
    pub nudge: f32x4,
    /// x_first_adj, y_first_adj, none, none.
    pub adjustment: f32x4,
    /// tdx, tdy, dx, dy.
    pub params: f32x4,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Line {
        // Floor adjustment and nudge: 0.0, 0
        // Ceil adjustment and nudge: 1.0, 1
        const FLOOR_NUDGE: u32 = 0;
        const CEIL_NUDGE: u32 = 1;

        let (x_start_nudge, x_first_adj) = if end.x >= start.x {
            (FLOOR_NUDGE, 1.0)
        } else {
            (CEIL_NUDGE, 0.0)
        };
        let (y_start_nudge, y_first_adj) = if end.y >= start.y {
            (FLOOR_NUDGE, 1.0)
        } else {
            (CEIL_NUDGE, 0.0)
        };

        let x_end_nudge = if end.x > start.x {
            CEIL_NUDGE
        } else {
            FLOOR_NUDGE
        };
        let y_end_nudge = if end.y > start.y {
            CEIL_NUDGE
        } else {
            FLOOR_NUDGE
        };

        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let tdx = if dx == 0.0 {
            core::f32::MAX
        } else {
            1.0 / dx
        };
        let tdy = 1.0 / dy;

        Line {
            coords: f32x4::new(start.x, start.y, end.x, end.y),
            nudge: f32x4::new_u32(x_start_nudge, y_start_nudge, x_end_nudge, y_end_nudge),
            adjustment: f32x4::new(x_first_adj, y_first_adj, 0.0, 0.0),
            params: f32x4::new(tdx, tdy, dx, dy),
        }
    }

    fn reposition(&mut self, bounds: AABB) {
        let (mut x0, mut y0, mut x1, mut y1) = self.coords.copied();

        x0 -= bounds.xmin;
        y0 -= bounds.ymax;
        y0 = abs(y0);

        x1 -= bounds.xmin;
        y1 -= bounds.ymax;
        y1 = abs(y1);

        *self = Self::new(Point::new(x0, y0), Point::new(x1, y1));
    }
}

#[derive(Clone)]
pub struct Geometry {
    v_lines: Vec<Line>,
    m_lines: Vec<Line>,
    effective_bounds: AABB,
    start_point: Point,
    previous_point: Point,
    max_angle: f32,
    reverse_points: bool,
}

impl ttf_parser::OutlineBuilder for Geometry {
    fn move_to(&mut self, x0: f32, y0: f32) {
        let next_point = Point::new(x0, y0);
        self.start_point = next_point;
        self.previous_point = next_point;
    }

    fn line_to(&mut self, x0: f32, y0: f32) {
        let next_point = Point::new(x0, y0);
        self.push(self.previous_point, next_point);
        self.previous_point = next_point;
    }

    fn quad_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        let control_point = Point::new(x0, y0);
        let next_point = Point::new(x1, y1);

        const STEPS: u32 = 20;
        const INCREMENT: f32 = 1.0 / (STEPS as f32);
        let curve = QuadCurve::new(self.previous_point, control_point, next_point);
        let mut previous_angle = curve.angle(0.0);
        for x in 1..STEPS {
            let t = INCREMENT * x as f32;
            let temp_angle = curve.angle(t);
            if abs(previous_angle - temp_angle) > self.max_angle {
                previous_angle = temp_angle;
                let temp_point = curve.point(t);
                self.push(self.previous_point, temp_point);
                self.previous_point = temp_point;
            }
        }
        self.push(self.previous_point, next_point);

        self.previous_point = next_point;
    }

    fn curve_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        let first_control = Point::new(x0, y0);
        let second_control = Point::new(x1, y1);
        let next_point = Point::new(x2, y2);

        const STEPS: u32 = 20;
        const INCREMENT: f32 = 1.0 / (STEPS as f32);
        let curve = CubeCurve::new(self.previous_point, first_control, second_control, next_point);
        let mut previous_angle = curve.angle(0.0);
        for x in 1..STEPS {
            let t = INCREMENT * x as f32;
            let temp_angle = curve.angle(t);
            if abs(previous_angle - temp_angle) > self.max_angle {
                previous_angle = temp_angle;
                let temp_point = curve.point(t);
                self.push(self.previous_point, temp_point);
                self.previous_point = temp_point;
            }
        }
        self.push(self.previous_point, next_point);

        self.previous_point = next_point;
    }

    fn close(&mut self) {
        if self.start_point != self.previous_point {
            self.push(self.previous_point, self.start_point);
        }
        self.previous_point = self.start_point;
    }
}

impl Geometry {
    pub fn new(scale: f32, reverse_points: bool) -> Geometry {
        const PI: f32 = 3.14159265359;
        const LOW_SIZE: f32 = 20.0;
        const LOW_ANGLE: f32 = 17.0;
        const HIGH_SIZE: f32 = 125.0;
        const HIGH_ANGLE: f32 = 5.0;
        const MAX_ANGLE: f32 = 3.0;
        const SLOPE: f32 = (HIGH_ANGLE - LOW_ANGLE) / (HIGH_SIZE - LOW_SIZE);
        const YINT: f32 = SLOPE * -HIGH_SIZE + HIGH_ANGLE;
        let max_angle = scale * SLOPE + YINT;
        let max_angle = clamp(MAX_ANGLE, LOW_ANGLE, max_angle);
        let max_angle = max_angle * PI / 180.0; // Convert into rads
        Geometry {
            v_lines: Vec::new(),
            m_lines: Vec::new(),
            effective_bounds: AABB {
                xmin: core::f32::MAX,
                xmax: core::f32::MIN,
                ymin: core::f32::MAX,
                ymax: core::f32::MIN,
            },
            start_point: Point::default(),
            previous_point: Point::default(),
            max_angle,
            reverse_points,
        }
    }

    fn push(&mut self, start: Point, end: Point) {
        if start.y != end.y {
            let (start, end) = if self.reverse_points {
                (end, start)
            } else {
                (start, end)
            };
            if start.x == end.x {
                self.v_lines.push(Line::new(start, end));
            } else {
                self.m_lines.push(Line::new(start, end));
            }
        }
    }

    pub(crate) fn finalize(mut self, glyph: &mut Glyph) {
        if self.v_lines.is_empty() && self.m_lines.is_empty() {
            self.effective_bounds = AABB::default();
        } else {
            for line in self.v_lines.iter().chain(self.m_lines.iter()) {
                let (x0, y0, x1, y1) = line.coords.copied();
                Self::recalculate_bounds(&mut self.effective_bounds, x0, y0);
                Self::recalculate_bounds(&mut self.effective_bounds, x1, y1);
            }
            for line in self.v_lines.iter_mut().chain(self.m_lines.iter_mut()) {
                line.reposition(self.effective_bounds);
            }
            self.v_lines.shrink_to_fit();
            self.m_lines.shrink_to_fit();
        }
        glyph.v_lines = self.v_lines;
        glyph.m_lines = self.m_lines;
        glyph.bounds = OutlineBounds {
            xmin: self.effective_bounds.xmin,
            ymin: self.effective_bounds.ymin,
            width: self.effective_bounds.xmax - self.effective_bounds.xmin,
            height: self.effective_bounds.ymax - self.effective_bounds.ymin,
        };
    }

    fn recalculate_bounds(bounds: &mut AABB, x: f32, y: f32) {
        if x < bounds.xmin {
            bounds.xmin = x;
        }
        if x > bounds.xmax {
            bounds.xmax = x;
        }
        if y < bounds.ymin {
            bounds.ymin = y;
        }
        if y > bounds.ymax {
            bounds.ymax = y;
        }
    }
}
