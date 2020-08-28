use crate::platform::{abs, atan, f32x4};
use crate::{FontSettings, AABB};
use alloc::vec::*;

#[derive(Copy, Clone, Debug, PartialEq)]
struct Curve {
    a: Point,
    b: Point,
    c: Point,
}

impl Curve {
    fn new(a: Point, b: Point, c: Point) -> Curve {
        Curve {
            a,
            b,
            c,
        }
    }

    /// The point at time t in the curve.
    fn point(&self, t: f32) -> Point {
        let a = 1.0 - t;
        let a = a * a;
        let x = a * self.a.x + 2.0 * (1.0 - t) * t * self.b.x + (t * t) * self.c.x;
        let y = a * self.a.y + 2.0 * (1.0 - t) * t * self.b.y + (t * t) * self.c.y;
        Point::new(x, y)
    }

    /// The slope of the tangent line at time t.
    fn slope(&self, t: f32) -> f32 {
        let x = 2.0 * (1.0 - t) * (self.b.x - self.a.x) + 2.0 * t * (self.c.x - self.b.x);
        let y = 2.0 * (1.0 - t) * (self.b.y - self.a.y) + 2.0 * t * (self.c.y - self.b.y);
        y / x
    }

    /// The angle of the tangent line at time t.
    fn angle(&self, t: f32) -> f32 {
        const PI: f32 = 3.14159265359;
        abs(atan(self.slope(t))) * 180.0 / PI
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

    pub fn offset(&self, settings: &FontSettings) -> Point {
        let mut new = *self;
        new.x += settings.offset_x;
        new.y += settings.offset_y;
        new
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
    pub effective_bounds: Option<AABB>,
    pub previous: Point,
    pub settings: FontSettings,
}

impl ttf_parser::OutlineBuilder for Geometry {
    fn move_to(&mut self, x0: f32, y0: f32) {
        self.previous = Point::new(x0, y0).offset(&self.settings);
    }

    fn line_to(&mut self, x0: f32, y0: f32) {
        let current = Point::new(x0, y0).offset(&self.settings);
        self.push(self.previous, current);
        self.previous = current;
    }

    fn quad_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        let current = Point::new(x0, y0).offset(&self.settings);
        let next = Point::new(x1, y1).offset(&self.settings);

        const MAX_ANGLE: f32 = 17.0;
        const SUBDIVISIONS: u32 = 20;
        const INCREMENT: f32 = 1.0 / (1.0 + SUBDIVISIONS as f32);
        let curve = Curve::new(self.previous, current, next);
        let mut previous_point = self.previous;
        let mut previous_angle = curve.angle(0.0);
        for x in 1..=SUBDIVISIONS {
            let t = INCREMENT * x as f32;
            let temp_angle = curve.angle(t);
            if abs(previous_angle - temp_angle) > MAX_ANGLE {
                previous_angle = temp_angle;
                let temp_point = curve.point(t);
                self.push(previous_point, temp_point);
                previous_point = temp_point;
            }
        }
        self.push(previous_point, next);

        self.previous = next;
    }

    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, x2: f32, y2: f32) {
        self.previous = Point::new(x2, y2).offset(&self.settings);
    }

    fn close(&mut self) {}
}

impl Geometry {
    pub fn new(settings: FontSettings) -> Geometry {
        Geometry {
            lines: Vec::new(),
            effective_bounds: None,
            previous: Point::default(),
            settings,
        }
    }

    pub fn reposition(&mut self) {
        if let Some(bounds) = self.effective_bounds {
            for line in &mut self.lines {
                line.reposition(bounds);
            }
        }
    }

    fn push(&mut self, start: Point, end: Point) {
        if start.y != end.y {
            self.lines.push(Line::new(start, end));
            self.recalculate_bounds(start);
            self.recalculate_bounds(end);
        }
    }

    fn recalculate_bounds(&mut self, point: Point) {
        if let Some(bounds) = self.effective_bounds.as_mut() {
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
        } else {
            self.effective_bounds = Some(AABB::new(point.x, point.x, point.y, point.y))
        }
    }
}
