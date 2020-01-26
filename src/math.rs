use crate::raw::RawPoint;
use alloc::vec::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    /// Absolute X coordinate.
    pub x: f32,
    /// Absolute Y coordinate.
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point {
            x,
            y,
        }
    }

    pub fn new_raw(p: &RawPoint) -> Point {
        Point {
            x: p.x,
            y: p.y,
        }
    }

    pub fn lerp(t: f32, a: &Point, b: &Point) -> Point {
        Point {
            x: a.x + t * (b.x - a.x),
            y: a.y + t * (b.y - a.y),
        }
    }

    pub fn lerp_raw(t: f32, a: &RawPoint, b: &RawPoint) -> Point {
        Point {
            x: a.x + t * (b.x - a.x),
            y: a.y + t * (b.y - a.y),
        }
    }

    pub fn midpoint_raw(a: &RawPoint, b: &RawPoint) -> Point {
        Point {
            x: (a.x + b.x) / 2.0,
            y: (a.y + b.y) / 2.0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// Variable names for job security.
pub struct Line {
    /// X0, Y0, X1, Y1.
    pub abcd: wide::f32x4,
    pub x_mod: f32,
    pub y_mod: f32,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Line {
        let x_mod = if end.x >= start.x {
            1.0
        } else {
            0.0
        };
        let y_mod = if end.y >= start.y {
            1.0
        } else {
            0.0
        };
        Line {
            abcd: wide::f32x4::new(start.x, start.y, end.x, end.y),
            x_mod,
            y_mod,
        }
    }

    pub fn new_raw(start: &RawPoint, end: &RawPoint) -> Line {
        Line::new(Point::new_raw(start), Point::new_raw(end))
    }

    pub fn scale_wide(&mut self, scale: wide::f32x4) {
        self.abcd *= scale;
    }

    pub fn mirror_x(&mut self, y: f32) {
        let &[x0, y0, x1, y1] = self.abcd.as_ref();
        let y0 = y0 + ((y - y0) * 2.0);
        let y1 = y1 + ((y - y1) * 2.0);
        self.abcd = wide::f32x4::new(x0, y0, x1, y1);
        self.y_mod = if y1 >= y0 {
            1.0
        } else {
            0.0
        };
    }

    pub fn offset_wide(&mut self, offset: wide::f32x4) {
        self.abcd += offset;
    }
}

pub struct Polygons {
    pub lines: Vec<Line>,
}

impl Polygons {
    pub fn new() -> Polygons {
        Polygons {
            lines: Vec::new(),
        }
    }

    /// Scales all X and Y components by the given scale.
    pub fn scale(&mut self, scale: f32) {
        let scale = wide::f32x4::from(scale);
        for line in &mut self.lines {
            line.scale_wide(scale);
        }
    }

    /// Mirrors the points ofer the horizontal line at the given y.
    pub fn mirror_x(&mut self, y: f32) {
        for line in &mut self.lines {
            line.mirror_x(y);
        }
    }

    /// Offsets the points by the given x and y.
    pub fn offset(&mut self, x: f32, y: f32) {
        let offset = wide::f32x4::new(x, y, x, y);
        for line in &mut self.lines {
            line.offset_wide(offset);
        }
    }
}

const SUBDIVISIONS: u32 = 3;

/// Calculates the point at t on a quadratic bezier curve.
fn curve_point(t: f32, previous: Point, current: Point, next: Point) -> Point {
    let x = (1.0 - t).powi(2) * previous.x + 2.0 * (1.0 - t) * t * current.x + t.powi(2) * next.x;
    let y = (1.0 - t).powi(2) * previous.y + 2.0 * (1.0 - t) * t * current.y + t.powi(2) * next.y;
    Point::new(x, y)
}

fn populate_lines(polygons: &mut Polygons, previous: &RawPoint, current: &RawPoint, next: &RawPoint) {
    if !current.on_curve() {
        // Curve. We're off the curve, find the on-curve positions for the previous and next points
        // then make a curve out of that.
        let previous = if previous.on_curve() {
            Point::new_raw(&previous)
        } else {
            Point::midpoint_raw(&previous, current)
        };
        let next = if next.on_curve() {
            Point::new_raw(&next)
        } else {
            Point::midpoint_raw(current, &next)
        };
        let current = Point::new_raw(current);

        if SUBDIVISIONS <= 1 {
            polygons.lines.push(Line::new(previous, current));
            polygons.lines.push(Line::new(current, next));
        } else {
            let increment = 1.0 / (SUBDIVISIONS as f32);
            for x in 0..SUBDIVISIONS {
                let t0 = increment * (x as f32);
                let t1 = increment * ((x + 1) as f32);
                let p0 = curve_point(t0, previous, current, next);
                let p1 = curve_point(t1, previous, current, next);
                if p0.y != p1.y {
                    polygons.lines.push(Line::new(p0, p1));
                }
            }
        }
    } else if next.on_curve() {
        // Line. Both the current and the next point are on the curve, it's a line.
        if current.y != next.y {
            polygons.lines.push(Line::new_raw(current, next));
        }
    } else {
        // Do nothing. The current point is on the curve but the next one isn't, so the next point
        // will end up drawing the curve that the current point is on.
    }
}

pub fn to_polygons(points: &[RawPoint]) -> Polygons {
    let mut polygons = Polygons::new();
    let mut first = RawPoint::default();
    let mut second = RawPoint::default();
    let mut previous = RawPoint::default();
    let mut current = RawPoint::default();
    let mut index = 0;
    for next in points {
        match index {
            0 => {
                first = *next;
                previous = *next;
            }
            1 => {
                second = *next;
                current = *next;
            }
            _ => {
                populate_lines(&mut polygons, &previous, &current, next);
                if next.end_point {
                    populate_lines(&mut polygons, &current, next, &first);
                    populate_lines(&mut polygons, next, &first, &second);
                    index = -1;
                } else {
                    previous = current;
                    current = *next;
                }
            }
        }
        index += 1;
    }
    polygons
}
