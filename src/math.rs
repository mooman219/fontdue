use crate::platform::{abs, atan2, clamp, f32x4};
use crate::{FontSettings, AABB};
use alloc::vec::*;

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

    pub fn offset(&self, settings: &FontSettings) -> Point {
        Point {
            x: self.x + settings.offset_x,
            y: self.y + settings.offset_y,
        }
    }
}

#[derive(Copy, Clone)]
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
            8388608.0 // 2^23, doesn't really matter.
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
    pub lines: Vec<Line>,
    pub effective_bounds: AABB,
    pub start_point: Point,
    pub previous_point: Point,
    pub settings: FontSettings,
    pub max_angle: f32,
    pub reverse_points: bool,
}

impl ttf_parser::OutlineBuilder for Geometry {
    fn move_to(&mut self, x0: f32, y0: f32) {
        let next_point = Point::new(x0, y0).offset(&self.settings);
        self.start_point = next_point;
        self.previous_point = next_point;
    }

    fn line_to(&mut self, x0: f32, y0: f32) {
        let next_point = Point::new(x0, y0).offset(&self.settings);
        self.push(self.previous_point, next_point);
        self.previous_point = next_point;
    }

    fn quad_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        let control_point = Point::new(x0, y0).offset(&self.settings);
        let next_point = Point::new(x1, y1).offset(&self.settings);

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
        let first_control = Point::new(x0, y0).offset(&self.settings);
        let second_control = Point::new(x1, y1).offset(&self.settings);
        let next_point = Point::new(x2, y2).offset(&self.settings);

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
    pub fn new(settings: FontSettings, reverse_points: bool) -> Geometry {
        const PI: f32 = 3.14159265359;
        const LOW_SIZE: f32 = 20.0;
        const LOW_ANGLE: f32 = 17.0;
        const HIGH_SIZE: f32 = 125.0;
        const HIGH_ANGLE: f32 = 5.0;
        const MAX_ANGLE: f32 = 3.0;
        const SLOPE: f32 = (HIGH_ANGLE - LOW_ANGLE) / (HIGH_SIZE - LOW_SIZE);
        const YINT: f32 = SLOPE * -HIGH_SIZE + HIGH_ANGLE;
        let max_angle = settings.scale * SLOPE + YINT;
        let max_angle = clamp(MAX_ANGLE, LOW_ANGLE, max_angle);
        let max_angle = max_angle * PI / 180.0; // Convert into rads
        Geometry {
            lines: Vec::new(),
            effective_bounds: AABB::new(core::f32::MAX, core::f32::MIN, core::f32::MAX, core::f32::MIN),
            start_point: Point::default(),
            previous_point: Point::default(),
            settings,
            max_angle,
            reverse_points,
        }
    }

    pub fn finalize(&mut self) {
        if self.lines.is_empty() {
            self.effective_bounds = AABB::default();
        } else {
            self.lines.shrink_to_fit();
            for line in &mut self.lines {
                line.reposition(self.effective_bounds);
            }
        }
    }

    fn push(&mut self, start: Point, end: Point) {
        let (a, b) = if self.reverse_points {
            (end, start)
        } else {
            (start, end)
        };
        if a.y != b.y {
            self.lines.push(Line::new(a, b));
            self.recalculate_bounds(a);
            self.recalculate_bounds(b);
        }
    }

    fn recalculate_bounds(&mut self, point: Point) {
        let bounds = &mut self.effective_bounds;
        if point.x < bounds.xmin {
            bounds.xmin = point.x;
        }
        if point.x > bounds.xmax {
            bounds.xmax = point.x;
        }
        if point.y < bounds.ymin {
            bounds.ymin = point.y;
        }
        if point.y > bounds.ymax {
            bounds.ymax = point.y;
        }
    }
}
